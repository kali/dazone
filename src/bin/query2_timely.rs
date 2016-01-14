#![feature(reflect_marker)]

extern crate dx16;
extern crate time;
extern crate timely;
extern crate capnp;
extern crate abomonation;

use timely::dataflow::*;
use timely::dataflow::operators::*;

use timely::progress::timestamp::RootTimestamp;

use timely::dataflow::channels::pact::Exchange;

use std::iter::Iterator;

use std::fs::File;
use std::path;

use std::collections::HashMap;

use std::hash::{Hash, Hasher};

use dx16::mapred::BI;
use abomonation::Abomonation;

trait FixedBytesArray {
    fn prefix(s: &str) -> Self;
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
fixed_bytes_array!( K12 12 );

fn main() {
    let t1 = ::time::get_time();
    ::dx16::rusage::start_monitor(::std::time::Duration::from_secs(10));
    run::<K12>();
    let t2 = ::time::get_time();
    println!("ctime: {}", (t2 - t1).num_seconds());
    ::dx16::rusage::dump_memory_stats();
}

fn run<K>() where
        K:FixedBytesArray+Abomonation+Send+Clone+::std::marker::Reflect+::std::hash::Hash+Eq+'static {
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

        let data: BI<(K, f32)> =
            Box::new(worker_files.into_iter()
                                 .flat_map(|file| {
                                     ::dx16::CapGzReader::new(File::open(file).unwrap())
                                         .map(|reader| {
                                             let reader = reader.unwrap();
                                             let visit: ::dx16::cap::user_visits::Reader =
                                                 reader.get_root()
                                                       .unwrap();
                                             (K::prefix(visit.get_source_i_p().unwrap()),
                                              visit.get_ad_revenue())
                                         })
                                 }));

        root.scoped(|builder| {

            let uservisits = data.to_stream(builder);

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

            let _count:Stream<_,()> = group_count.unary_stream(Exchange::new(|_| 0u64),
                                                  "count",
                                                  move |input, _| {
                                                      while let Some((_, data)) = input.next() {
                                                          for x in data.drain(..) {
                                                              sum += x;
                                                          }
                                                      }
                                                  });

        });

        while root.step() {
        }

        if index == 0 {
            println!("groups: {}", sum);
        }
    });
}
