extern crate dx16;
extern crate glob;
extern crate simple_parallel;
extern crate num_cpus;
extern crate rmp;
extern crate flate2;

use std::fs;
use std::path;
use std::io::{ BufRead, BufReader, BufWriter };

use flate2::{ Compression, FlateWriteExt };
use rmp::encode::{ write_u32, write_str };

use dx16::{ Dx16Result };

fn main() {
    let set = "5nodes";
    pack(set, "rankings").unwrap();
}

fn pack(set:&str, table:&str) -> Dx16Result<()> {
    let source_root = dx16::data_dir_for("csv",set,table);
    let target_root = dx16::data_dir_for("rmp-gz",set,table);
    let _ = fs::remove_dir_all(target_root.clone());
    try!(fs::create_dir_all(target_root.clone()));
    let glob = source_root.clone() + "/*.csv";
    let jobs:Dx16Result<Vec<(path::PathBuf,path::PathBuf)>> =
        try!(::glob::glob(&glob)).map( |entry| {
            let entry:String = try!(entry).to_str().unwrap().to_string();
            let target =
                target_root.clone()
                + &entry[source_root.len() .. entry.find(".").unwrap()]
                + ".rmp.gz";
            Ok((path::PathBuf::from(&*entry), path::PathBuf::from(&target)))
    }).collect();
    let jobs = try!(jobs);
    let mut pool = simple_parallel::Pool::new(1+num_cpus::get());
    let task = |job:(path::PathBuf,path::PathBuf)| -> Dx16Result<()> {
        let input = try!(fs::File::open(job.0));
        let reader = BufReader::new(input);
        let mut output = BufWriter::new(try!(fs::File::create(job.1))).gz_encode(Compression::Default);
        for line in reader.lines() {
            let line = try!(line);
            let mut tokens = line.split(",");
            let url = tokens.next().unwrap();
            let pagerank:u32 = try!(tokens.next().unwrap().parse());
            let duration:u32 = try!(tokens.next().unwrap().parse());
            try!(write_str(&mut output, url));
            try!(write_u32(&mut output, pagerank));
            try!(write_u32(&mut output, duration));
        }
        Ok(())
    };
    let result:Dx16Result<Vec<()>> = unsafe { pool.map(jobs, &task).collect() };
    try!(result);
    Ok(())
}
