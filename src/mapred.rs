use std::collections::HashMap;
use std::collections::hash_map::Entry;

use simple_parallel::pool::Pool;

use std::sync::Mutex;

use pbr::ProgressBar;

pub type BI<'a, A> = Box<Iterator<Item = A> + Send + 'a>;

pub enum Emit<K, V> {
    None,
    One(K, V),
    Vec(Vec<(K, V)>),
}

trait Aggregator<R,K,V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
        K: Send + Eq + ::std::hash::Hash + 'static,
        V: Send + 'static
{
    fn create_inlet<'b>(&'b self) -> Box<Inlet<R, K, V> + 'b>;
}


trait Inlet<R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static
{
    fn push(&mut self, e: Emit<K, V>);
}

struct HashMapAggregator<'a, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static
{
    hashmap: Mutex<&'a mut HashMap<K, V>>,
    reducer: &'a R,
}

impl<'a, R, K, V> Aggregator<R, K, V> for HashMapAggregator<'a, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static
{
    fn create_inlet<'b>(&'b self) -> Box<Inlet<R, K, V> + 'b> {
        Box::new(HashMapInlet {
            parent: &self,
            partial: HashMap::new(),
        })
    }
}

struct HashMapInlet<'a, 'b, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static,
          'a: 'b
{
    parent: &'b HashMapAggregator<'a, R, K, V>,
    partial: HashMap<K, V>,
}

impl<'a, 'b, R, K, V> Inlet<R, K, V> for HashMapInlet<'a, 'b, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static,
          'a: 'b
{
    fn push(&mut self, e: Emit<K, V>) {
        match e {
            Emit::None => (),
            Emit::One(k, v) => update_hashmap(&mut self.partial, self.parent.reducer, k, v),
            Emit::Vec(mut v) => {
                for p in v.drain(..) {
                    update_hashmap(&mut self.partial, self.parent.reducer, p.0, p.1)
                }
            }
        }
    }
}

impl<'a, 'b, R, K, V> Drop for HashMapInlet<'a, 'b, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static
{
    fn drop(&mut self) {
        let mut locked = self.parent.hashmap.lock().unwrap();
        for (k, v) in self.partial.drain() {
            update_hashmap(&mut locked, self.parent.reducer, k, v);
        }
    }
}

fn update_hashmap<'h, 'r, R, K, V>(hash: &'h mut HashMap<K, V>, reducer: &'r R, k: K, v: V)
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static
{
    let val = hash.entry(k);
    match val {
        Entry::Occupied(prev) => {
            let next = reducer(prev.get(), &v);
            *(prev.into_mut()) = next;
        }
        Entry::Vacant(vac) => {
            vac.insert(v);
        }
    }
}

pub struct MapReduceOp<'a, M, R, A, K, V>
    where M: Sync + Fn(A) -> Emit<K, V>,
          R: Sync + Fn(&V, &V) -> V + 'static,
          A: Send,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static
{
    mapper: M,
    reducer: R,
    _phantom: ::std::marker::PhantomData<A>,
    _phantom_2: ::std::marker::PhantomData<&'a usize>,
}

impl<'a, M, R, A, K, V> MapReduceOp<'a, M, R, A, K, V>
    where M: Sync + Fn(A) -> Emit<K, V>,
          R: Sync + Fn(&V, &V) -> V + 'static,
          A: Send,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static
{
    pub fn run(&self, chunks: BI<BI<A>>) -> HashMap<K, V> {
        let reducer = &self.reducer;
        let mapper = &self.mapper;
        println!("Mapping...");
        let mut rawpb = ProgressBar::new(chunks.size_hint().0);
        rawpb.format(" _🐌·🍀");
        let pb = Mutex::new(rawpb);

        let mut result: HashMap<K, V> = HashMap::new();
        {
            let aggregator = HashMapAggregator {
                hashmap: Mutex::new(&mut result),
                reducer: reducer,
            };

            let each = |it: BI<A>| {
                {
                    let mut inlet = aggregator.create_inlet();
                    for e in it.map(|e| mapper(e)) {
                        inlet.push(e)
                    }
                }
                pb.lock().unwrap().inc();
            };
            let mut pool = Pool::new(1 + ::num_cpus::get());
            unsafe {
                pool.map(chunks, &each).count();
            }

        }
        result
    }

    pub fn new_map_reduce(map: M, reduce: R) -> MapReduceOp<'a, M, R, A, K, V> {
        MapReduceOp {
            mapper: map,
            reducer: reduce,
            _phantom: ::std::marker::PhantomData,
            _phantom_2: ::std::marker::PhantomData,
        }
    }
}

pub fn map_reduce<'a, M, R, A, K, V>(map: M, reduce: R, chunks: BI<BI<A>>) -> HashMap<K, V>
    where M: Sync + Fn(A) -> Emit<K, V>,
          R: Sync + Fn(&V, &V) -> V + 'static,
          A: Send,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static
{
    MapReduceOp::new_map_reduce(map, reduce).run(chunks)
}


pub struct FilterCountOp<'a, M, A>
    where M: Sync + Fn(A) -> bool,
          A: Send
{
    mapper: M,
    _phantom: ::std::marker::PhantomData<A>,
    _phantom_2: ::std::marker::PhantomData<&'a usize>,
}

impl<'a, M, A> FilterCountOp<'a, M, A>
    where M: Sync + Fn(A) -> bool,
          A: Send
{
    pub fn run(&self, chunks: BI<BI<A>>) -> usize {
        let mapper = &self.mapper;
        let each = |it: BI<A>| -> usize {
            let mut aggregates = 0usize;
            for e in it {
                if mapper(e) {
                    aggregates += 1
                }
            }
            aggregates
        };
        let mut pool = Pool::new(16 /* + ::num_cpus::get() */);
        let halfway: usize = unsafe { pool.map(chunks, &each).sum() };
        halfway
    }

    pub fn new_filter_count(map: M) -> FilterCountOp<'a, M, A> {
        FilterCountOp {
            mapper: map,
            _phantom: ::std::marker::PhantomData,
            _phantom_2: ::std::marker::PhantomData,
        }
    }

    pub fn filter_count(map: M, chunks: BI<BI<A>>) -> usize {
        FilterCountOp::new_filter_count(map).run(chunks)
    }
}

pub fn par_foreach<A, F>(chunks: BI<BI<A>>, func: &F)
    where A: Send,
          F: Sync + Fn(A) -> ()
{

    let mapper = &func;
    let each = |it: BI<A>| -> () {
        it.map(|e| mapper(e)).count();
    };
    let mut pool = Pool::new(1 + ::num_cpus::get());
    let _: Vec<()> = unsafe { pool.map(chunks, &each).collect() };
}
