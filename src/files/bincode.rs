use std::marker::PhantomData;
use std::io;
use rustc_serialize::Decodable;

pub struct BincodeReader<R, T>
    where T: Send + Decodable + 'static,
          R: io::Read
{
    stream: io::BufReader<R>,
    phantom: PhantomData<T>,
}

impl<R, T> BincodeReader<R, T>
    where T: Send + Decodable + 'static,
          R: io::Read
{
    pub fn new(r: R) -> BincodeReader<R, T> {
        BincodeReader {
            stream: io::BufReader::new(r),
            phantom: PhantomData,
        }
    }
}

impl<R, T> Iterator for BincodeReader<R, T>
    where T: Send + Decodable + 'static,
          R: io::Read
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match ::bincode::rustc_serialize::decode_from(&mut self.stream, ::bincode::SizeLimit::Infinite) {
            Ok(it) => Some(it),
            Err(::bincode::rustc_serialize::DecodingError::InvalidEncoding(_)) => None,
            Err(e) => panic!(e)
        }
    }
}
