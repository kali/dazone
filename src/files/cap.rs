extern crate capnp;

use std::{io, fs};

use self::capnp::serialize_packed;
use self::capnp::serialize::OwnedSegments;
use self::capnp::message::Reader;

use super::flate2::FlateReadExt;

use crunch::BI;

use {Dx16Error, Dx16Result};

pub struct CapReader<R: io::Read> {
    options: capnp::message::ReaderOptions,
    stream: io::BufReader<R>,
}

impl<R: io::Read> CapReader<R> {
    pub fn new(input: R) -> CapReader<R> {
        CapReader {
            options: capnp::message::ReaderOptions::new(),
            stream: io::BufReader::new(input),
        }
    }
}

impl<R: io::Read> Iterator for CapReader<R> {
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

pub fn bibi<'a, 'b>(set: &str, table: &str) -> BI<'a, BI<'b, Dx16Result<Reader<OwnedSegments>>>> {
    Box::new(super::files_for_format(set, table, "cap")
                 .into_iter()
                 .map(|f| -> BI<Dx16Result<Reader<OwnedSegments>>> {
                     let file = fs::File::open(f).unwrap();
                     Box::new(CapReader::new(file))
                 }))
}

pub fn bibi_gz<'a, 'b>(set: &str,
                       table: &str)
                       -> BI<'a, BI<'b, Dx16Result<Reader<OwnedSegments>>>> {
    Box::new(super::files_for_format(set, table, "cap-gz")
                 .into_iter()
                 .map(|f| -> BI<Dx16Result<Reader<OwnedSegments>>> {
                     let file = fs::File::open(f).unwrap();
                     Box::new(CapReader::new(file.gz_decode().unwrap()))
                 }))
}

pub fn bibi_gz_fork<'a, 'b>(set: &str,
                            table: &str)
                            -> BI<'a, BI<'b, Dx16Result<Reader<OwnedSegments>>>> {
    Box::new(super::files_for_format(set, table, "cap-gz")
                 .into_iter()
                 .map(|f| -> BI<Dx16Result<Reader<OwnedSegments>>> {
                     Box::new(CapReader::new(super::gz_read(f)))
                 }))
}
