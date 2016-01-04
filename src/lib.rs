#![feature(iter_arith)]

extern crate glob;
extern crate simple_parallel;
extern crate num_cpus;
extern crate rmp;
extern crate rmp_serialize;
#[macro_use]
extern crate quick_error;
extern crate rustc_serialize;

pub mod mapred;

use rustc_serialize::{Encodable, Decodable};
use rmp_serialize::decode::Decoder;
use std::io;

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
        DecodeString { }
    }
}

pub type Dx16Result<T> = Result<T, Dx16Error>;

pub fn data_dir_for(state: &str, set: &str, table: &str) -> String {
    format!("data/{}/{}/{}", state, set, table)
}

#[derive(RustcDecodable,RustcEncodable,Debug)]
pub struct Ranking {
    pub url: String,
    pub pagerank: u64,
    pub duration: u64,
}

pub struct RankingReader<R: io::Read> {
    stream: Decoder<io::BufReader<R>>,
}

impl <R:io::Read> RankingReader<R> {
    pub fn new(r: R) -> RankingReader<R> {
        RankingReader { stream: Decoder::new(io::BufReader::new(r)) }
    }
}

unsafe impl<R:io::Read> Send for RankingReader<R> {}

impl <R:io::Read> Iterator for RankingReader<R> {
    type Item = Dx16Result<Ranking>;

    fn next(&mut self) -> Option<Dx16Result<Ranking>> {
        use rmp_serialize::decode::Error::InvalidMarkerRead;
        use rmp::decode::ReadError;
        let res: Result<Ranking, _> = Decodable::decode(&mut self.stream);
        match res {
            Err(InvalidMarkerRead(ReadError::UnexpectedEOF)) => None,
            Err(a) => Some(Err(Dx16Error::from(a))),
            Ok(r) => {
                Some(Ok(r))
            }
        }
    }
}
