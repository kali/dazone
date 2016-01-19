
use std::io;

use capnp;
use capnp::serialize_packed;
use capnp::serialize::OwnedSegments;
use capnp::message::Reader;

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
    type Item = Reader<OwnedSegments>;

    fn next(&mut self) -> Option<Reader<OwnedSegments>> {
        match serialize_packed::read_message(&mut self.stream, self.options) {
            Ok(msg) => Some(msg),
            Err(err) => {
                use std::error::Error;
                if err.description().contains("Premature EOF") {
                    return None;
                } else {
                    panic!(err)
                }
            }
        }
    }
}
