use std::collections::HashMap;
use std::sync::Mutex;

use std::hash::Hash;

use simple_parallel::pool::Pool;


use pbr::ProgressBar;


pub type BI<'a, A> = Box<Iterator<Item = A> + Send + 'a>;

pub enum Emit<K, V> {
    None,
    One(K, V),
    Vec(Vec<(K, V)>),
}

// ************************** AGGREGATOR / INLET *****************************

pub trait Aggregator<'a, R,K,V> : Sync
    where R: Sync + Fn(&V, &V) -> V + 'static,
        K: Send + Eq + Hash + 'static,
        V: Send + 'static
{
    fn create_inlet<'b>(&'b self, i: usize) -> Box<Inlet<R, K, V> + 'b>;
    fn converge(&mut self) {}
    fn len(&self) -> u64;
}

pub trait Inlet<R, K, V>
    where R: Sync + Fn(&V, &V) -> V + 'static,
          K: Send + Eq + Hash + 'static,
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

pub struct MapOp<'a, M, A, K, V>
    where M: Sync + Fn(A) -> Emit<K, V>,
          A: Send,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    mapper: M,
    _phantom: ::std::marker::PhantomData<A>,
    _phantom_2: ::std::marker::PhantomData<&'a usize>,
    workers: usize,
    progressbar: bool,
}

impl<'a, M, A, K, V> MapOp<'a, M, A, K, V>
    where M: Sync + Fn(A) -> Emit<K, V>,
          A: Send,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    pub fn run<Agg, R>(&self, chunks: BI<BI<A>>, aggregator: &Agg)
        where Agg: Aggregator<'a, R, K, V>,
              R: Sync + Fn(&V, &V) -> V + 'static
    {
        let progressbar = self.progressbar;
        let mapper = &self.mapper;

        let rawpb = ProgressBar::new(chunks.size_hint().0);
        // rawpb.format(" _🐌·🍀");
        let pb = Mutex::new(rawpb);

        {
            let each = |(i, it): (usize, BI<A>)| {
                {
                    let mut inlet = aggregator.create_inlet(i);
                    for e in it.map(|e| mapper(e)) {
                        inlet.push(e)
                    }
                }
                if progressbar {
                    pb.lock().unwrap().inc();
                }
            };
            let mut pool = Pool::new(self.workers);
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
            workers: ::num_cpus::get() * 2,
            progressbar: false,
        }
    }

    pub fn with_workers(self, w: usize) -> MapOp<'a, M, A, K, V> {
        MapOp { workers: w, ..self }
    }

    pub fn with_progress(self, p: bool) -> MapOp<'a, M, A, K, V> {
        MapOp { progressbar: p, ..self }
    }
}

pub fn map_reduce<'a, M, R, A, K, V>(map: M, reduce: R, chunks: BI<BI<A>>) -> HashMap<K, V>
    where M: Sync + Fn(A) -> Emit<K, V>,
          R: Sync + Fn(&V, &V) -> V + 'static,
          A: Send,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    let mut aggregator = ::aggregators::HashMapAggregator::new(&reduce);
    MapOp::new_map_reduce(map).run(chunks, &aggregator);
    aggregator.converge();
    aggregator.as_inner()
}

pub fn map_par_reduce<'a, M, R, A, K, V>(map: M,
                                         reduce: R,
                                         partitions: usize,
                                         chunks: BI<BI<A>>)
                                         -> Vec<HashMap<K, V>>
    where M: Sync + Fn(A) -> Emit<K, V>,
          R: Sync + Fn(&V, &V) -> V + 'static,
          A: Send,
          K: Send + Eq + Hash + 'static,
          V: Send + 'static
{
    let mut aggregator = ::aggregators::MultiHashMapAggregator::new(&reduce, partitions);
    MapOp::new_map_reduce(map).run(chunks, &aggregator);
    aggregator.converge();
    aggregator.as_inner()
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
