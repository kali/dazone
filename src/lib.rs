#![feature(iter_arith)]

extern crate glob;
extern crate simple_parallel;
extern crate num_cpus;
extern crate rmp;
#[macro_use]
extern crate quick_error;

pub mod mapred;

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
        DecodeString { }
    }
}

pub type Dx16Result<T> = Result<T, Dx16Error>;

pub fn data_dir_for(state: &str, set: &str, table: &str) -> String {
    format!("data/{}/{}/{}", state, set, table)
}

pub struct RankingReader<R: io::Read> {
    stream: io::BufReader<R>,
    buf: [u8; 1024],
}

impl <R:io::Read> RankingReader<R> {
    pub fn new(r: R) -> RankingReader<R> {
        RankingReader {
            stream: io::BufReader::new(r),
            buf: [0u8; 1024],
        }
    }
}

impl <R:io::Read> Iterator for RankingReader<R> {
    type Item = Dx16Result<u32>;

    fn next(&mut self) -> Option<Dx16Result<u32>> {
        use rmp::decode::*;
        match read_str(&mut self.stream, &mut self.buf) {
            Err(DecodeStringError::InvalidMarkerRead(_)) => return None,
            Err(a) => return Some(Err(Dx16Error::DecodeString)),
            _ => {}
        }
        let pagerank = read_u32(&mut self.stream);
        if pagerank.is_err() {
            return Some(Err(Dx16Error::DecodeString));
        }
        let duration = read_u32(&mut self.stream);
        if duration.is_err() {
            return Some(Err(Dx16Error::DecodeString));
        }
        Some(Ok(pagerank.unwrap()))
    }
}
