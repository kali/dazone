use std::marker::PhantomData;
use std::io;
use rustc_serialize::Decodable;
use cbor;

pub struct CborReader<R, T>
    where T: Send + Decodable + 'static,
          R: io::Read
{
    stream: cbor::Decoder<io::BufReader<R>>,
    phantom: PhantomData<T>,
}

impl<R, T> CborReader<R, T>
    where T: Send + Decodable + 'static,
          R: io::Read
{
    pub fn new(r: R) -> CborReader<R, T> {
        CborReader {
            stream: cbor::Decoder::from_reader(r),
            phantom: PhantomData,
        }
    }
}

impl<R, T> Iterator for CborReader<R, T>
    where T: Send + Decodable + 'static,
          R: io::Read
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.stream.decode().next().map(|it| it.unwrap())
    }
}
