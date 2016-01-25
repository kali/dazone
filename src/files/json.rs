use std::marker::PhantomData;
use std::io;
use std::io::BufRead;
use serde_json;
use serde::Deserialize;

pub struct JsonReader<R, T>
    where T: Send + Deserialize + 'static,
          R: io::Read
{
    stream: io::Lines<io::BufReader<R>>,
    phantom: PhantomData<T>,
}

impl<R, T> JsonReader<R, T>
    where T: Send + Deserialize + 'static,
          R: io::Read
{
    pub fn new(r: R) -> JsonReader<R, T> {
        JsonReader {
            stream: io::BufReader::new(r).lines(),
            phantom: PhantomData,
        }
    }
}

impl<R, T> Iterator for JsonReader<R, T>
    where T: Send + Deserialize + 'static,
          R: io::Read
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.stream.next().map(|x|
            serde_json::de::from_str(&*x.unwrap()).unwrap()
        )
    }
}
