extern crate dx16;
extern crate glob;
extern crate simple_parallel;
extern crate num_cpus;
extern crate rmp;
extern crate flate2;

use dx16::{ Dx16Result };
use dx16::mapred;
use dx16::mapred::BI;

use std::fs::File;
use flate2::FlateReadExt;

fn main() {
    let set = "5nodes";
    scan(set, "rankings").unwrap();
}

fn scan(set:&str, table:&str) -> Dx16Result<()> {
    let source_root = dx16::data_dir_for("rmp-gz",set,table);
    let glob = source_root.clone() + "/*.rmp.gz";
    let pageranks:BI<BI<u32>> = Box::new(
        ::glob::glob(&glob).unwrap().map( |f| {

            let cmd = ::std::process::Command::new("gzcat")
                .arg("-d").arg(f.unwrap())
                .stdout(::std::process::Stdio::piped())
                .spawn().unwrap();
            let bi:BI<u32> = Box::new(::dx16::RankingReader::new(cmd.stdout.unwrap()).map(|u|u.unwrap()));

//            let bi:BI<u32> = Box::new(::dx16::RankingReader::new(File::open(f.unwrap()).unwrap().gz_decode().unwrap()).map(|u|u.unwrap()));
            bi
        })
    );
    let result = mapred::MapReduceOp::map_reduce(
        |i:u32| i>10,
        pageranks
    );
    println!("{:?}", result);
    Ok(())
}
