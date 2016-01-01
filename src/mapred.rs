use simple_parallel::pool::Pool;

pub type BI<'a,A> = Box<Iterator<Item=A> + Send + 'a>;

pub struct MapReduceOp<'a,M,A>
    where   M:Sync + Fn(A) -> bool,
            A:Send
{
    mapper: M,
    _phantom: ::std::marker::PhantomData<A>,
    _phantom_2: ::std::marker::PhantomData<&'a usize>,
}

impl <'a,M,A> MapReduceOp<'a,M,A>
    where   M:Sync + Fn(A) -> bool,
            A:Send
{
    pub fn run(&self, chunks:BI<BI<A>>) -> usize {
        let mapper = &self.mapper;
        let each = |it: BI<A>| -> usize {
            let mut aggregates = 0usize;
            for e in it {
                if mapper(e) { aggregates += 1 }
            };
            aggregates
        };
        let mut pool = Pool::new(16 /* + ::num_cpus::get() */);
        let halfway:usize = unsafe { pool.map(chunks, &each).sum() };
        halfway
    }

    pub fn new_map_reduce(map:M) -> MapReduceOp<'a,M,A> {
        MapReduceOp {
            mapper: map,
            _phantom: ::std::marker::PhantomData,
            _phantom_2: ::std::marker::PhantomData
        }
    }

    pub fn map_reduce(map:M, chunks:BI<BI<A>>) -> usize {
        MapReduceOp::new_map_reduce(map).run(chunks)
    }

}

pub fn par_foreach<A,F>(chunks:BI<BI<A>>, func:&F)
    where A:Send, F: Sync + Fn(A) -> () {

    let mapper = &func;
    let each = |it:BI<A>| -> () {
        it.map(|e| { mapper(e) }).count();
    };
    let mut pool = Pool::new(1 + ::num_cpus::get());
    let _:Vec<()> = unsafe { pool.map(chunks, &each).collect() };
}
