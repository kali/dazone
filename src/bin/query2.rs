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

use dazone::crunch::aggregators::update_hashmap;

use std::hash::Hasher;
use std::collections::HashMap;

use std::sync::Arc;

fn main() {
    let app = clap_app!( query2 =>
                         (@arg SET: -s --set +takes_value "(tiny, 1node, 5nodes")
                         (@arg INPUT: -i --input +takes_value "(cap, rmp)")
                         (@arg CHUNKS: -c --chunks +takes_value "all")
                         (@arg KEY_LENGTH: -k --key_length +takes_value "(8, 9, 10, 11, 12)")
                         (@arg REDUCE: -r --reduce +takes_value "(hash, hashes, tries, timely)")
                         (@arg SIP: --sip +takes_value "(use SIP hasher)")
                         (@arg BUCKETS: -b --buckets +takes_value "reduce buckets (256)")
                         (@arg PARTIAL: -p --partial "activate partial aggregation")
                         (@arg MONITOR: -m --monitor +takes_value "monitor resouce usage")
                         (@arg WORKERS: -w --workers +takes_value "worker threads (num_cpu*2)")
                         (@arg HOSTS: -h --hosts +takes_value "hosts, coma sep (for timely)")
                         (@arg ME: --me +takes_value "my position in hosts (starting at 0)")
                         (@arg PROGRESS_BAR: --pb "show progress bar")
                       );
    let matches = app.get_matches();


    let set = matches.value_of("SET").unwrap_or("5nodes").to_string();
    let input = matches.value_of("INPUT").unwrap_or("cap").to_string();

    let monitor: Option<Arc<Monitor>> = matches.value_of("MONITOR").map(|f| {
        Monitor::new(time::Duration::seconds(1),
                     ::std::fs::File::create(f).unwrap(),
                     ::dazone::files::files_for_format(&*set, "uservisits", &*input)
                         .size_hint()
                         .1
                         .unwrap_or(0),
                     matches.is_present("PROGRESS_BAR"))
    });

    let runner = Runner {
        set: set,
        input: input,
        chunks: matches.value_of("CHUNKS").unwrap_or("999999999").parse().unwrap(),
        strategy: matches.value_of("REDUCE").unwrap_or("hashes").to_string(),
        sip: matches.is_present("SIP"),
        partial: matches.is_present("PARTIAL"),
        workers: matches.value_of("WORKERS")
                        .map(|a| a.parse::<usize>().unwrap())
                        .unwrap_or(::num_cpus::get() * 2),
        buckets: matches.value_of("BUCKETS").unwrap_or("257").parse().unwrap(),
        hosts: matches.value_of("HOSTS").map(|x| x.to_string()),
        me: matches.value_of("ME").map(|x| x.parse().unwrap()),
        monitor: monitor,
    };

    println!("runner: {:?}", runner);

    let t1 = ::time::get_time();

    let length: usize = matches.value_of("KEY_LENGTH").unwrap_or("8").parse().unwrap();
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
              rss_mb: {:5} vmmsize_mb: {:5} utime_s: {:5} stime_s: {:5} ctime_s: {:.03}",
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
             (t2 - t1).num_milliseconds() as f32 / 1000.0);

}

#[derive(Clone,Debug)]
struct Runner {
    set: String,
    input: String,
    chunks: usize,
    strategy: String,
    sip: bool,
    partial: bool,
    workers: usize,
    buckets: usize,
    // progress: bool,
    hosts: Option<String>,
    me: Option<usize>,
    monitor: Option<Arc<Monitor>>,
}

struct Sigil<I: Iterator<Item = T>, T> {
    monitor: Option<Arc<Monitor>>,
    count: usize,
    inner: I,
}

impl<I: Iterator<Item = T>, T> Sigil<I, T> {
    fn new(it: I, mon: Option<Arc<Monitor>>) -> Sigil<I, T> {
        Sigil {
            monitor: mon,
            count: 0,
            inner: it,
        }
    }
}

impl<I: Iterator<Item = T>, T> Iterator for Sigil<I, T> {
    type Item=T;
    fn next(&mut self) -> Option<T> {
        match self.inner.next() {
            Some(item) => {
                self.count += 1;
                Some(item)
            }
            None => {
                for m in self.monitor.as_ref() {
                    m.add_progress(1);
                    m.add_read(self.count);
                }
                None
            }
        }
    }
}

impl Runner {
    fn run<K>(mut self)
        where K: ShortBytesArray
    {
        if self.strategy == "timely" {
            self.run_timely::<K>();
        } else {
            self.run_standalone::<K>();
        }
    }

    fn shard<T: 'static + Send, I: Iterator<Item = T> + 'static + Send>
        (&self,
         it: I,
         index: usize,
         peers: usize)
         -> Box<Iterator<Item = T> + 'static + Send> {
        Box::new(it.take(self.chunks)
                   .enumerate()
                   .filter_map(move |(i, f)| {
                       if i % peers == index {
                           Some(f)
                       } else {
                           None
                       }
                   }))
    }

    fn sharded_input<'a, K>(&self, index: usize, peers: usize) -> BI<'a, BI<'a, (K, f32)>>
        where K: ShortBytesArray
    {
        let monitor = self.monitor.clone();
        if self.input.starts_with("cap") || self.input.starts_with("pcap") {
            Box::new(self.shard(dazone::files::bibi_cap(&*self.set, "uservisits", &*self.input),
                                index,
                                peers)
                         .map(move |chunk| -> BI<(K, f32)> {
                             Box::new(Sigil::new(chunk.map(move |reader: Reader<OwnedSegments>| {
                                                     let visit: cap::user_visits::Reader =
                                                         reader.get_root()
                                                               .unwrap();
                                                     (K::prefix(visit.get_source_i_p().unwrap()),
                                                      visit.get_ad_revenue())
                                                 }),
                                                 monitor.clone()))
                         }))
        } else if self.input.starts_with("buren") {
            let compressor = dazone::files::compressor::Compressor::for_format(&*self.input);
            Box::new(self.shard(dazone::files::files_for_format(&*self.set,
                                                                "uservisits",
                                                                &*self.input),
                                index,
                                peers)
                         .map(move |file| -> BI<(K, f32)> {
                             let reader = dazone::buren::PartialDeserializer::new(file,
                                                                                  compressor,
                                                                                  &[0, 3]);
                             Box::new(Sigil::new(reader.map(|pair: (String, f32)| {
                                                     (K::prefix(&*pair.0), pair.1)
                                                 }),
                                                 monitor.clone()))
                         }))
        } else if self.input == "mcap" {
            Box::new(self.shard(dazone::files::files_for_format(&*self.set,
                                                                "uservisits",
                                                                &*self.input),
                                index,
                                peers)
                         .map(move |file| -> BI<(K, f32)> {
                             Box::new(Sigil::new(dazone::files::cap::MmapReader::new(file),
                                                 monitor.clone()))
                         }))
        } else if self.input.starts_with("pbuf") {
            Box::new(self.shard(dazone::files::bibi_pbuf(&*self.set, "uservisits", &*self.input),
                                index,
                                peers)
                         .map(move |chunk| -> BI<(K, f32)> {
                             Box::new(Sigil::new(chunk.map(move |visit: ::dazone::data::pbuf::UserVisits| {
                                 (K::prefix(visit.get_sourceIP()), visit.get_adRevenue())
                             }), monitor.clone()))
                         }))
        } else {
            Box::new(self.shard(dazone::files::bibi_pod(&*self.set, "uservisits", &*self.input),
                                index,
                                peers)
                         .map(move |chunk| -> BI<(K, f32)> {
                             Box::new(Sigil::new(chunk.map(move |visit: ::dazone::data::pod::UserVisits| {
                                 (K::prefix(&*visit.source_ip), visit.ad_revenue)
                             }),monitor.clone()))
                         }))
        }
    }


    fn run_standalone<K>(&mut self)
        where K: ShortBytesArray
    {
        let r = |a: &f32, b: &f32| a + b;
        let bibi = self.sharded_input::<K>(0, 1);
        let groups = match &*self.strategy {
            "hash" => {
                let mut aggregator = ::dazone::crunch::aggregators::HashMapAggregator::new(&r)
                                         .with_monitor(self.monitor.clone());
                MapOp::new_map_reduce(|(a, b)| Emit::One(a, b))
                    .with_monitor(self.monitor.clone())
                    .with_workers(self.workers)
                    .run(bibi, &mut aggregator);
                aggregator.converge();
                aggregator.len()
            }
            "hashes" => {
                let hasher = ::dazone::crunch::fnv::FnvState;
                let mut aggregator =
                        ::dazone::crunch::aggregators::MultiHashMapAggregator::with_hasher(&r, self.buckets, hasher)
                        .with_monitor(self.monitor.clone())
                        .with_partial_aggregation(self.partial);
                MapOp::new_map_reduce(|(a, b)| Emit::One(a, b))
                    .with_monitor(self.monitor.clone())
                    .with_workers(self.workers)
                    .run(bibi, &mut aggregator);
                aggregator.converge();
                aggregator.len()
            }
            "tries" => {
                let mut aggregator =
                    ::dazone::crunch::aggregators::MultiTrieAggregator::new(&r, self.buckets)
                        .with_monitor(self.monitor.clone());
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
            for m in self.monitor.as_ref() {
                m.target.fetch_add(bibi.size_hint().1.unwrap_or(0),
                                   std::sync::atomic::Ordering::Relaxed);
            }
            let monitor = self.monitor.clone();
            root.scoped::<u64, _, _>(move |builder| {

                let uservisits = bibi.flat_map(move |inner| inner).to_stream(builder);

                let group_count = uservisits.unary_notify(Exchange::new(move |x: &(K, f32)| {
                                                              ::dazone::hash(&(x.0)) as u64
                                                          }),
                                                          "groupby-map",
                                                          vec![],
                                                          move |input, output, notif| {
                                                              while let Some((time, data)) =
                                                                        input.next() {
                                                                  notif.notify_at(time);
                                                                  for m in monitor.as_ref() {
                                                                      m.add_partial_aggreg(data.len());
                                                                  }
                                                                  let before = hashmap.len();
                                                                  for (k, v) in data.drain(..) {
                                                                      update_hashmap(&mut hashmap,
                                                                                     &|a, b| a + b,
                                                                                     k,
                                                                                     v);
                                                                  }
                                                                  for m in monitor.as_ref() {
                                                                      m.add_aggreg(hashmap.len() -
                                                                                   before);
                                                                  }
                                                              }
                                                              while let Some((iter, _)) =
                                                                        notif.next() {
                                                                  if hashmap.len() > 0 {
                                                                      output.session(&iter)
                                                                            .give(hashmap.len());
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
                                                                             for x in
                                                                                 data.drain(..) {
                                                                                 sum += x;
                                                                             }
                                                                         }
                                                                         notify.for_each(|_, _| {
                                                                             if sum > 0 {
                                                                                 println!("XXXX w\
                                                                                           orker: \
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
