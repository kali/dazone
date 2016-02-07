#![feature(hashmap_hasher)]
#![feature(reflect_marker)]
#![feature(iter_arith,plugin)]
#![feature(custom_derive)]

#![plugin(serde_macros)]

extern crate abomonation;
extern crate bincode;
extern crate byteorder;
extern crate capdata;
extern crate capnp;
extern crate cbor;
extern crate csv;
extern crate flate2;
extern crate fnv;
extern crate glob;
extern crate itertools;
extern crate libc;
extern crate lz4;
extern crate memmap;
extern crate num_cpus;
extern crate pbr;
extern crate protobuf;
#[macro_use]
extern crate quick_error;
extern crate radix_trie;
extern crate rand;
extern crate rmp;
extern crate rmp_serialize;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
extern crate simple_parallel;
extern crate snappy_framed;
extern crate time;

pub mod buren;
pub mod errors;
pub mod data;
pub mod files;
pub mod crunch;
pub mod rusage;
pub mod short_bytes_array;
pub mod timely_accumulators;

pub use errors::*;

use std::hash::{ Hash, Hasher };

pub fn hash<K: Hash>(k: &K) -> usize {
    let mut s  = fnv::FnvHasher::default();
    k.hash(&mut s);
    s.finish() as usize
}
