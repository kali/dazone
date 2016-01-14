extern crate dx16;
extern crate time;
extern crate timely;
extern crate capnp;

use timely::dataflow::*;
use timely::dataflow::operators::*;

use timely::progress::timestamp::RootTimestamp;

use timely::dataflow::channels::pact::Exchange;

use std::io::Read;

use std::iter::Iterator;

use std::fs::File;
use std::path;

use std::collections::HashMap;

use dx16::mapred::BI;

fn main() {
    ::dx16::rusage::start_monitor(::std::time::Duration::from_secs(10));
    let t1 = ::time::get_time();
    timely::execute_from_args(std::env::args(), move |root| {
        let mut hashmap = HashMap::new();
        let mut sum = 0usize;
        let index = root.index();
        let peers = root.peers();

        let worker_files: Vec<path::PathBuf> = ::dx16::files_for_format("5nodes",
                                                                        "uservisits",
                                                                        "cap")
                                                   .into_iter()
                                                   .enumerate()
                                                   .filter(move |ref pair| pair.0 % peers == index)
                                                   .map(|x| x.1)
                                                   .collect();

        let data: BI<(Vec<u8>, f32)> =
            Box::new(worker_files.into_iter()
                                 .flat_map(|file| {
                                     ::dx16::CapGzReader::new(File::open(file).unwrap())
                                         .map(|reader| {
                                             let reader = reader.unwrap();
                                             let visit: ::dx16::cap::user_visits::Reader =
                                                 reader.get_root()
                                                       .unwrap();
                                             let chars: Vec<u8> = visit.get_source_i_p()
                                                                       .unwrap()
                                                                       .as_bytes()
                                                                       .into_iter()
                                                                       .take(12)
                                                                       .map(|a| *a)
                                                                       .collect();
                                             (chars, visit.get_ad_revenue())
                                         })
                                 }));

        root.scoped(|builder| {

            let uservisits = data.to_stream(builder);

            let group_count =
                uservisits.unary_notify(Exchange::new(move |x: &(Vec<u8>, f32)| {
                                            ::dx16::hash(&x.0) as u64
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

            let _count = group_count.unary_notify(Exchange::new(|_| 0u64),
                                                  "count",
                                                  vec![RootTimestamp::new(0)],
                                                  move |input, output, notif| {
                                                      notif.notify_at(&RootTimestamp::new(0));
                                                      while let Some((_, data)) = input.next() {
                                                          for x in data.drain(..) {
                                                              sum += x;
                                                          }
                                                      }
                                                      while let Some((iter, _)) = notif.next() {
                                                          if sum > 0 {
                                                              output.session(&iter).give(sum);
                                                              sum = 0;
                                                          }
                                                      }

                                                  });

        });

        while root.step() {
        }
    });
    let t2 = ::time::get_time();
    println!("ctime: {}", (t2-t1).num_seconds());
    ::dx16::rusage::dump_memory_stats();
}
