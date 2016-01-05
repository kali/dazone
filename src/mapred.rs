use std::collections::HashMap;
use std::collections::hash_map::Entry;

use simple_parallel::pool::Pool;

pub type BI<'a,A> = Box<Iterator<Item=A> + Send + 'a>;

pub struct MapReduceOp<'a, M, R, A, K, V>
    where M: Sync + Fn(A) -> BI<'a, (K, V)>,
          R: Sync + Fn(&V, &V) -> V,
          A: Send,
          K: Send + Eq + ::std::hash::Hash + Clone,
          V: Clone + Send
{
    mapper: M,
    reducer: R,
    _phantom: ::std::marker::PhantomData<A>,
    _phantom_2: ::std::marker::PhantomData<&'a usize>,
}

impl <'a,M,R,A,K,V> MapReduceOp<'a,M,R,A,K,V>
    where   M:Sync + Fn(A) -> BI<'a,(K,V)>,
            R:Sync + Fn(&V,&V) -> V,
            A:Send,
            K:Send + Eq + ::std::hash::Hash + Clone,
            V:Clone+Send
{
    pub fn run(&self, chunks: BI<BI<A>>) -> HashMap<K, V> {
        let reducer = &self.reducer;
        let mapper = &self.mapper;
        let each = |it: BI<A>| -> HashMap<K, V> {
            let mut aggregates: HashMap<K, V> = HashMap::new();
            for (k, v) in it.flat_map(|e| mapper(e)) {
                let val = aggregates.entry(k.clone());
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
            aggregates
        };
        let mut pool = Pool::new(1 + ::num_cpus::get());
        let halfway: Vec<HashMap<K, V>> = unsafe { pool.map(chunks, &each).collect() };
        let mut result: HashMap<K, V> = HashMap::new();
        for h in halfway.iter() {
            for (k, v) in h.iter() {
                let val = result.entry(k.clone());
                match val {
                    Entry::Occupied(prev) => {
                        let next = reducer(prev.get(), v);
                        *(prev.into_mut()) = next.clone();
                    }
                    Entry::Vacant(vac) => {
                        vac.insert(v.clone());
                    }
                }
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

    pub fn map_reduce(map: M, reduce: R, chunks: BI<BI<A>>) -> HashMap<K, V> {
        MapReduceOp::new_map_reduce(map, reduce).run(chunks)
    }

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
