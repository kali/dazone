use std::{io, path};
use serde::{ ser };
use serde::ser::{Serialize, SeqVisitor, MapVisitor};
use serde::de::{ Deserializer};

use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};

use files::compressor::Compressor;

pub struct Serializer {
    pub dir: path::PathBuf,
    pub streams: Vec<Box<io::Write>>,
    pub compressor: Compressor,
    inited: bool,
    index: usize,
}

impl Serializer {
    pub fn new(dir: path::PathBuf, compressor:Compressor) -> Serializer {
        Serializer { dir:dir, compressor:compressor, streams: vec![], inited:false, index:0 }
    }
}

impl ser::Serializer for Serializer {
    type Error = io::Error;

    fn visit_bool(&mut self, _v: bool) -> Result<(), Self::Error> {
        panic!("not implemented");
    }
    fn visit_i64(&mut self, _v: i64) -> Result<(), Self::Error> {
        panic!("not implemented");
    }
    fn visit_u64(&mut self, v: u64) -> Result<(), Self::Error> {
        let s = &mut self.streams[self.index];
        self.index += 1;
        try!(s.write_u64::<LittleEndian>(v));
        Ok(())
    }
    fn visit_f64(&mut self, v: f64) -> Result<(), Self::Error> {
        let s = &mut self.streams[self.index];
        self.index += 1;
        try!(s.write_f64::<LittleEndian>(v));
        Ok(())
    }
    fn visit_str(&mut self, value: &str) -> Result<(), Self::Error> {
        let s = &mut self.streams[self.index];
        self.index += 1;
        try!(s.write_u64::<LittleEndian>(value.len() as u64));
        try!(s.write_all(value.as_bytes()));
        Ok(())
    }
    fn visit_unit(&mut self) -> Result<(), Self::Error> {
        panic!("not implemented");
    }
    fn visit_none(&mut self) -> Result<(), Self::Error> {
        panic!("not implemented");
    }
    fn visit_some<V>(&mut self, _value: V) -> Result<(), Self::Error> where V: Serialize {
        panic!("not implemented");
    }
    fn visit_seq<V>(&mut self, _visitor: V) -> Result<(), Self::Error> where V: SeqVisitor {
        panic!("not implemented");
    }
    fn visit_seq_elt<T>(&mut self, _value: T) -> Result<(), Self::Error> where T: Serialize {
        panic!("not implemented");
    }
    fn visit_map<V>(&mut self, mut visitor: V) -> Result<(), Self::Error> where V: MapVisitor {
        while let Some(()) = try!(visitor.visit(self)) { }
        self.inited = true;
        self.index = 0;
        Ok(())
    }
    fn visit_map_elt<K, V>(&mut self, _key: K, value: V) -> Result<(), Self::Error> where K: Serialize, V: Serialize {
        if !self.inited {
            let mut file = self.dir.clone();
            file.push(format!("col-{:03}", self.index));
            self.streams.push(self.compressor.write_file(file));
        }
        try!(value.serialize(self));
        Ok(())
    }
}

pub struct PartialDeserializer {
    pub streams: Vec<Box<io::Read+Send>>,
}

impl PartialDeserializer {
    pub fn new(dir:path::PathBuf, compressor:Compressor, columns: &[usize]) -> PartialDeserializer
        {
            PartialDeserializer {
                streams: columns.iter().map(|col| {
                    let mut file = dir.clone();
                    file.push(format!("col-{:03}", col));
                    compressor.read_file(file)
                }).collect()
            }
        }

    fn read_string(&mut self, field:usize) -> io::Result<String> {
        let l = try!(self.streams[field].read_u64::<LittleEndian>()) as usize;
        let mut bytes = Vec::with_capacity(l);
        unsafe { bytes.set_len(l) };
        try!(self.streams[field].read_exact(&mut bytes));
        Ok(unsafe { ::std::mem::transmute(bytes) })
    }

    fn read_f64(&mut self, field:usize) -> io::Result<f64> {
        let f = try!(self.streams[field].read_f64::<LittleEndian>());
        Ok(f)
    }
}

impl Iterator for PartialDeserializer {
    type Item=(String, f32);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.read_string(0), self.read_f64(1)) {
            (Ok(s), Ok(f)) => Some((s,f as f32)),
            _ => None
        }
    }

}
