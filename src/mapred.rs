use std::collections::HashMap;
use std::collections::hash_map::Entry;

use simple_parallel::pool::Pool;

use std::sync::Mutex;

use pbr::ProgressBar;

pub type BI<'a,A> = Box<Iterator<Item=A> + Send + 'a>;

//
// trait Accumulator<R,K,V>
// where R: Sync + Fn(&V, &V) -> V,
// K: Send+ 'static,
// V: Send + 'static {
// type Shard:PartialAccumulator<R,K,V>;
//
// fn make_shard<'a>(&mut self) -> &'a mut Self::Shard;
// fn finalize(&mut self);
// }
//
// trait PartialAccumulator<R,K,V>
// where R: Sync + Fn(&V, &V) -> V,
// K: Send +'static,
// V: Send+ 'static {
// fn offer(&mut self, k: K, v: V);
// }
//
// struct VecHashMapAccu<'a, R, K, V>
// where R: Sync + Fn(&V, &V) -> V,
// K: Send + 'static,
// V: Send + 'static
// {
// shards: Vec<HashMap<K, V>>,
// reducer: R,
// phantom: ::std::marker::PhantomData<&'a usize>,
// }
//
// struct HashMapAccu<'a, R, K, V>
// where R: Sync + Fn(&V, &V) -> V,
// K: Send + 'static,
// V: Send + 'static
// {
// hash: &'a HashMap<K, V>,
// reducer: R,
// }
//
// impl<'a, R,K,V> Accumulator< R,K,V> for VecHashMapAccu<'a, R,K,V>
// where R: Sync + Fn(&V, &V) -> V,
// K: Send + Eq + ::std::hash::Hash + 'static,
// V: Send + 'static {
// type Shard = HashMapAccu<'a, R,K,V>;
//
// fn make_shard(&mut self) -> &'a mut Self::Shard {
// let shard = HashMapAccu {
// hash: HashMap::new(),
// reducer: self.reducer,
// };
// self.shards.push(shard);
// &mut shard
// }
// fn finalize(&mut self) {
// }
// }
//
// impl<'a, R,K,V> PartialAccumulator<R,K,V> for HashMapAccu<'a, R,K,V>
// where R: Sync + Fn(&V, &V) -> V,
// K: Send + Eq + ::std::hash::Hash + 'static,
// V: Send + 'static {
// fn offer(&mut self, k: K, v: V) {
// let reducer = &self.reducer;
// let val = self.hash.entry(k);
// match val {
// Entry::Occupied(prev) => {
// let next = reducer(prev.get(), &v);
// (prev.into_mut()) = next;
// }
// Entry::Vacant(vac) => {
// vac.insert(v);
// }
// }
// }
//
// }
//

pub enum Emit<K, V> {
    None,
    One(K, V),
    Vec(Vec<(K, V)>),
}

pub struct MapReduceOp<'a, M, R, A, K, V>
    where M: Sync + Fn(A) -> Emit<K, V>,
          R: Sync + Fn(&V, &V) -> V,
          A: Send,
          K: Send + Eq + ::std::hash::Hash,
          V: Send
{
    mapper: M,
    reducer: R,
    _phantom: ::std::marker::PhantomData<A>,
    _phantom_2: ::std::marker::PhantomData<&'a usize>,
}

impl <'a,M,R,A,K,V> MapReduceOp<'a,M,R,A,K,V>
    where   M:Sync + Fn(A) -> Emit<K,V>,
            R:Sync + Fn(&V,&V) -> V,
            A:Send,
            K:Send + Eq + ::std::hash::Hash,
            V:Send
{
    fn update_hashmap(hash: &mut HashMap<K, V>, reducer: &R, k: K, v: V) {
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


    pub fn run(&self, chunks: BI<BI<A>>) -> HashMap<K, V> {
        let reducer = &self.reducer;
        let mapper = &self.mapper;
        println!("Mapping...");
        let mut rawpb = ProgressBar::new(chunks.size_hint().0);
        rawpb.format(" _🐌·🍀");
        let pb = Mutex::new(rawpb);
        let each = |it: BI<A>| -> HashMap<K, V> {
            let mut aggregates: HashMap<K, V> = HashMap::new();
            {
                for em in it.map(|e| mapper(e)) {
                    match em {
                        Emit::None => (),
                        Emit::One(k, v) => Self::update_hashmap(&mut aggregates, reducer, k, v),
                        Emit::Vec(mut v) => for p in v.drain(..) {
                            Self::update_hashmap(&mut aggregates, reducer, p.0, p.1)
                        },
                    }
                }
            }
            pb.lock().unwrap().inc();
            aggregates
        };
        let mut pool = Pool::new(1 + ::num_cpus::get());
        let mut halfway: Vec<HashMap<K, V>> = unsafe { pool.map(chunks, &each).collect() };
        println!("\nReducing...");
        let mut rawpb = ProgressBar::new(halfway.len());
        rawpb.format(" _🐌·🍀");
        let mut result: HashMap<K, V> = HashMap::new();
        for mut h in halfway.drain(..) {
            for (k, v) in h.drain() {
                Self::update_hashmap(&mut result, reducer, k, v);
            }
            rawpb.inc();
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
          R: Sync + Fn(&V, &V) -> V,
          A: Send,
          K: Send + Eq + ::std::hash::Hash,
          V: Send
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

impl <'a,M,A> FilterCountOp<'a,M,A>
    where   M:Sync + Fn(A) -> bool,
            A:Send
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
