use std::{io, path};
use serde::ser;
use serde::ser::{Serialize, SeqVisitor, MapVisitor};

use byteorder::{LittleEndian, WriteBytesExt};

pub struct Serializer<W,C> where W: io::Write, C: Fn(usize)->W {
    pub streams: Vec<W>,
    pub create: C,
    pub inited: bool,
    pub index: usize,
}

impl<W,C> Serializer<W,C> where W: io::Write, C: Fn(usize)->W {
    pub fn new(f:C) -> Serializer<W,C> {
        Serializer { create:f, streams: vec![], inited:false, index:0 }
    }
}

impl<W,C> ser::Serializer for Serializer<W,C> where W: io::Write, C: Fn(usize)->W{
    type Error = io::Error;

    fn visit_bool(&mut self, v: bool) -> Result<(), Self::Error> {
        panic!("not implemented");
    }
    fn visit_i64(&mut self, v: i64) -> Result<(), Self::Error> {
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
        try!(s.write(value.as_bytes()));
        Ok(())
    }
    fn visit_unit(&mut self) -> Result<(), Self::Error> {
        panic!("not implemented");
    }
    fn visit_none(&mut self) -> Result<(), Self::Error> {
        panic!("not implemented");
    }
    fn visit_some<V>(&mut self, value: V) -> Result<(), Self::Error> where V: Serialize {
        panic!("not implemented");
    }
    fn visit_seq<V>(&mut self, visitor: V) -> Result<(), Self::Error> where V: SeqVisitor {
        panic!("not implemented");
    }
    fn visit_seq_elt<T>(&mut self, value: T) -> Result<(), Self::Error> where T: Serialize {
        panic!("not implemented");
    }
    fn visit_map<V>(&mut self, mut visitor: V) -> Result<(), Self::Error> where V: MapVisitor {
        while let Some(()) = try!(visitor.visit(self)) { }
        self.inited = true;
        Ok(())
    }
    fn visit_map_elt<K, V>(&mut self, key: K, value: V) -> Result<(), Self::Error> where K: Serialize, V: Serialize {
        if ! self.inited {
            let l = self.streams.len();
            self.streams.push((self.create)(l));
        }
        try!(value.serialize(self));
        Ok(())
    }
}
