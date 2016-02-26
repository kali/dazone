#![feature(float_extras)]
#![feature(iter_arith)]

#[macro_use]
extern crate dazone;
extern crate capnp;
extern crate capdata as cap;
#[macro_use]
extern crate clap;
extern crate chrono;
extern crate fnv;
extern crate time;
extern crate num_cpus;
extern crate abomonation;
extern crate timely;
extern crate timely_communication;
extern crate differential_dataflow;
extern crate libc;

use std::sync;
use std::sync::atomic::Ordering::Relaxed;

use dazone::crunch::{BI, Emit, MapOp, Aggregator};
use dazone::short_bytes_array::*;
use dazone::rusage::*;

use timely::dataflow::*;
use timely::dataflow::operators::*;
use timely::dataflow::channels::pact::Exchange;
use timely::dataflow::scopes::root::Root;
use timely_communication::allocator::generic::Generic;

use capnp::serialize::OwnedSegments;
use capnp::message::Reader;

use std::hash::Hasher;

#[derive(Clone,Copy,PartialEq,PartialOrd,Default,Debug)]
struct SaneF32(f32);
impl abomonation::Abomonation for SaneF32 {}
impl ::std::cmp::Eq for SaneF32 {
}
impl ::std::cmp::Ord for SaneF32 {
    fn cmp(&self, other:&SaneF32) -> ::std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl ::std::hash::Hash for SaneF32 {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        let triplet = self.0.integer_decode();
        triplet.0.hash(state);
        triplet.1.hash(state);
        triplet.2.hash(state);
    }
}


fn main() {
    let app = clap_app!( query2 =>
                         (@arg SET: -s --set +takes_value "(tiny, 1node, 5nodes")
                         (@arg INPUT: -i --input +takes_value "(cap, rmp)")
                         (@arg CHUNKS: -c --chunks +takes_value "all")
                         (@arg KEY_LENGTH: -k --key_length +takes_value "(8, 9, 10, 11, 12)")
                         (@arg REDUCE: -r --reduce +takes_value "(hash, hashes, tries, timely)")
                         (@arg SIP: --sip "(use SIP hasher)")
                         (@arg BUCKETS: -b --buckets +takes_value "reduce buckets (256)")
                         (@arg PARTIAL: -p --partial "activate partial aggregation")
                         (@arg MONITOR: -m --monitor +takes_value "monitor resouce usage")
                         (@arg WORKERS: -w --workers +takes_value "worker threads (num_cpu*2)")
                         (@arg HOSTS: -h --hosts +takes_value "hosts, coma sep (for timely)")
                         (@arg ME: --me +takes_value "my position in hosts (starting at 0)")
                         (@arg PROGRESS_BAR: --pb "show progress bar")
                         (@arg VERBOSE: -v --verbose "verbose")
                       );
    let matches = app.get_matches();
    let start = chrono::UTC::now();

    let set = matches.value_of("SET").unwrap_or("5nodes").to_string();
    let input = matches.value_of("INPUT").unwrap_or("buren-snz").to_string();

    let monitor: sync::Arc<Monitor> = Monitor::new(time::Duration::seconds(1),
    matches.value_of("MONITOR").map(|f| {
        ::std::fs::File::create(f).unwrap()
    }),
    ::dazone::files::files_for_format(&*set,
                                      "uservisits",
                                      &*input)
    .size_hint()
    .1
    .unwrap_or(0),
    matches.is_present("PROGRESS_BAR"));

    let runner = Runner {
        set: set,
        input: input,
        chunks: matches.value_of("CHUNKS").map(|x| x.parse().unwrap()),
        strategy: matches.value_of("REDUCE").unwrap_or("hashes").to_string(),
        sip: matches.is_present("SIP"),
        partial: matches.is_present("PARTIAL"),
        workers: matches.value_of("WORKERS")
            .map(|a| a.parse::<usize>().unwrap())
            .unwrap_or(::num_cpus::get() * 2),
            buckets: matches.value_of("BUCKETS").unwrap_or("256").parse().unwrap(),
            hosts: matches.values_of("HOSTS").map(|x| x.map(|v| v.to_string()).collect::<Vec<_>>()),
            me: matches.value_of("ME").map(|x| x.parse().unwrap()),
            monitor: monitor,
    };

    if matches.is_present("VERBOSE") {
        println!("{:?}", runner);
    }

    let t1 = ::time::get_time();

    let length: usize = matches.value_of("KEY_LENGTH").unwrap_or("8").parse().unwrap();
    let (groups, inner_timer) = match length {
        8 => runner.clone().run::<Bytes8>(),
        9 => runner.clone().run::<Bytes9>(),
        10 => runner.clone().run::<Bytes10>(),
        11 => runner.clone().run::<Bytes11>(),
        12 => runner.clone().run::<Bytes12>(),
        _ => panic!("key length {} not implemented"),
    };
    let t2 = ::time::get_time();

    let usage = ::dazone::rusage::get_rusage();
    let vmsize = ::dazone::rusage::get_memory_usage().unwrap().virtual_size;
    println!("{} length: {:2} strat: {:6} buckets: {:4} workers: {:4} hosts: {:2} rss_mb: {:5} vmmsize_mb: {:5} \
              utime_s: {:5} stime_s: {:5} ctime_s: {:5.01} groups: {:9} inner_time: {:5.01}",
              start.format("%+"),
              length,
              &*runner.strategy,
              runner.buckets,
              runner.workers,
              runner.hosts.map(|h| h.len()).unwrap_or(0),
              usage.ru_maxrss / 1024 / 1024,
              vmsize / 1024 / 1024,
              usage.ru_utime.tv_sec,
              usage.ru_stime.tv_sec,
              (t2 - t1).num_milliseconds() as f32 / 1000.0,
              groups,
              inner_timer.num_milliseconds() as f32 / 1000.0
            );

}

#[derive(Clone,Debug)]
struct Runner {
    set: String,
    input: String,
    chunks: Option<usize>,
    strategy: String,
    sip: bool,
    partial: bool,
    workers: usize,
    buckets: usize,
    // progress: bool,
    hosts: Option<Vec<String>>,
    me: Option<usize>,
    monitor: sync::Arc<Monitor>,
}

struct Sigil<I: Iterator<Item = T>, T> {
    monitor: sync::Arc<Monitor>,
    count: usize,
    inner: I,
}

impl<I: Iterator<Item = T>, T> Sigil<I, T> {
    fn new(it: I, mon: sync::Arc<Monitor>) -> Sigil<I, T> {
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
                self.monitor.add_progress(1);
                self.monitor.add_read(self.count);
                None
            }
        }
    }
}

impl Runner {
    fn run<K>(mut self) -> (usize, time::Duration)
        where K: ShortBytesArray
        {
            if self.strategy == "timely" || self.strategy == "timelydd" {
                self.run_timely::<K>()
            } else {
                self.run_standalone::<K>()
            }
        }

    fn shard<T: 'static + Send, I: Iterator<Item = T> + 'static + Send>
        (&self,
         it: I,
         index: usize,
         peers: usize)
            -> Box<Iterator<Item = T> + 'static + Send> {
                Box::new(it.take(self.chunks.unwrap_or(99999999))
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

    fn run_standalone_hashes<K, S>(&self, bibi: BI<BI<(K, f32)>>, hasher: S) -> usize
        where K: Send + Eq + ::std::hash::Hash + 'static,
              S: Send + Sync + ::std::hash::BuildHasher + 'static + Clone
              {
                  let r = |a: &f32, b: &f32| a + b;
                  let mut aggregator =
                      ::dazone::crunch::aggregators::MultiHashMapAggregator::with_hasher(&r,
                                                                                         self.buckets,
                                                                                         hasher)
                      .with_monitor(Some(self.monitor.clone()))
                      .with_partial_aggregation(self.partial);
                  MapOp::new_map_reduce(|(a, b)| Emit::One(a, b))
                      .with_monitor(Some(self.monitor.clone()))
                      .with_workers(self.workers)
                      .run(bibi, &mut aggregator);
                  aggregator.converge();
                  aggregator.len() as usize
              }

    fn run_standalone<K>(&mut self) -> (usize, time::Duration)
        where K: ShortBytesArray
        {
            let t1 = time::get_time();
            let r = |a: &f32, b: &f32| a + b;
            let bibi = self.sharded_input::<K>(0, 1);
            let groups = match &*self.strategy {
                "hash" => {
                    let mut aggregator = ::dazone::crunch::aggregators::HashMapAggregator::new(&r)
                        .with_monitor(Some(self.monitor.clone()));
                    MapOp::new_map_reduce(|(a, b)| Emit::One(a, b))
                        .with_monitor(Some(self.monitor.clone()))
                        .with_workers(self.workers)
                        .run(bibi, &mut aggregator);
                    aggregator.converge();
                    aggregator.len() as usize
                }
                "hashes" => {
                    if self.sip {
                        self.run_standalone_hashes(bibi, std::collections::hash_map::RandomState::new())
                    } else {
                        self.run_standalone_hashes(bibi, ::dazone::crunch::fnv::FnvState)
                    }
                }
                "tries" => {
                    let mut aggregator =
                        ::dazone::crunch::aggregators::MultiTrieAggregator::new(&r, self.buckets)
                        .with_monitor(Some(self.monitor.clone()));
                    MapOp::new_map_reduce(|(a, b): (K, f32)| Emit::One(a.to_vec(), b))
                                                    .with_monitor(Some(self.monitor.clone()))
                                                    .with_workers(self.workers)
                                                    .run(bibi, &mut aggregator);
                    aggregator.converge();
                    aggregator.len() as usize
                }
                s => panic!("unknown strategy {}", s),
            };
            let t2 = time::get_time();
            (groups, t2-t1)
        }

    fn run_timely<K>(self) -> (usize, time::Duration)
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
                    let position = self.me.unwrap_or_else(|| {
                        hosts.iter().position(|h| h == &*gethostname()).unwrap()
                    });
                    let hosts_with_ports: Vec<String> = hosts.iter()
                        .enumerate()
                        .map(|(index, host)| {
                            format!("{}:{}", host, 2101 + index)
                        })
                    .collect();
                    ::timely::Configuration::Cluster(self.workers, position, hosts_with_ports, false)
                }
                None => ::timely::Configuration::Process(self.workers),
            };

            let result = sync::Arc::new(sync::atomic::AtomicUsize::new(0));
            let result_to_go = result.clone();
            let t1:sync::Arc<sync::Mutex<Option<time::Timespec>>> = sync::Arc::new(sync::Mutex::new(None));
            let t1_to_go = t1.clone();

            timely::execute(conf, move |root| {
                use dazone::timely_accumulators::HashMapAccumulator;
                {
                    let mut t1 = t1_to_go.lock().unwrap();
                    *t1 = Some(time::get_time());
                }
                if self.strategy == "timely" {
                    let accu: HashMapAccumulator<K, f32, _> = HashMapAccumulator::new(|a: &f32,
                                                                                      b: &f32| {
                                                                                          a + b
                                                                                      });
                    self.run_timely_with_accumulator(accu, result_to_go.clone(), root);
                } else {
                    self.run_timely_diff_data::<K>(result_to_go.clone(), root);
                }
            });
            let t1 = t1.lock().unwrap();
            (result.fetch_add(0, Relaxed), time::get_time() - t1.unwrap())
        }

    fn run_timely_with_accumulator<K, A>(&self,
                                         mut accu: A,
                                         result: sync::Arc<sync::atomic::AtomicUsize>,
                                         root: &mut Root<Generic>)
        where K: ShortBytesArray,
              A: dazone::timely_accumulators::SimpleAccumulator<K, f32> + 'static
              {
                  let result_to_go = result.clone();
                  let mut sum = 0usize;
                  let index = root.index();
                  let peers = root.peers();
                  let bibi = self.sharded_input::<K>(index, peers);
                  self.monitor.target.fetch_add(bibi.size_hint().1.unwrap_or(0), Relaxed);
                  let monitor = self.monitor.clone();
                  root.scoped::<u64, _, _>(move |builder| {
                      let result_to_go = result_to_go.clone();

                      let uservisits = bibi.flat_map(move |inner| inner).to_stream(builder);

                      let group_count =
                          uservisits.unary_notify(Exchange::new(move |x: &(K, f32)| {
                              ::dazone::hash(&(x.0)) as u64
                          }),
                          "groupby-map",
                          vec![],
                          move |input, output, notif| {
                              while let Some((time, data)) = input.next() {
                                  notif.notify_at(time);
                                  monitor.add_partial_aggreg(data.len());
                                  // let before = hashmap.len();
                                  //
                                  for (k, v) in data.drain(..) {
                                      accu.offer(k, v);
                                  }
                                  // monitor.add_aggreg(hashmap.len() - before);
                              }
                              while let Some((iter, _)) = notif.next() {
                                  accu.converge();
                                  if accu.len() > 0 {
                                      output.session(&iter)
                                          .give(accu.len());
                                  }
                              }
                          });

                      let _count: Stream<_, ()> = group_count.unary_notify(Exchange::new(|_| 0u64),
                      "count",
                      vec![],
                      move |input, _, notify| {
                          while let Some((time, data)) =
                              input.next() {
                                  notify.notify_at(time);
                                  for x in data.drain(..) {
                                      sum += x;
                                  }
                              }
                          notify.for_each(|_, _| {
                              if sum > 0 {
                                  result_to_go.store(sum, Relaxed);
                                  sum = 0;
                              }
                          })
                      });

                  });
                  while root.step() {
                  }

              }

    fn run_timely_diff_data<K>(&self,
                               result: sync::Arc<sync::atomic::AtomicUsize>,
                               root: &mut Root<Generic>)
        where K: ShortBytesArray
        {
            use differential_dataflow::Collection;
            use differential_dataflow::operators::consolidate::ConsolidateExt;
            use differential_dataflow::operators::group::Group;

            let index = root.index();
            let peers = root.peers();
            let bibi = self.sharded_input::<K>(index, peers);
            self.monitor.target.fetch_add(bibi.size_hint().1.unwrap_or(0), Relaxed);
            let result = result.clone();

            let mut input = root.scoped::<u64, _, _>(move |builder| {
                let (input, stream) = builder.new_input::<((K, _), i32)>();
                let collections = Collection::new(stream);


                let group:Collection<_, (K,SaneF32)> = collections.group(|_,vs,o| {
                    let v:f32 = vs.map(|(sane,weight):(&SaneF32,i32)| sane.0*weight as f32).sum();
                    o.push( (SaneF32(v), 1) );
                });
                let count:Collection<_, (bool, u32)> = group.map(|(_,_):(K,SaneF32)| (true, 1)).consolidate();
                let count:Collection<_, (bool, i32)> = count.group(|_,vs,o| {
                    let c:i32 = vs.map(|(c,weight):(&u32,i32)| *c as i32 * weight).sum();
                    o.push((c, 1));
                });
                count.inspect(move |rec| result.store((rec.0).1 as usize, Relaxed));

            input
            });

            for (_i,chunk) in bibi.enumerate() {
                for item in chunk {
                    input.send(((item.0, SaneF32(item.1)), 1));
                }
            //    input.advance_to(i as u64 + 1);
            //    root.step();
            }

        }
}
