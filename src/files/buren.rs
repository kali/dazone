use std::marker::PhantomData;
use std::path;
use files::compressor::Compressor;

use buren::PartialDeserializer;
use serde::Deserialize;

/*
pub struct BurenReader<T> {
    deser: PartialDeserializer,
    phantom: PhantomData<T>
}

impl<T> BurenReader<T> {
    pub fn new(dir:path::PathBuf, compressor:Compressor, columns: &[usize]) -> BurenReader<T>
    {
        BurenReader { deser: PartialDeserializer::new(dir, compressor, columns), phantom: PhantomData }
    }
}

impl<T> Iterator for BurenReader<T> where T : Deserialize {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match <Self::Item as Deserialize>::deserialize(&mut self.deser) {
            Ok(it) => Some(it),
            _ => None
        }
    }
}
*/
