use std::{fs,io};

use std::marker::PhantomData;

use capnp;
use capnp::{ serialize, serialize_packed, Word};
use capnp::serialize::{OwnedSegments, SliceSegments};
use capnp::message::Reader;

use data::cap::Mode;

use memmap::{ Mmap, Protection };

use byteorder::{ByteOrder, LittleEndian};

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

pub struct MmapReader<K>
        where K: ::short_bytes_array::ShortBytesArray {
    options: capnp::message::ReaderOptions,
    mmap:Mmap,
    position:usize,
    phantom:PhantomData<K>
}

impl<K> MmapReader<K>
        where K: ::short_bytes_array::ShortBytesArray {
    pub fn new<P>(f:P) -> MmapReader<K> where P:AsRef<::std::path::Path> {
        MmapReader {
            options: capnp::message::ReaderOptions::new(),
            mmap: Mmap::open_path(f, Protection::Read).unwrap(),
            position: 0,
            phantom:PhantomData
        }
    }
}

impl<K> Iterator for MmapReader<K>
        where K: ::short_bytes_array::ShortBytesArray {
    type Item = (K,f32);

    fn next(&mut self) -> Option<(K,f32)> {
        let slice = unsafe { self.mmap.as_slice() };
        if self.position == slice.len() {
            return None;
        }
        let len = LittleEndian::read_u64(&slice[self.position..]) as usize;
        self.position += ::std::mem::size_of::<u64>();
        let end = self.position + ::std::mem::size_of::<u64>() * len;
        let bytes = &slice[self.position..end];
        let words:&[Word] = Word::bytes_to_words(bytes);
        let msg = serialize::read_message_from_words(words, self.options).unwrap();
        self.position = end;
        let visit: ::capdata::user_visits::Reader = msg.get_root().unwrap();
        Some((K::prefix(visit.get_source_i_p().unwrap()), visit.get_ad_revenue()))
    }
}
