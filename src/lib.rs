#![feature(hashmap_hasher)]
#![feature(reflect_marker)]
#![feature(iter_arith)]

extern crate abomonation;
extern crate bincode;
extern crate capdata;
extern crate capnp;
extern crate cbor;
extern crate csv;
extern crate flate2;
extern crate fnv;
extern crate glob;
extern crate libc;
extern crate num_cpus;
extern crate pbr;
#[macro_use]
extern crate quick_error;
extern crate radix_trie;
extern crate rmp;
extern crate rmp_serialize;
extern crate rustc_serialize;
extern crate simple_parallel;
extern crate snappy_framed;
extern crate time;

pub mod errors;
pub mod data;
pub mod files;
pub mod crunch;
pub mod rusage;
pub mod short_bytes_array;

pub use errors::*;

use std::hash::Hash;

pub fn hash<K: Hash>(k: &K) -> usize {
    use std::hash::{Hasher, SipHasher};
    let mut s = SipHasher::new();
    k.hash(&mut s);
    s.finish() as usize
}
