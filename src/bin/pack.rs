extern crate dazone;

extern crate bincode;
extern crate cbor;
extern crate clap;
extern crate csv;
extern crate flate2;
extern crate num_cpus;
extern crate pbr;
extern crate rmp;
extern crate rmp_serialize;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
extern crate simple_parallel;
extern crate snappy_framed;

use std::{fs, io, path};
use std::io::BufWriter;

use std::fmt::Debug;

use std::sync::Mutex;

use flate2::{Compression, FlateWriteExt};

use rustc_serialize::{Encodable, Decodable};

use snappy_framed::write::SnappyFramedEncoder;

use dazone::Dx16Result;
use dazone::data;
use dazone::data::cap::{ Capitanable, Mode };
use dazone::data::pbuf::Protobufable;
use serde::Serialize;

use pbr::ProgressBar;

use clap::{Arg, App};

fn main() {
    let matches = App::new("pack")
                      .about("repack data to better formats")
                      .arg(Arg::with_name("SET")
                               .short("s")
                               .long("set")
                               .help("pick a set (tiny, 1node, 5nodes)")
                               .takes_value(true))
                      .arg(Arg::with_name("TABLE")
                               .index(1)
                               .required(true)
                               .help("table to process"))
                      .arg(Arg::with_name("FORMAT")
                               .index(2)
                               .required(true)
                               .help("cap-gz or rmp-gz"))
                      .get_matches();
    let set = matches.value_of("SET").unwrap_or("5nodes");
    let table = matches.value_of("TABLE").unwrap();
    let dst = matches.value_of("FORMAT").unwrap();
    if dst == "text-deflate" {
        panic!("text-deflate can not be used as the output format!");
    }
    match &*table {
        "rankings" => loop_files::<data::pod::Ranking>(set, "rankings", &*dst).unwrap(),
        "uservisits" => loop_files::<data::pod::UserVisits>(set, "uservisits", &*dst).unwrap(),
        t => panic!("unknwon table {}", &*t),
    }
}

fn loop_files<T>(set: &str, table: &str, dst: &str) -> Dx16Result<()>
    where T: Decodable + Encodable + Capitanable + Debug + Protobufable + Serialize
{
    let source_root = dazone::files::data_dir_for("text-deflate", set, table);
    let target_root = dazone::files::data_dir_for(dst, set, table);
    let _ = fs::remove_dir_all(target_root.clone());
    try!(fs::create_dir_all(target_root.clone()));
    let jobs: Dx16Result<Vec<(path::PathBuf, path::PathBuf)>> =
        dazone::files::files_for_format(set, table, "text-deflate")
            .iter()
            .map(|entry| {
                let entry: String = entry.to_str().unwrap().to_string();
                let target = target_root.clone() +
                             &entry[source_root.len()..entry.find(".").unwrap()] +
                             "." + dst;
                Ok((path::PathBuf::from(&*entry), path::PathBuf::from(&target)))
            })
            .collect();
    let jobs = try!(jobs);

    let pb = Mutex::new(ProgressBar::new(jobs.len()));
    let mut pool = simple_parallel::Pool::new(/*2 * num_cpus::get()*/ 1);
    let task = |job: (path::PathBuf, path::PathBuf)| -> Dx16Result<()> {
        let input = flate2::FlateReadExt::zlib_decode(fs::File::open(job.0.clone()).unwrap());
        let mut reader = csv::Reader::from_reader(input).has_headers(false);

        let tokens: Vec<&str> = dst.split("-").collect();

        fn compress<P:AsRef<path::Path>>(tokens:&Vec<&str>, p:P) -> Box<io::Write> {
            let file = fs::File::create(p).unwrap();
            if tokens.len() == 1 {
                Box::new(BufWriter::new(file))
            } else if tokens[1] == "gz" {
                Box::new(file.gz_encode(Compression::Default))
            } else if tokens[1] == "snz" {
                Box::new(io::BufWriter::with_capacity(64 * 1024,
                                                      SnappyFramedEncoder::new(file).unwrap()))
            } else {
                panic!("unknown compression {}", tokens[1]);
            }
        }

        match tokens[0] {
            "bincode" => {
                let mut compressed = compress(&tokens, job.1);
                for item in reader.decode() {
                    let item: T = item.unwrap();
                    bincode::rustc_serialize::encode_into(&item,
                                                          &mut compressed,
                                                          bincode::SizeLimit::Infinite)
                        .unwrap();
                }
            }
            "buren" => {
                fs::create_dir_all(job.1.clone()).unwrap();
                let mut coder = ::dazone::buren::Serializer::new(|col| {
                    let mut file = job.1.clone();
                    file.push(format!("col-{}", col));
                    compress(&tokens, file)
                });
                for item in reader.decode() {
                    let item: T = item.unwrap();
                    item.serialize(&mut coder).unwrap();
                }
            }
            "cap" => {
                let mut compressed = compress(&tokens, job.1);
                for item in reader.decode() {
                    let item: T = item.unwrap();
                    item.write_to_cap(&mut compressed, Mode::Unpacked).unwrap();
                }
            }
            "cbor" => {
                let mut compressed = compress(&tokens, job.1);
                let mut coder = ::cbor::Encoder::from_writer(&mut compressed);
                for item in reader.decode() {
                    let item: T = item.unwrap();
                    item.encode(&mut coder).unwrap();
                }
            }
            "csv" => {
                let compressed = compress(&tokens, job.1);
                let mut coder = ::csv::Writer::from_writer(compressed);
                for item in reader.decode() {
                    let item: T = item.unwrap();
                    coder.encode(item).unwrap();
                }
            }
            "json" => {
                let mut compressed = compress(&tokens, job.1);
                for item in reader.decode() {
                    let item: T = item.unwrap();
                    ::serde_json::ser::to_writer(&mut compressed, &item).unwrap();
                    write!(compressed, "\n").unwrap();
                }
            }
            "mcap" => {
                let mut compressed = compress(&tokens, job.1);
                for item in reader.decode() {
                    let item: T = item.unwrap();
                    item.write_to_cap(&mut compressed, Mode::Mappable).unwrap();
                }
            }
            "pbuf" => {
                let mut compressed = compress(&tokens, job.1);
                for item in reader.decode() {
                    let item: T = item.unwrap();
                    item.write_to_pbuf(&mut compressed).unwrap();
                }
            }
            "pcap" => {
                let mut compressed = compress(&tokens, job.1);
                for item in reader.decode() {
                    let item: T = item.unwrap();
                    item.write_to_cap(&mut compressed, Mode::Packed).unwrap();
                }
            }
            "rmp" => {
                let mut compressed = compress(&tokens, job.1);
                let mut coder = ::rmp_serialize::Encoder::new(&mut compressed);
                for item in reader.decode() {
                    let item: T = item.unwrap();
                    item.encode(&mut coder).unwrap();
                }
            }
            any => panic!("unknown format {}", any),
        }
        pb.lock().unwrap().inc();
        Ok(())
    };
    let result: Dx16Result<Vec<()>> = unsafe { pool.map(jobs, &task).collect() };
    try!(result);
    Ok(())
}
