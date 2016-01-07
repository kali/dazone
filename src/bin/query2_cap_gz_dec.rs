#![feature(iter_arith)]
extern crate dx16;
extern crate glob;
extern crate capnp;
extern crate clap;

use dx16::Dx16Result;
use dx16::mapred::*;

use capnp::serialize::OwnedSegments;
use capnp::message::Reader;

use clap::{Arg, App};

fn main() {
    let matches = App::new("pack")
                      .about("query 2")
                      .arg(Arg::with_name("SET")
                               .short("s")
                               .long("set")
                               .help("pick a set (tiny, 1node, or 5nodes)")
                               .takes_value(true))
                      .arg(Arg::with_name("LENGTH")
                               .short("l")
                               .long("length")
                               .help("length (8, 10, or 12)")
                               .takes_value(true))
                      .arg(Arg::with_name("REDUCE")
                               .short("r")
                               .long("reduce")
                               .help("reduce strategy: mono multi")
                               .takes_value(true))
                      .arg(Arg::with_name("REDUCE PAR")
                               .short("p")
                               .long("parallelism")
                               .help("multi reduce par factor")
                               .takes_value(true))
                      .get_matches();
    let set = matches.value_of("SET").unwrap_or("5nodes");
    let length: usize = matches.value_of("LENGTH").unwrap_or("8").parse().unwrap();
    let par: usize = matches.value_of("REDUCE PAR").unwrap_or("16").parse().unwrap();
    match matches.value_of("REDUCE").unwrap_or("multi") {
        "multi" => multi(set, length, par).unwrap(),
        "mono" => mono(set, length).unwrap(),
        s => panic!("unknown strategy {}", s),
    }
}

fn mapper(length: usize, reader: Dx16Result<Reader<OwnedSegments>>) -> Emit<[u8; 12], f32> {
    let reader = reader.unwrap();
    let visit: ::dx16::cap::user_visits::Reader = reader.get_root()
                                                        .unwrap();
    let mut chars = [b' '; 12];
    for (i, v) in visit.get_source_i_p()
                       .unwrap()
                       .bytes()
                       .take(::std::cmp::min(chars.len(), length))
                       .enumerate() {
        chars[i] = v;
    }
    Emit::One(chars, visit.get_ad_revenue())
}

fn mono(set: &str, length: usize) -> Dx16Result<()> {
    let bibi = dx16::bibi_cap_gz_dec(set, "uservisits");
    let result = map_reduce(|r| mapper(length, r), |a, b| a + b, bibi);
    println!("{} groups", result.len());

    Ok(())
}

fn multi(set: &str, length: usize, par: usize) -> Dx16Result<()> {
    let bibi = dx16::bibi_cap_gz_dec(set, "uservisits");
    let result = map_par_reduce(|r| mapper(length, r), |a, b| a + b, par, bibi);
    let len: usize = result.iter().map(|h| h.len()).sum();
    println!("{} groups", len);

    Ok(())
}
