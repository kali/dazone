extern crate timely;
extern crate dazone;

use timely::dataflow::*;
use timely::dataflow::channels::pact::Exchange;
use timely::dataflow::operators::to_stream::ToStream;
use timely::dataflow::operators::unary::Unary;

use dazone::short_bytes_array::ShortBytesArray;
use dazone::short_bytes_array::Bytes8;

use dazone::buren::PartialDeserializer;
use dazone::files::compressor::Compressor;

use dazone::crunch::aggregators::update_hashmap;

fn main() {
    timely::execute_from_args(std::env::args(), move |root| {
        let peers = root.peers();
        let index = root.index();

        root.scoped::<u64, _, _>(move |builder| {
            let files = dazone::files::files_for_format("5nodes", "uservisits", "buren-snz");
            let files = files.enumerate().filter_map(move |(i, f)| {
                if i % peers == index {
                    Some(f)
                } else {
                    None
                }
            });
            let uservisits = files.flat_map(|file| {
                PartialDeserializer::new(file, Compressor::get("snz"), &[0, 3])
            });

            let uservisits = uservisits.map(|visit: (String, f32)| {
                (Bytes8::prefix(&*visit.0), visit.1)
            });

            let stream = uservisits.to_stream(builder);

            let mut hashmap = ::std::collections::HashMap::new();
            let mut sum = 0usize;

            let group = stream.unary_notify(Exchange::new(move |x: &(Bytes8, f32)| {
                                                ::dazone::hash(&(x.0)) as u64
                                            }),
                                            "group-count",
                                            vec![],
                                            move |input, output, notif| {
                                                input.for_each(|time, chunk| {
                                                    notif.notify_at(time);
                                                    for (k, v) in chunk.drain(..) {
                                                        update_hashmap(&mut hashmap,
                                                                       &|&a, &b| a + b,
                                                                       k,
                                                                       v);
                                                    }
                                                });
                                                notif.for_each(|time, _| {
                                                    if hashmap.len() > 0 {
                                                        output.session(&time)
                                                              .give(hashmap.len());
                                                    }
                                                });
                                            });

            let _: Stream<_, ()> = group.unary_notify(Exchange::new(|_| 0u64),
                                                      "count",
                                                      vec![],
                                                      move |input, _, notif| {
                                                          input.for_each(|time, data| {
                                                              notif.notify_at(time);
                                                              for x in data.drain(..) {
                                                                  sum += x;
                                                              }
                                                          });
                                                          notif.for_each(|_, _| {
                                                              if index == 0 {
                                                                  println!("result: {}", sum);
                                                              }
                                                          });
                                                      });
        });
        while root.step() {}
    })
}
