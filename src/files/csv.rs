extern crate csv;
extern crate rustc_serialize;

use std::marker::PhantomData;
use std::{io, path};
use self::rustc_serialize::Decodable;

use crunch::BI;
use Dx16Result;
use std::fs::File;

use super::flate2::FlateReadExt;

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
    fn new(r: R) -> CSVReader<R, T> {
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
    type Item = Dx16Result<T>;

    fn next(&mut self) -> Option<Dx16Result<T>> {
        self.stream.decode().next().map(|it| Ok(try!(it)))
    }
}

pub fn bibi<'a, 'b, T>(set: &str, table: &str) -> BI<'a, BI<'b, Dx16Result<T>>>
    where T: Decodable + 'static + Send
{
    let source_root = super::data_dir_for("text-deflate", set, table);
    let glob = source_root.clone() + "/*.deflate";
    let files: Vec<path::PathBuf> = super::glob::glob(&glob)
                                        .unwrap()
                                        .map(|p| p.unwrap().to_owned())
                                        .collect();
    Box::new(files.into_iter()
                  .map(|f| -> BI<Dx16Result<T>> {
                      Box::new(CSVReader::new(File::open(f).unwrap().zlib_decode()))
                  }))
}
