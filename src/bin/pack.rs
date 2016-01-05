extern crate dx16;
extern crate glob;
extern crate simple_parallel;
extern crate num_cpus;
extern crate csv;
extern crate rmp;
extern crate rmp_serialize;
extern crate rustc_serialize;
extern crate flate2;

use std::fs;
use std::path;
use std::io::BufWriter;
use std::io::Read;

use rmp_serialize::Encoder;

use flate2::{Compression, FlateWriteExt};

use rustc_serialize::{Encodable, Decodable};

use dx16::Dx16Result;
use dx16::data;

fn main() {
    let set = "5nodes";
    let table = ::std::env::args().nth(1).expect("please specify some table");
    match &*table {
        "rankings" => pack::<data::Ranking>(set, "rankings").unwrap(),
        "uservisits" => pack::<data::UserVisits>(set, "uservisits").unwrap(),
        t => panic!("unknwon table {}", &*t),
    }
}

fn pack<T: Decodable + Encodable>(set: &str, table: &str) -> Dx16Result<()> {
    let source_root = dx16::data_dir_for("text-deflate", set, table);
    let target_root = dx16::data_dir_for("rmp-gz", set, table);
    let _ = fs::remove_dir_all(target_root.clone());
    try!(fs::create_dir_all(target_root.clone()));
    let glob = source_root.clone() + "/*.deflate";
    let jobs: Dx16Result<Vec<(path::PathBuf, path::PathBuf)>> =
        try!(::glob::glob(&glob))
            .map(|entry| {
                let entry: String = try!(entry).to_str().unwrap().to_string();
                let target = target_root.clone() +
                             &entry[source_root.len()..entry.find(".").unwrap()] +
                             ".rmp.gz";
                Ok((path::PathBuf::from(&*entry), path::PathBuf::from(&target)))
            })
            .collect();
    let jobs = try!(jobs);
    let mut pool = simple_parallel::Pool::new(2 * num_cpus::get());
    let task = |job: (path::PathBuf, path::PathBuf)| -> Dx16Result<()> {
        let mut cmd: ::std::process::Child = ::std::process::Command::new("./zpipe.sh")
                                                 .arg(job.0.clone())
                                                 .stdout(::std::process::Stdio::piped())
                                                 .spawn()
                                                 .unwrap();
        {
            let mut output = cmd.stdout.as_mut().unwrap();
            let mut reader = csv::Reader::from_reader(output.by_ref()).has_headers(false);
            let mut output = BufWriter::new(try!(fs::File::create(job.1)))
                                 .gz_encode(Compression::Default);
            let mut coder = Encoder::new(&mut output);
            for item in reader.decode() {
                let item: T = item.unwrap();
                item.encode(&mut coder).unwrap();
            }
        }
        cmd.wait().unwrap();
        Ok(())
    };
    let result: Dx16Result<Vec<()>> = unsafe { pool.map(jobs, &task).collect() };
    try!(result);
    Ok(())
}
