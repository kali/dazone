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

// ************************** AGGREGATOR / INLET *****************************

pub trait Aggregator<R,K,V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
        K: Send + Eq + ::std::hash::Hash + 'static,
        V: Send + 'static
{
    fn create_inlet<'b>(&'b self, i: usize) -> Box<Inlet<R, K, V> + 'b>;
    fn converge(&mut self) {}
}

pub trait Inlet<R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static
{
    fn push_one(&mut self, k: K, v: V);

    fn push(&mut self, e: Emit<K, V>) {
        match e {
            Emit::None => (),
            Emit::One(k, v) => self.push_one(k, v),
            Emit::Vec(mut v) => {
                for p in v.drain(..) {
                    self.push_one(p.0, p.1)
                }
            }
        }
    }
}

// ************************** HashMap Aggregator *****************************

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
    fn create_inlet<'b>(&'b self, i: usize) -> Box<Inlet<R, K, V> + 'b> {
        Box::new(HashMapInlet {
            parent: &self,
            partial: HashMap::new(),
            i: i,
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
    #[allow(dead_code)]
    i: usize,
}

impl<'a, 'b, R, K, V> Inlet<R, K, V> for HashMapInlet<'a, 'b, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static,
          'a: 'b
{
    fn push_one(&mut self, k: K, v: V) {
        update_hashmap(&mut self.partial, self.parent.reducer, k, v);
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

// ************************** MultiHashMap Aggregator *************************

trait MultiAggregator<'a, R,K,V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static {
    fn partition(&self, k: &K) -> usize;
    fn merge<'b>(&self, inlet: &mut MultiHashMapInlet<'a, 'b, R, K, V>);
}

struct MultiHashMapAggregator<'a, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static
{
    hashmaps: Vec<Mutex<&'a mut HashMap<K, V>>>,
    reducer: &'a R,
}

impl<'a, R, K, V> MultiAggregator<'a, R, K, V> for MultiHashMapAggregator<'a, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static
{
    fn merge<'b>(&self, inlet: &mut MultiHashMapInlet<'a, 'b, R, K, V>) {
        for (i, mut partial) in inlet.hashmaps.drain(..).enumerate() {
            let len = partial.len();
            let t1 = ::time::get_time();
            let mut locked = self.hashmaps[i].lock().unwrap();
            let t2 = ::time::get_time();
            for (k, v) in partial.drain() {
                update_hashmap(&mut locked, self.reducer, k, v);
            }
            let t3 = ::time::get_time();
            if t2 - t1 > ::time::Duration::milliseconds(100) {
                println!("{:4} wait {}ms ({})", i, (t2 - t1).num_milliseconds(), len);
            }
            if t3 - t2 > ::time::Duration::milliseconds(100) {
                println!("{:4} took {}ms ({})", i, (t3 - t2).num_milliseconds(), len);
            }
        }
    }

    fn partition(&self, k: &K) -> usize {
        use std::hash::{Hasher, SipHasher};
        let mut s = SipHasher::new();
        k.hash(&mut s);
        s.finish() as usize % self.hashmaps.len()
    }
}

impl<'a, R, K, V> Aggregator<R, K, V> for MultiHashMapAggregator<'a, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static
{
    fn create_inlet<'b>(&'b self, i: usize) -> Box<Inlet<R, K, V> + 'b> {
        let mut v = Vec::with_capacity(self.hashmaps.len());
        for _ in 0..self.hashmaps.len() {
            v.push(HashMap::new())
        }
        Box::new(MultiHashMapInlet {
            parent: self,
            hashmaps: v,
            reducer: self.reducer,
            i: i,
        })
    }
}


// ************************** MultiHashMap Inlet *************************

struct MultiHashMapInlet<'a, 'b, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static,
          'a: 'b
{
    parent: &'b MultiAggregator<'a, R, K, V>,
    hashmaps: Vec<HashMap<K, V>>,
    reducer: &'a R,
    #[allow(dead_code)]
    i: usize,
}

impl<'a, 'b, R, K, V> Inlet<R, K, V> for MultiHashMapInlet<'a, 'b, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static,
          'a: 'b
{
    fn push_one(&mut self, k: K, v: V) {
        let partial: usize = self.parent.partition(&k);
        update_hashmap(&mut self.hashmaps[partial], self.reducer, k, v)
    }
}

impl<'a, 'b, R, V, K> Drop for MultiHashMapInlet<'a, 'b, R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static,
          'a: 'b
{
    fn drop(&mut self) {
        self.parent.merge(self)
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

#[allow(dead_code)]
fn dump_vec_map<K, V>(i: usize, hashes: &Vec<HashMap<K, V>>)
    where K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static
{
    let lens: Vec<usize> = hashes.iter().map(|h| h.len()).collect();
    let sum: usize = lens.iter().sum();
    println!("{:4} min:{:5} max:{:5} avg:{:5}",
             i,
             lens.iter().min().unwrap(),
             lens.iter().max().unwrap(),
             sum / lens.len());
}

pub struct MapOp<'a, M, A, K, V>
    where M: Sync + Fn(A) -> Emit<K, V>,
          A: Send,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static
{
    mapper: M,
    _phantom: ::std::marker::PhantomData<A>,
    _phantom_2: ::std::marker::PhantomData<&'a usize>,
}

impl<'a, M, A, K, V> MapOp<'a, M, A, K, V>
    where M: Sync + Fn(A) -> Emit<K, V>,
          A: Send,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static
{
    pub fn run<Agg, R>(&self, chunks: BI<BI<A>>, aggregator: &Agg)
        where Agg: Aggregator<R, K, V> + Sync,
              R: Sync + Fn(&V, &V) -> V + 'static
    {
        let mapper = &self.mapper;
        println!("Mapping...");
        let mut rawpb = ProgressBar::new(chunks.size_hint().0);
        rawpb.format(" _🐌·🍀");
        let pb = Mutex::new(rawpb);

        {
            let each = |(i, it): (usize, BI<A>)| {
                {
                    let mut inlet = aggregator.create_inlet(i);
                    for e in it.map(|e| mapper(e)) {
                        inlet.push(e)
                    }
                }
                pb.lock().unwrap().inc();
            };
            let mut pool = Pool::new(1 + ::num_cpus::get());
            unsafe {
                pool.map(chunks.enumerate(), &each).count();
            }

        }
    }

    pub fn new_map_reduce(map: M) -> MapOp<'a, M, A, K, V> {
        MapOp {
            mapper: map,
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
    let mut result: HashMap<K, V> = HashMap::new();
    {
        let mut aggregator = HashMapAggregator {
            hashmap: Mutex::new(&mut result),
            reducer: &reduce,
        };
        MapOp::new_map_reduce(map).run(chunks, &aggregator);
        aggregator.converge()
    }
    result
}

pub fn map_par_reduce<'a, M, R, A, K, V>(map: M,
                                         reduce: R,
                                         partitions: usize,
                                         chunks: BI<BI<A>>)
                                         -> Vec<HashMap<K, V>>
    where M: Sync + Fn(A) -> Emit<K, V>,
          R: Sync + Fn(&V, &V) -> V + 'static,
          A: Send,
          K: Send + Eq + ::std::hash::Hash + 'static,
          V: Send + 'static
{
    let mut result: Vec<HashMap<K, V>> = (0..partitions).map(|_| HashMap::new()).collect();
    {
        let mut aggregator = MultiHashMapAggregator {
            hashmaps: result.iter_mut().map(|h| Mutex::new(h)).collect(),
            reducer: &reduce,
        };
        MapOp::new_map_reduce(map).run(chunks, &aggregator);
        aggregator.converge()
    }
    result
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
