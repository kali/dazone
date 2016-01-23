
use std::io;

use capnp;
use capnp::{ serialize, serialize_packed};
use capnp::serialize::OwnedSegments;
use capnp::message::Reader;

use data::cap::Mode;

pub struct CapReader<R: io::Read> {
    options: capnp::message::ReaderOptions,
    stream: io::BufReader<R>,
    mode: Mode
}

impl<R: io::Read> CapReader<R> {
    pub fn new(input: R, mode:Mode) -> CapReader<R> {
        CapReader {
            options: capnp::message::ReaderOptions::new(),
            stream: io::BufReader::new(input),
            mode: mode
        }
    }
}

impl<R: io::Read> Iterator for CapReader<R> {
    type Item = Reader<OwnedSegments>;

    fn next(&mut self) -> Option<Reader<OwnedSegments>> {
        let it = if self.mode == Mode::Packed {
            serialize_packed::read_message(&mut self.stream, self.options)
        } else {
            serialize::read_message(&mut self.stream, self.options)
        };
        match it {
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
