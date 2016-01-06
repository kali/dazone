use std::marker::PhantomData;
use std::{io, path, process};

use rustc_serialize::Decodable;

use rmp;
use rmp_serialize::decode::Decoder;

use mapred::BI;
use Dx16Error;
use Dx16Result;


pub struct RMPReader<T: Decodable> {
    stream: Decoder<io::BufReader<::PipeReader>>,
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
            stream: Decoder::new(io::BufReader::new(::PipeReader { child: child })),
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
    let source_root = ::data_dir_for("rmp-gz", set, table);
    let glob = source_root.clone() + "/*.rmp-gz";
    let files: Vec<path::PathBuf> = ::glob::glob(&glob)
                                        .unwrap()
                                        .map(|p| p.unwrap().to_owned())
                                        .collect();
    Box::new(files.into_iter()
                  .map(|f| -> BI<Dx16Result<T>> { Box::new(RMPReader::new(&f)) }))
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
