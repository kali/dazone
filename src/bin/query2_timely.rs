extern crate dx16;
extern crate timely;

use timely::dataflow::*;
use timely::dataflow::operators::*;

use timely::progress::timestamp::RootTimestamp;

use timely::dataflow::channels::pact::Exchange;

use std::io::Read;

use std::fs::File;

use std::collections::HashMap;

fn main() {
    ::dx16::rusage::start_monitor(::std::time::Duration::from_secs(10));
    timely::execute_from_args(std::env::args(), move |root| {
        let mut hashmap = HashMap::new();
        let mut sum = 0usize;
        let index = root.index();
        let peers = root.peers();
        let mut seen = 0;

        let mut input = root.scoped(|builder| {
            let (input, files) = builder.new_input::<String>();

            let uservisits = files.unary_stream(Exchange::new(move |x| ::dx16::hash(x) as u64),
                                                "loader",
                                                move |input, output| {
                                                    input.for_each(|iter, data| {
                    println!("worker {}, {:?} files", index, data.len());
                    for file in data.iter() {
                        let mut session = output.session(&iter);
                        for reader in ::dx16::CapGzReader::new(File::open(file).unwrap()) {
                            let reader = reader.unwrap();
                            let visit: ::dx16::cap::user_visits::Reader = reader.get_root()
                                .unwrap();
                            let chars: Vec<u8> = visit.get_source_i_p()
                                .unwrap()
                                .as_bytes()
                                .into_iter()
                                .take(8)
                                .map(|a| *a)
                                .collect();
                            session.give((chars, visit.get_ad_revenue()));
                        }}})
                                                });

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
                                                    if seen == 0 {
                                                        println!("first data in {}", index);
                                                    }
                                                    seen += 1;
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
                                                    println!("groupby {} worker at {:?} seen:{} \
                                                              mapped:{}",
                                                             index,
                                                             iter,
                                                             seen,
                                                             hashmap.len());
                                                    output.session(&iter).give(hashmap.len());
                                                    hashmap.clear();
                                                }
                                            }

                                        });

            let count = group_count.unary_notify(Exchange::new(|_| 0u64),
                                                 "count",
                                                 vec![RootTimestamp::new(0)],
                                                 move |input, output, notif| {
                                                     notif.notify_at(&RootTimestamp::new(0));
                                                     while let Some((_, data)) = input.next() {
                                                         for x in data.drain(..) {
                                                             sum += x;
                                                             println!("count worker {} receive \
                                                                       {} sum:{}",
                                                                      index,
                                                                      x,
                                                                      sum);
                                                         }
                                                     }
                                                     while let Some((iter, _)) = notif.next() {
                                                         if sum > 0 {
                                                             println!("groups: {} iter: {:?}",
                                                                      sum,
                                                                      iter);
                                                             output.session(&iter).give(sum);
                                                             sum = 0;
                                                         }
                                                     }

                                                 });

            input
        });

        let mut data = ::dx16::files_for_format("5nodes", "uservisits", "cap");
        if root.index() == 0 {
            for file in data.drain(..) {
                input.send(file.to_str().unwrap().to_string())
            }

            println!("input done ({})", root.index());
        }
        input.close();
        while root.step() {
        }
    });
}
