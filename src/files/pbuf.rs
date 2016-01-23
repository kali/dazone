use std::io;
use std::marker::PhantomData;

use protobuf::MessageStatic;

use byteorder::{ReadBytesExt, LittleEndian};

pub struct PBufReader<R,T>
    where T: Send + MessageStatic + 'static,
          R: io::Read {
    pub stream: R,
    pub phantom: PhantomData<T>,
}

impl<R,T> Iterator for PBufReader<R,T>
    where T: Send + MessageStatic + 'static,
          R: io::Read {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match self.stream.read_u16::<LittleEndian>() {
            Ok(len) => {
                let mut bytes = Vec::with_capacity(len as usize);
                unsafe { bytes.set_len(len as usize); }
                self.stream.read_exact(&mut *bytes).unwrap();
                Some(::protobuf::core::parse_from_bytes(&* bytes).unwrap())
            }
            Err(_) => None
        }
    }
}
