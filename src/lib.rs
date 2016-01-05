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

pub mod data;
pub mod mapred;

pub mod cap {
    include!(concat!(env!("OUT_DIR"), "/dx16_capnp.rs"));
}

use std::marker::PhantomData;

use rustc_serialize::Decodable;
use rmp_serialize::decode::Decoder;
use std::{io, path, process};
use std::io::{Read, BufReader};

use mapred::BI;

use data::UserVisits;

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

pub struct RMPReader<T: Decodable> {
    stream: Decoder<BufReader<PipeReader>>,
    phantom: PhantomData<T>,
}

impl < T:Decodable> RMPReader< T> {
    pub fn new(file: &path::Path) -> RMPReader<T> {
        let child = process::Command::new("gzcat")
                        .arg("-d")
                        .arg(file)
                        .stdout(process::Stdio::piped())
                        .spawn()
                        .unwrap();
        RMPReader {
            stream: Decoder::new(BufReader::new(PipeReader { child: child })),
            phantom: PhantomData,
        }
    }
}

unsafe impl< T:Decodable> Send for RMPReader< T> {}

impl < T:Decodable> Iterator for RMPReader< T> {
    type Item = Dx16Result<T>;

    fn next(&mut self) -> Option<Dx16Result<T>> {
        use rmp_serialize::decode::Error::InvalidMarkerRead;
        use rmp::decode::ReadError;
        let res: Result<T, _> = Decodable::decode(&mut self.stream);
        match res {
            Err(InvalidMarkerRead(ReadError::UnexpectedEOF)) => None,
            Err(a) => Some(Err(Dx16Error::from(a))),
            Ok(r) => {
                Some(Ok(r))
            }
        }
    }
}

pub fn bibi_rmp_gz_dec<'a, 'b, T>(set: &str, table: &str) -> BI<'a, BI<'b, Dx16Result<T>>>
    where T: Decodable + 'static
{
    let source_root = data_dir_for("rmp-gz", set, table);
    let glob = source_root.clone() + "/*.rmp.gz";
    let files: Vec<path::PathBuf> = ::glob::glob(&glob)
                                        .unwrap()
                                        .map(|p| p.unwrap().to_owned())
                                        .collect();
    Box::new(files.into_iter()
                  .map(|f| -> BI<Dx16Result<T>> { Box::new(RMPReader::new(&f)) }))
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
            Ok(msg) => {
                Some(Ok(msg))
            }
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

pub struct RankingRMPReader<R: io::Read> {
    stream: io::BufReader<R>,
    buf: [u8; 1024],
}

impl <R:io::Read> RankingRMPReader<R> {
    pub fn new(r: R) -> RankingRMPReader<R> {
        RankingRMPReader {
            stream: io::BufReader::new(r),
            buf: [0u8; 1024],
        }
    }
}

impl <R:io::Read> Iterator for RankingRMPReader<R> {
    type Item = Dx16Result<u32>;

    fn next(&mut self) -> Option<Dx16Result<u32>> {
        use rmp::decode::*;
        let _marker = match read_marker(&mut self.stream) {
            Err(rmp::decode::MarkerReadError::UnexpectedEOF) => return None,
            Err(error) => panic!(error),
            Ok(mark) => mark,
        };
        let _s = read_str(&mut self.stream, &mut self.buf);
        let pagerank = read_u32_loosely(&mut self.stream);
        if pagerank.is_err() {
            return Some(Err(Dx16Error::DecodeString));
        }
        let duration = read_u32_loosely(&mut self.stream);
        if duration.is_err() {
            return Some(Err(Dx16Error::DecodeString));
        }
        Some(Ok(pagerank.unwrap() as u32))
    }
}
