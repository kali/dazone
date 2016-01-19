use std::marker::PhantomData;
use std::io;
use rustc_serialize::Decodable;
use csv;

pub struct CSVReader<R, T>
    where T: Send + Decodable + 'static,
          R: io::Read
{
    stream: csv::Reader<R>,
    phantom: PhantomData<T>,
}

impl<R, T> CSVReader<R, T>
    where T: Send + Decodable + 'static,
          R: io::Read
{
    pub fn new(r: R) -> CSVReader<R, T> {
        CSVReader {
            stream: csv::Reader::from_reader(r),
            phantom: PhantomData,
        }
    }
}

impl<R, T> Iterator for CSVReader<R, T>
    where T: Send + Decodable + 'static,
          R: io::Read
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.stream.decode().next().map(|it| it.unwrap())
    }
}
