#[macro_use]
extern crate dazone;
extern crate capnp;
extern crate capdata as cap;
#[macro_use]
extern crate clap;
extern crate time;
extern crate num_cpus;
extern crate abomonation;
extern crate timely;
extern crate libc;

use dazone::crunch::*;
use dazone::short_bytes_array::*;
use dazone::rusage::*;

use timely::dataflow::*;
use timely::dataflow::operators::*;
use timely::dataflow::channels::pact::Exchange;

use capnp::serialize::OwnedSegments;
use capnp::message::Reader;

use std::hash::Hasher;
use std::collections::HashMap;

use std::sync::Arc;

fn main() {
    let app = clap_app!( query2 =>
                         (@arg SET: -s --set +takes_value "(tiny, 1node, 5nodes")
                         (@arg INPUT: -i --input +takes_value "(cap, rmp)")
                         (@arg CHUNKS: -c --chunks +takes_value "all")
                         (@arg KEY_LENGTH: -k --key-length +takes_value "(8, 9, 10, 11, 12)")
                         (@arg REDUCE: -r --reduce +takes_value "(hash, hashes, tries, timely)")
                         (@arg BUCKETS: -b --buckets +takes_value "reduce buckets (256)")
                         (@arg MONITOR: -m --monitor +takes_value "monitor resouce usage")
                         (@arg WORKERS: -w --workers +takes_value "worker threads (num_cpu*2)")
                         (@arg HOSTS: -h --hosts +takes_value "hosts, coma sep (for timely)")
                         (@arg ME: --me +takes_value "my position in hosts (starting at 0)")
                       );
    let matches = app.get_matches();

    let monitor: Option<Arc<Monitor>> = matches.value_of("MONITOR").map(|f| {
        Monitor::new(time::Duration::seconds(1),
                     ::std::fs::File::create(f).unwrap())
    });

    let runner = Runner {
        set: matches.value_of("SET").unwrap_or("5nodes").to_string(),
        input: matches.value_of("INPUT").unwrap_or("cap").to_string(),
        chunks: matches.value_of("CHUNKS").unwrap_or("999999999").parse().unwrap(),
        strategy: matches.value_of("REDUCE").unwrap_or("hashes").to_string(),
        workers: matches.value_of("WORKERS")
                        .map(|a| a.parse::<usize>().unwrap())
                        .unwrap_or(::num_cpus::get() * 2),
        buckets: matches.value_of("BUCKETS").unwrap_or("256").parse().unwrap(),
        hosts: matches.value_of("HOSTS").map(|x| x.to_string()),
        me: matches.value_of("ME").map(|x| x.parse().unwrap()),
        monitor: monitor,
    };

    println!("runner: {:?}", runner);

    let t1 = ::time::get_time();

    let length: usize = matches.value_of("LENGTH").unwrap_or("8").parse().unwrap();
    match length {
        8 => runner.clone().run::<Bytes8>(),
        9 => runner.clone().run::<Bytes9>(),
        10 => runner.clone().run::<Bytes10>(),
        11 => runner.clone().run::<Bytes11>(),
        12 => runner.clone().run::<Bytes12>(),
        _ => panic!("key length {} not implemented"),
    }
    let t2 = ::time::get_time();

    let usage = ::dazone::rusage::get_rusage();
    let vmsize = ::dazone::rusage::get_memory_usage().unwrap().virtual_size;
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
    // progress: bool,
    hosts: Option<String>,
    me: Option<usize>,
    monitor: Option<Arc<Monitor>>,
}

impl Runner {
    fn run<K>(self)
        where K: ShortBytesArray
    {
        if self.strategy == "timely" {
            self.run_timely::<K>();
        } else {
            self.run_standalone::<K>();
        }
    }

    fn sharded_input<'a, K>(&self, index: usize, peers: usize) -> BI<'a, BI<'a, (K, f32)>>
        where K: ShortBytesArray
    {
        if self.input.starts_with("cap") {
            Box::new(dazone::files::bibi_cap(&*self.set, "uservisits", &*self.input)
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
                             Box::new(chunk.map(move |reader: Reader<OwnedSegments>| {
                                 let visit: cap::user_visits::Reader = reader.get_root()
                                                                             .unwrap();
                                 (K::prefix(visit.get_source_i_p().unwrap()),
                                  visit.get_ad_revenue())
                             }))
                         }))
        } else if self.input.starts_with("pbuf") {
            Box::new(dazone::files::bibi_pbuf(&*self.set, "uservisits", &*self.input)
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
                             Box::new(chunk.map(move |visit: ::dazone::data::pbuf::UserVisits| {
                                 (K::prefix(visit.get_sourceIP()), visit.get_adRevenue())
                             }))
                         }))
        } else {
            Box::new(dazone::files::bibi_pod(&*self.set, "uservisits", &*self.input)
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
                             Box::new(chunk.map(move |visit: ::dazone::data::pod::UserVisits| {
                                 (K::prefix(&*visit.source_ip), visit.ad_revenue)
                             }))
                         }))
        }
    }

    fn run_standalone<K>(&self)
        where K: ShortBytesArray
    {
        let r = |a: &f32, b: &f32| a + b;
        let bibi = self.sharded_input::<K>(0, 1);
        let groups = match &*self.strategy {
            "hash" => {
                let mut aggregator = ::dazone::crunch::aggregators::HashMapAggregator::new(&r);
                MapOp::new_map_reduce(|(a, b)| Emit::One(a, b))
                    .with_monitor(self.monitor.clone())
                    .with_workers(self.workers)
                    .run(bibi, &mut aggregator);
                aggregator.converge();
                aggregator.len()
            }
            "hashes" => {
                let mut aggregator =
                        ::dazone::crunch::aggregators::MultiHashMapAggregator::with_hash_state(&r, self.buckets, dazone::crunch::fnv::FnvState);
                MapOp::new_map_reduce(|(a, b)| Emit::One(a, b))
                    .with_monitor(self.monitor.clone())
                    .with_workers(self.workers)
                    .run(bibi, &mut aggregator);
                aggregator.converge();
                aggregator.len()
            }
            "tries" => {
                let mut aggregator =
                    ::dazone::crunch::aggregators::MultiTrieAggregator::new(&r, self.buckets);
                MapOp::new_map_reduce(|(a, b): (K, f32)| Emit::One(a.to_vec(), b))
                    .with_monitor(self.monitor.clone())
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
        where K: ShortBytesArray
    {

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
                let position = self.me.unwrap_or_else(|| {
                    hosts.iter().position(|h| h == &*gethostname()).unwrap()
                });
                let hosts_with_ports: Vec<String> = hosts.iter()
                                                         .enumerate()
                                                         .map(|(index, host)| {
                                                             format!("{}:{}", host, 2101 + index)
                                                         })
                                                         .collect();
                println!("workers:{} hosts_with_ports:{:?} me:{}",
                         self.workers,
                         hosts_with_ports,
                         position);
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

            root.scoped::<u64, _, _>(move |builder| {

                    let uservisits = bibi.flat_map(|f| f).to_stream(builder);

                    let group_count =
                        uservisits.unary_notify(Exchange::new(move |x: &(K, f32)| {
                            ::dazone::hash(&(x.0)) as u64
                        }),
                        "groupby-map",
                        vec![],
                        move |input, output, notif| {
                            while let Some((time, data)) = input.next() {
                                notif.notify_at(time);
                                for (k, v) in data.drain(..) {
                                    dazone::crunch::aggregators::update_hashmap(&mut hashmap,
                                                                                &|a, b| {
                                                                                    a + b
                                                                                },
                                                                                k,
                                                                                v);
                                }
                            }
                            while let Some((iter, _)) = notif.next() {
                                if hashmap.len() > 0 {
                                    println!("worker {} map done, contributing ({})", index, hashmap.len());
                                    output.session(&iter).give(hashmap.len());
                                    hashmap.clear();
                                }
                            }
                        });

                    let _count: Stream<_, ()> = group_count.unary_notify(Exchange::new(|_| 0u64),
                    "count",
                    vec![],
                    move |input, _, notify| {
                        while let Some((time,
                                        data)) =
                            input.next() {
                                notify.notify_at(time);
                                println!("worker {} \
                                                                                       receiving \
                                                                                       {} counts ",
                                                                                       index,
                                                                                       data.len());
                                for x in
                                    data.drain(..) {
                                        sum += x;
                                    }
                            }
                        notify.for_each(|_, _| {
                            if sum > 0 {
                                println!("XXXX w\
                                                                                           orker:\
                                                                                           {} gro\
                                                                                           ups:{} \
                                                                                           XXXX",
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
