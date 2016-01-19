extern crate dazone;
extern crate glob;
extern crate simple_parallel;
extern crate num_cpus;
extern crate csv;
extern crate rmp;
extern crate rmp_serialize;
extern crate rustc_serialize;
extern crate flate2;
extern crate pbr;
extern crate clap;
extern crate snappy_framed;

use std::{ fs, io, path };
use std::io::BufWriter;

use std::sync::Mutex;

use rmp_serialize::Encoder;

use flate2::{Compression, FlateWriteExt};

use rustc_serialize::{Encodable, Decodable};

use snappy_framed::write::SnappyFramedEncoder;

use dazone::Dx16Result;
use dazone::data;
use dazone::data::cap::Capitanable;

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
    match &*table {
        "rankings" => loop_files::<data::pod::Ranking>(set, "rankings", &*dst).unwrap(),
        "uservisits" => loop_files::<data::pod::UserVisits>(set, "uservisits", &*dst).unwrap(),
        t => panic!("unknwon table {}", &*t),
    }
}

fn loop_files<T>(set: &str, table: &str, dst: &str) -> Dx16Result<()>
    where T: Decodable + Encodable + Capitanable
{
    let source_root = dazone::files::data_dir_for("text-deflate", set, table);
    let target_root = dazone::files::data_dir_for(dst, set, table);
    let _ = fs::remove_dir_all(target_root.clone());
    try!(fs::create_dir_all(target_root.clone()));
    let glob = source_root.clone() + "/*.deflate";
    let jobs: Dx16Result<Vec<(path::PathBuf, path::PathBuf)>> =
        try!(::glob::glob(&glob))
            .map(|entry| {
                let entry: String = try!(entry).to_str().unwrap().to_string();
                let target = target_root.clone() +
                             &entry[source_root.len()..entry.find(".").unwrap()] +
                             "." + dst;
                Ok((path::PathBuf::from(&*entry), path::PathBuf::from(&target)))
            })
            .collect();
    let jobs = try!(jobs);

    let pb = Mutex::new(ProgressBar::new(jobs.len()));
    let mut pool = simple_parallel::Pool::new(2 * num_cpus::get());
    let task = |job: (path::PathBuf, path::PathBuf)| -> Dx16Result<()> {
        let input = flate2::FlateReadExt::zlib_decode(fs::File::open(job.0).unwrap());
        let mut reader = csv::Reader::from_reader(input).has_headers(false);
        let tokens:Vec<&str> = dst.split("-").collect();

        let file = fs::File::create(job.1).unwrap();
        let mut compressed:Box<io::Write> = if tokens.len() == 1 {
            Box::new(BufWriter::new(file))
        } else if tokens[1] == "gz" {
            Box::new(file.gz_encode(Compression::Default))
        } else if tokens[1] == "snz" {
            Box::new(SnappyFramedEncoder::new(file).unwrap())
        } else {
            panic!("unknown compression {}", tokens[1]);
        };

        match tokens[0] {
            "cap" => {
                for item in reader.decode() {
                    let item: T = item.unwrap();
                    item.write_to_cap(&mut compressed).unwrap();
                }
            }
            "rmp" => {
                let mut coder = Encoder::new(&mut compressed);
                for item in reader.decode() {
                    let item: T = item.unwrap();
                    item.encode(&mut coder).unwrap();
                }
            }
            "csv" => {
                let mut coder = ::csv::Writer::from_writer(compressed);
                for item in reader.decode() {
                    let item: T = item.unwrap();
                    coder.encode(item).unwrap();
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
