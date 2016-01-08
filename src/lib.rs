#![feature(iter_arith)]

extern crate glob;
extern crate simple_parallel;
extern crate num_cpus;
extern crate rmp;
extern crate rmp_serialize;
#[macro_use]
extern crate quick_error;
extern crate rustc_serialize;
extern crate pbr;
extern crate capnp;
// extern crate rayon;
extern crate time;
extern crate radix_trie;

pub mod data;
pub mod mapred;
pub mod aggregators;
pub mod rmp_read;

pub mod cap {
    include!(concat!(env!("OUT_DIR"), "/dx16_capnp.rs"));
}


use std::{io, path, process};
use std::io::{Read, BufReader};

use mapred::BI;

use capnp::serialize_packed;
use capnp::serialize::OwnedSegments;
use capnp::message::Reader;

quick_error! {
#[derive(Debug)]
    pub enum Dx16Error {
        Io(err: std::io::Error) { from() }
        GlobPattern(err: glob::PatternError) { from() }
        GlobGlob(err: glob::GlobError) { from() }
        ParseInt(err: std::num::ParseIntError) { from() }
        ValueWrite(err: rmp::encode::ValueWriteError) { from() }
        ValueRead(err: rmp::decode::ValueReadError) { from() }
        RmpDecode(err: rmp_serialize::decode::Error) { from() }
        Capnp(err: capnp::Error) { from() }
        DecodeString { }
    }
}

pub type Dx16Result<T> = Result<T, Dx16Error>;

pub fn data_dir_for(state: &str, set: &str, table: &str) -> String {
    format!("data/{}/{}/{}", state, set, table)
}

pub struct PipeReader {
    child: process::Child,
}

impl Read for PipeReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let res = self.child.stdout.as_mut().unwrap().read(buf);
        if let Ok(0) = res {
            try!(self.child.wait());
        }
        res
    }
}

pub fn bibi_cap_gz_dec<'a, 'b>(set: &str,
                               table: &str)
                               -> BI<'a, BI<'b, Dx16Result<Reader<OwnedSegments>>>> {
    let source_root = data_dir_for("cap-gz", set, table);
    let glob = source_root.clone() + "/*.cap-gz";
    let files: Vec<path::PathBuf> = ::glob::glob(&glob)
                                        .unwrap()
                                        .map(|p| p.unwrap().to_owned())
                                        .collect();
    Box::new(files.into_iter()
                  .map(|f| -> BI<Dx16Result<Reader<OwnedSegments>>> {
                      Box::new(CapGzReader::new(&f))
                  }))
}

pub struct CapGzReader {
    options: capnp::message::ReaderOptions,
    stream: io::BufReader<PipeReader>,
}

impl CapGzReader {
    pub fn new(file: &path::Path) -> CapGzReader {
        let child = process::Command::new("gzcat")
                        .arg("-d")
                        .arg(file)
                        .stdout(process::Stdio::piped())
                        .spawn()
                        .unwrap();
        CapGzReader {
            options: capnp::message::ReaderOptions::new(),
            stream: BufReader::new(PipeReader { child: child }),
        }
    }
}

impl Iterator for CapGzReader {
    type Item = Dx16Result<Reader<OwnedSegments>>;

    fn next(&mut self) -> Option<Dx16Result<Reader<OwnedSegments>>> {
        match serialize_packed::read_message(&mut self.stream, self.options) {
            Ok(msg) => Some(Ok(msg)),
            Err(err) => {
                use std::error::Error;
                if err.description().contains("Premature EOF") {
                    return None;
                } else {
                    return Some(Err(Dx16Error::from(err)));
                }
            }
        }
    }
}
