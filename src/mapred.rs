use std::collections::HashMap;
use std::collections::hash_map::Entry;

use simple_parallel::pool::Pool;

use std::sync::Mutex;

use pbr::ProgressBar;

pub type BI<'a,A> = Box<Iterator<Item=A> + Send + 'a>;

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
    pub fn run(&self, chunks: BI<BI<A>>) -> HashMap<K, V> {
        let reducer = &self.reducer;
        let mapper = &self.mapper;
        let pb = Mutex::new(ProgressBar::new(chunks.size_hint().0));
        let each = |it: BI<A>| -> HashMap<K, V> {
            let mut aggregates: HashMap<K, V> = HashMap::new();
            {
                let mut use_pair = |k: K, v: V| {
                    let val = aggregates.entry(k);
                    match val {
                        Entry::Occupied(prev) => {
                            let next = reducer(prev.get(), &v);
                            *(prev.into_mut()) = next;
                        }
                        Entry::Vacant(vac) => {
                            vac.insert(v);
                        }
                    }
                };
                for em in it.map(|e| mapper(e)) {
                    match em {
                        Emit::None => (),
                        Emit::One(k, v) => use_pair(k, v),
                        Emit::Vec(mut v) => for p in v.drain(..) {
                            use_pair(p.0, p.1)
                        },
                    }
                }
            }
            pb.lock().unwrap().inc();
            aggregates
        };
        let mut pool = Pool::new(1 + ::num_cpus::get());
        let mut halfway: Vec<HashMap<K, V>> = unsafe { pool.map(chunks, &each).collect() };
        let mut result: HashMap<K, V> = HashMap::new();
        for mut h in halfway.drain(..) {
            for (k, v) in h.drain() {
                let val = result.entry(k);
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
