#![feature(float_extras)]
#![feature(iter_arith)]

extern crate timely;
extern crate abomonation;
extern crate differential_dataflow;
extern crate dazone;


use timely::dataflow::*;
use timely::dataflow::operators::input::Input;

use differential_dataflow::Collection;
use differential_dataflow::operators::group::Group;

use dazone::short_bytes_array::ShortBytesArray;
use dazone::short_bytes_array::Bytes8;

use dazone::buren::PartialDeserializer;
use dazone::files::compressor::Compressor;

use std::hash::Hasher;

#[derive(Clone,Copy,PartialEq,PartialOrd,Default,Debug)]
struct SaneF32(f32);
impl abomonation::Abomonation for SaneF32 {}
impl ::std::cmp::Eq for SaneF32 {}

impl ::std::cmp::Ord for SaneF32 {
    fn cmp(&self, other: &SaneF32) -> ::std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl ::std::hash::Hash for SaneF32 {
    fn hash<H>(&self, state: &mut H)
        where H: Hasher
    {
        let triplet = self.0.integer_decode();
        triplet.0.hash(state);
        triplet.1.hash(state);
        triplet.2.hash(state);
    }
}

fn main() {
    timely::execute_from_args(std::env::args(), move |root| {
        let peers = root.peers();
        let index = root.index();

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

        let mut input = root.scoped::<u64, _, _>(move |builder| {
            let (input, stream) = builder.new_input::<((Bytes8, _), i32)>();
            let collections = Collection::new(stream);

            let group: Collection<_, (Bytes8, SaneF32)> = collections.group(|_, vs, o| {
                let v: f32 = vs.map(|(sane, weight): (&SaneF32, i32)| sane.0 * weight as f32).sum();
                o.push((SaneF32(v), 1));
            });
            let count: Collection<_, (bool, u32)> = group.map(|(_, _): (Bytes8, SaneF32)| {
                (true, 1)
            });
            let count: Collection<_, (bool, i32)> = count.group(|_, vs, o| {
                let c: i32 = vs.map(|(c, weight): (&u32, i32)| *c as i32 * weight).sum();
                o.push((c, 1));
            });
            count.inspect(move |rec| println!("XXX {} XXX", (rec.0).1));

            input
        });

        for visit in uservisits {
            input.send(((Bytes8::prefix(&*visit.0), SaneF32(visit.1)), 1));
        }
    })
}
