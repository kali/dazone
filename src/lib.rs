#![feature(iter_arith)]

extern crate glob;
extern crate simple_parallel;
extern crate num_cpus;
extern crate rmp;
extern crate rmp_serialize;
#[macro_use]
extern crate quick_error;
extern crate rustc_serialize;

pub mod data;
pub mod mapred;

use rustc_serialize::Decodable;
use rmp_serialize::decode::Decoder;
use std::io;

use data::Ranking;

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

pub struct RankingRankingReader<R: io::Read> {
    stream: io::BufReader<R>,
    buf: [u8; 1024],
}

impl <R:io::Read> RankingRankingReader<R> {
    pub fn new(r: R) -> RankingRankingReader<R> {
        RankingRankingReader {
            stream: io::BufReader::new(r),
            buf: [0u8; 1024],
        }
    }
}

impl <R:io::Read> Iterator for RankingRankingReader<R> {
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
