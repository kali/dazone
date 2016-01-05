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

use std::marker::PhantomData;

use rustc_serialize::Decodable;
use rmp_serialize::decode::Decoder;
use std::io;

use mapred::BI;

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

pub struct RMPReader<R: io::Read, T: Decodable> {
    stream: Decoder<io::BufReader<R>>,
    phantom: PhantomData<T>,
}

impl <R:io::Read, T:Decodable> RMPReader<R, T> {
    pub fn new(r: R) -> RMPReader<R, T> {
        RMPReader {
            stream: Decoder::new(io::BufReader::new(r)),
            phantom: PhantomData,
        }
    }
}

unsafe impl<R:io::Read, T:Decodable> Send for RMPReader<R,T> {}

impl <R:io::Read, T:Decodable> Iterator for RMPReader<R,T> {
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

pub fn bibi_rmp_gz_dec<'a, 'b, T>(set: &str, table: &str) -> BI<'a, BI<'b, Dx16Result<T>>>
    where T: Decodable + 'static
{
    let source_root = data_dir_for("rmp-gz", set, table);
    let glob = source_root.clone() + "/*.rmp.gz";
    Box::new(::glob::glob(&glob).unwrap().map(|f| -> BI<Dx16Result<T>> {
        let cmd = ::std::process::Command::new("gzcat")
                      .arg("-d")
                      .arg(f.unwrap())
                      .stdout(::std::process::Stdio::piped())
                      .spawn()
                      .unwrap();
        Box::new(RMPReader::new(cmd.stdout.unwrap()))
    }))
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
