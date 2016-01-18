#![feature(reflect_marker)]
#![feature(iter_arith)]

#[macro_use]
extern crate quick_error;
extern crate rustc_serialize;

extern crate radix_trie;

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
