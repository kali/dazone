#![feature(reflect_marker)]
#![feature(cstring_into)]

extern crate dx16;
extern crate capnp;
#[macro_use]
extern crate clap;
extern crate time;
extern crate num_cpus;
extern crate abomonation;
extern crate timely;
extern crate libc;

use dx16::Dx16Result;
use dx16::mapred::*;

use timely::dataflow::*;
use timely::dataflow::operators::*;
use timely::progress::timestamp::RootTimestamp;
use timely::dataflow::channels::pact::Exchange;

use capnp::serialize::OwnedSegments;
use capnp::message::Reader;

use std::hash::{Hash, Hasher};
use std::collections::HashMap;

use abomonation::Abomonation;

use std::fmt::Debug;

trait FixedBytesArray {
    fn prefix(s: &str) -> Self;
    fn to_vec(&self) -> Vec<u8>;
}

macro_rules! fixed_bytes_array {
    ( $name:ident $size:expr ) => {
        #[derive(Copy,Clone,Debug,PartialEq,Eq)]
        pub struct $name([u8;$size]);

        impl FixedBytesArray for $name {
            fn prefix(s:&str) -> $name {
                let mut buf = [b' ';$size];
                {
                    use std::io::Write;
                    let mut slice:&mut [u8] = &mut buf;
                    let bytes = s.as_bytes();
                    let len = ::std::cmp::min(bytes.len(), $size);
                    slice.write_all(&bytes[0..len]).unwrap();
                }
                $name(buf)
            }

            fn to_vec(&self) -> Vec<u8> {
                self.0.to_vec()
            }
        }

        impl Abomonation for $name { }

        impl Hash for $name {
            fn hash<H>(&self, state: &mut H) where H: Hasher {
                self.0.hash(state)
            }
        }
    }
}

fixed_bytes_array!( K8 8 );
fixed_bytes_array!( K9 9 );
fixed_bytes_array!( K10 10 );
fixed_bytes_array!( K11 11 );
fixed_bytes_array!( K12 12 );

fn main() {
    let app = clap_app!( query2 =>
                         (@arg SET: -s --set +takes_value "(tiny, 1node, 5nodes")
                         (@arg INPUT: -i --input +takes_value "(cap, rmp)")
                         (@arg CHUNKS: -c --chunks +takes_value "all")
                         (@arg KEY_LENGTH: -k --key-length +takes_value "(8, 9, 10, 11, 12)")
                         (@arg REDUCE: -r --reduce +takes_value "(hash, hashes, tries, timely)")
                         (@arg BUCKETS: -b --buckets +takes_value "reduce buckets (256)")
                         (@arg PROGRESS: -p --progress "show progressbar")
                         (@arg MEMORY: -m --memory "monitor memory usage")
                         (@arg WORKERS: -w --workers +takes_value "worker threads (num_cpu*2)")
                         (@arg HOSTS: -h --hosts +takes_value "hosts, coma sep (for timely)")
                         (@arg ME: --me +takes_value "my position in hosts (starting at 0)")
                       );
    let matches = app.get_matches();

    let runner = Runner {
        set: matches.value_of("SET").unwrap_or("5nodes").to_string(),
        input: matches.value_of("INPUT").unwrap_or("cap").to_string(),
        chunks: matches.value_of("CHUNKS").unwrap_or("999999999").parse().unwrap(),
        strategy: matches.value_of("REDUCE").unwrap_or("hashes").to_string(),
        workers: matches.value_of("WORKERS")
                        .map(|a| a.parse::<usize>().unwrap())
                        .unwrap_or(::num_cpus::get() * 2),
        buckets: matches.value_of("BUCKETS").unwrap_or("256").parse().unwrap(),
        progress: matches.is_present("PROGRESS"),
        hosts: matches.value_of("HOSTS").map(|x| x.to_string()),
        me: matches.value_of("ME").map(|x| x.parse().unwrap())
    };

    println!("runner: {:?}", runner);

    let t1 = ::time::get_time();

    if matches.is_present("MEMORY") {
        ::dx16::rusage::start_monitor(::std::time::Duration::from_secs(10));
    }

    let length: usize = matches.value_of("LENGTH").unwrap_or("8").parse().unwrap();
    match length {
        8 => runner.clone().run::<K8>(),
        9 => runner.clone().run::<K9>(),
        10 => runner.clone().run::<K10>(),
        11 => runner.clone().run::<K11>(),
        12 => runner.clone().run::<K12>(),
        _ => panic!("key length {} not implemented"),
    }
    let t2 = ::time::get_time();

    let usage = ::dx16::rusage::get_rusage();
    let vmsize = ::dx16::rusage::get_memory_usage().unwrap().virtual_size;
    println!("set: {:6} chunks: {:4} length: {:2} strat: {:6} buckets: {:4} workers: {:4} \
              rss_mb: {:5} vmmsize_mb: {:5} utime_s: {:5} stime_s: {:5} ctime_s: {:5}",
             &*runner.set,
             runner.chunks,
             length,
             &*runner.strategy,
             runner.buckets,
             runner.workers,
             usage.ru_maxrss / 1024 / 1024,
             vmsize / 1024 / 1024,
             usage.ru_utime.tv_sec,
             usage.ru_stime.tv_sec,
             (t2 - t1).num_seconds());

}

#[derive(Clone,Debug)]
struct Runner {
    set: String,
    input: String,
    chunks: usize,
    strategy: String,
    workers: usize,
    buckets: usize,
    progress: bool,
    hosts: Option<String>,
    me: Option<usize>,
}

impl Runner {
    fn run<K>(self)
        where K:FixedBytesArray+Abomonation+Send+Clone+::std::marker::Reflect+::std::hash::Hash+Eq+'static+Debug {
        if self.strategy == "timely" {
            self.run_timely::<K>();
        } else {
            self.run_standalone::<K>();
        }
    }

    fn sharded_input<'a, K>(&self, index:usize, peers:usize) -> BI<'a, BI<'a, (K,f32)>>
        where K:FixedBytesArray+Abomonation+Send+Clone+::std::marker::Reflect+::std::hash::Hash+Eq+'static+Debug {
        match &*self.input {
            "cap" | "cap-gz" => {
                Box::new(if &*self.input == "cap-gz" {
                             dx16::bibi_cap_gz_dec(&*self.set, "uservisits")
                         } else {
                             dx16::bibi_cap_dec(&*self.set, "uservisits")
                         }
                         .take(self.chunks)
                         .enumerate()
                         .filter_map(move |(i, f)| {
                             if i % peers == index {
                                 Some(f)
                             } else {
                                 None
                             }
                         })
                         .map(move |chunk| -> BI<(K, f32)> {
                             Box::new(chunk.map(move |reader: Dx16Result<Reader<OwnedSegments>>| {
                                 let reader = reader.unwrap();
                                 let visit: ::dx16::cap::user_visits::Reader = reader.get_root()
                                                                                     .unwrap();
                                 (K::prefix(visit.get_source_i_p().unwrap()),
                                  visit.get_ad_revenue())
                             }))
                         }))
            }
            "rmp" => {
                    Box::new(dx16::rmp_read::bibi_rmp_gz_dec(&*self.set, "uservisits").take(self.chunks)
                             .enumerate()
                             .filter_map(move |(i,f)| if i % peers == index { Some(f) } else { None })
                             .map(move |chunk| -> BI<(K, f32)> {
                                 Box::new(chunk.map(move |reader: Dx16Result<::dx16::data::UserVisits>| {
                                     let visit = reader.unwrap();
                                     (K::prefix(&*visit.source_ip), visit.ad_revenue)
                                 }))
                             }))
                }
            any => panic!("unknown input format {}", any),
        }
    }

    fn run_standalone<K>(&self)
        where K:FixedBytesArray+Abomonation+Send+Clone+::std::marker::Reflect+::std::hash::Hash+Eq+'static+Debug {
        let r = |a: &f32, b: &f32| a + b;
        let bibi = self.sharded_input::<K>(0, 1);
        let groups = match &*self.strategy {
            "hash" => {
                let mut aggregator = ::dx16::aggregators::HashMapAggregator::new(&r);
                MapOp::new_map_reduce(|(a, b)| Emit::One(a, b))
                    .with_progress(self.progress)
                    .with_workers(self.workers)
                    .run(bibi, &mut aggregator);
                aggregator.converge();
                aggregator.len()
            }
            "hashes" => {
                let mut aggregator = ::dx16::aggregators::MultiHashMapAggregator::new(&r,
                                                                                      self.buckets);
                MapOp::new_map_reduce(|(a, b)| Emit::One(a, b))
                    .with_progress(self.progress)
                    .with_workers(self.workers)
                    .run(bibi, &mut aggregator);
                aggregator.converge();
                aggregator.len()
            }
            "tries" => {
                let mut aggregator = ::dx16::aggregators::MultiTrieAggregator::new(&r,
                                                                                   self.buckets);
                MapOp::new_map_reduce(|(a, b): (K, f32)| Emit::One(a.to_vec(), b))
                    .with_progress(self.progress)
                    .with_workers(self.workers)
                    .run(bibi, &mut aggregator);
                aggregator.converge();
                aggregator.len()
            }
            s => panic!("unknown strategy {}", s),
        };
        println!("groups: {}", groups);
    }

    fn run_timely<K>(self)
        where K:FixedBytesArray+Abomonation+Send+Clone+::std::marker::Reflect+::std::hash::Hash+Eq+'static+Debug {

        fn gethostname() -> String {
            let host: String = unsafe {
                let mut buf = [0i8; 1024];
                let _err = ::libc::gethostname(::std::mem::transmute(&mut buf), buf.len());
                let cstr = ::std::ffi::CStr::from_ptr(buf.as_ptr());
                cstr.to_str().unwrap().to_owned()
            };
            host.split(".").next().unwrap().to_owned()
        }

        let conf: ::timely::Configuration = match self.hosts.as_ref() {
            Some(hosts) => {
                let hosts: Vec<String> = hosts.split(",").map(|x| x.to_owned()).collect();
                let position = self.me.unwrap_or_else(|| hosts.iter().position(|h| h == &*gethostname()).unwrap());
                let hosts_with_ports: Vec<String> = hosts.iter()
                                                         .enumerate()
                                                         .map(|(index, host)| {
                                                             format!("{}:{}", host, 2101 + index)
                                                         })
                                                         .collect();
                println!("workers:{} hosts_with_ports:{:?} me:{}", self.workers, hosts_with_ports, position);
                ::timely::Configuration::Cluster(self.workers, position, hosts_with_ports, false)
            }
            None => ::timely::Configuration::Process(self.workers),
        };

        timely::execute(conf, move |root| {
            let mut hashmap = HashMap::new();
            let mut sum = 0usize;
            let index = root.index();
            let peers = root.peers();
            let bibi = self.sharded_input::<K>(index, peers);
            println!("worker:{}/{} start", index, peers);

            root.scoped(move |builder| {

                let uservisits = bibi.flat_map(|f| f).to_stream(builder);

                let group_count =
                        uservisits.unary_notify(Exchange::new(move |x: &(K, f32)| {
                            ::dx16::hash(&(x.0)) as u64
                        }),
                        "groupby-map",
                        vec![RootTimestamp::new(0)],
                        move |input, output, notif| {
                            notif.notify_at(&RootTimestamp::new(0));
                            while let Some((_, data)) = input.next() {
                                for (k, v) in data.drain(..) {
                                    dx16::aggregators::update_hashmap(&mut hashmap,
                                                                      &|a, b| {
                                                                          a + b
                                                                      },
                                                                      k,
                                                                      v);
                                }
                            }
                            while let Some((iter, _)) = notif.next() {
                                if hashmap.len() > 0 {
                                    output.session(&iter).give(hashmap.len());
                                    hashmap.clear();
                                }
                            }
                        });

                let _count: Stream<_, ()> =
                    group_count.unary_notify(Exchange::new(|_| 0u64),
                                             "count",
                                             vec![RootTimestamp::new(0)],
                                             move |input, _, notify| {
                                                 notify.notify_at(&RootTimestamp::new(0));
                                                 while let Some((_, data)) = input.next() {
                                                     for x in data.drain(..) {
                                                         sum += x;
                                                     }
                                                 }
                                                 notify.for_each(|_, _| {
                                                     if sum > 0 {
                                                         println!("worker:{} groups:{}",
                                                                  index,
                                                                  sum);
                                                         sum = 0;
                                                     }
                                                 })
                                             });

            });

            while root.step() {
            }
        });
    }
}
