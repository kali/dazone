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
    timely::execute_from_args(std::env::args(), move |root| {
        let mut hashmap = HashMap::new();
        let mut sum = 0usize;
        let index = root.index();

        let mut input = root.scoped(|builder| {
            let (input, uservisits) = builder.new_input::<(Vec<u8>, f32)>();

            let group_count: Stream<_, usize> =
                uservisits.unary_notify(Exchange::new(|x: &(Vec<u8>, f32)| {
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
                                                    println!("groupby {} worker at {:?} {}",
                                                             index,
                                                             iter,
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
        for (i, file) in data.drain(..).enumerate() {
            if i % root.peers() == root.index() {
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
                    input.send((chars, visit.get_ad_revenue()))
                }
            }
        }

        input.close();
        while root.step() {
        }
    });
}
