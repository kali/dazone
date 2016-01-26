use std::marker::PhantomData;

use rustc_serialize::Decodable;

use capnp::serialize::OwnedSegments;
use capnp::message::Reader;

use data::cap::Mode;

use protobuf::MessageStatic;

use std::path;

use crunch::BI;

use serde::Deserialize;

pub mod bincode;
pub mod cap;
pub mod compressor;
pub mod rmp;
pub mod csv;
pub mod cbor;
pub mod json;
pub mod pbuf;

pub fn data_dir_for(state: &str, set: &str, table: &str) -> String {
    format!("data/{}/{}/{}", state, set, table)
}

pub fn files_for_format(set: &str, table: &str, format: &str) -> BI<'static, path::PathBuf> {
    let source_root = data_dir_for(format, set, table);
    let ext = if format == "text-deflate" {
        "deflate"
    } else {
        format
    };
    let glob = source_root.clone() + "/*." + ext;
    let mut vec: Vec<path::PathBuf> = ::glob::glob(&glob)
        .unwrap()
        .map(|p| p.unwrap().to_owned())
        .collect();
    vec.sort();
    Box::new(vec.into_iter())
}

pub fn bibi_pod<'a, 'b, T>(set: &str, table: &str, format: &str) -> BI<'a, BI<'b, T>>
where T: Decodable + Deserialize + Send + 'static
{
    let encoding:String = format.split("-").map(|x| x.to_string()).next().unwrap();
    let compressor = compressor::Compressor::for_format(format);
    Box::new(files_for_format(set, table, format).into_iter().map(move |f| {
        let f = compressor.read_file(f);
        let it: BI<T> = match &*encoding {
            "bincode" => Box::new(bincode::BincodeReader::new(f)),
            "cbor" => Box::new(cbor::CborReader::new(f)),
            "csv" | "text" => Box::new(csv::CSVReader::new(f)),
            "json" => Box::new(json::JsonReader::new(f)),
            "rmp" => Box::new(rmp::RMPReader::new(f)),
            any => panic!("unknown format {}", any),
        };
        it
    }))
}

pub fn bibi_cap<'a, 'b>(set: &str,
                        table: &str,
                        format: &str)
-> BI<'a, BI<'b, Reader<OwnedSegments>>> {
    let mode = if format.starts_with("cap") { Mode::Unpacked } else { Mode::Packed };
    let compressor = compressor::Compressor::for_format(format);
    Box::new(files_for_format(set, table, format)
             .map(move |f| -> BI<Reader<OwnedSegments>> {
                 Box::new(cap::CapReader::new(compressor.read_file(f), mode)) 
             }))
}

/*
   pub fn bibi_mcap<'a, 'b, 'c>(set: &str,
   table: &str,
   format: &str)
   -> BI<'a, BI<'b, Reader<SliceSegments<'c>>>> where 'b: 'c, 'a: 'b {
   assert!(format == "mcap");
   Box::new(files_for_format(set, table, format).iter()
   .map(move |f| -> BI<Reader<SliceSegments<'c>>> { Box::new(cap::MmapReader::new(&fs::File::open(f).unwrap())) }))
   }
   */

pub fn bibi_pbuf<'a, 'b, T>(set: &str,
                            table: &str,
                            format: &str)
-> BI<'a, BI<'b, T>> where T:MessageStatic+ Send{
    let compressor = compressor::Compressor::for_format(format);
    Box::new(files_for_format(set, table, format)
             .map(move |f| -> BI<T> { Box::new(pbuf::PBufReader{ stream: compressor.read_file(f), phantom:PhantomData}) }))
}
