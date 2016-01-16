extern crate rmp;
extern crate rmp_serialize;
extern crate rustc_serialize;

use std::marker::PhantomData;
use std::io;

use self::rustc_serialize::Decodable;
use self::rmp_serialize::decode::Decoder;

use crunch::BI;
use Dx16Result;
use Dx16Error;

pub struct RMPReader<R, T>
    where R: io::Read,
          T: Decodable + Send
{
    stream: Decoder<io::BufReader<R>>,
    phantom: PhantomData<T>,
}

impl<R, T> RMPReader<R, T>
    where R: io::Read,
          T: Decodable + Send
{
    pub fn new(file: R) -> RMPReader<R, T> {
        RMPReader {
            stream: Decoder::new(io::BufReader::new(file)),
            phantom: PhantomData,
        }
    }
}

impl<R, T> Iterator for RMPReader<R, T>
    where R: io::Read,
          T: Decodable + Send
{
    type Item = Dx16Result<T>;

    fn next(&mut self) -> Option<Dx16Result<T>> {
        use self::rmp_serialize::decode::Error::InvalidMarkerRead;
        use self::rmp::decode::ReadError;
        let res: Result<T, _> = Decodable::decode(&mut self.stream);
        match res {
            Err(InvalidMarkerRead(ReadError::UnexpectedEOF)) => None,
            Err(a) => Some(Err(Dx16Error::from(a))),
            Ok(r) => Some(Ok(r)),
        }
    }
}

pub fn bibi_gz<'a, 'b, T>(set: &str, table: &str) -> BI<'a, BI<'b, Dx16Result<T>>>
    where T: Decodable + 'static + Send
{
    Box::new(super::files_for_format(set, table, "rmp-gz")
                 .into_iter()
                 .map(|f| -> BI<Dx16Result<T>> { Box::new(RMPReader::new(super::gz_read(f))) }))
}

pub struct RankingRMPReader<R: io::Read> {
    stream: io::BufReader<R>,
    buf: [u8; 1024],
}

impl<R: io::Read> RankingRMPReader<R> {
    pub fn new(r: R) -> RankingRMPReader<R> {
        RankingRMPReader {
            stream: io::BufReader::new(r),
            buf: [0u8; 1024],
        }
    }
}
impl<R: io::Read> Iterator for RankingRMPReader<R> {
    type Item = Dx16Result<u32>;

    fn next(&mut self) -> Option<Dx16Result<u32>> {
        use self::rmp::decode::*;
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
