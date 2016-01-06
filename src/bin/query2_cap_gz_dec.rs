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
                      .get_matches();
    let set = matches.value_of("SET").unwrap_or("5nodes");
    let length: usize = matches.value_of("LENGTH").unwrap_or("8").parse().unwrap();
    scan(set, length).unwrap();
}

fn scan(set: &str, length: usize) -> Dx16Result<()> {
    let bibi = dx16::bibi_cap_gz_dec(set, "uservisits");
    let result = map_reduce(|reader: Dx16Result<Reader<OwnedSegments>>| {
                                let reader = reader.unwrap();
                                let visit: ::dx16::cap::user_visits::Reader = reader.get_root()
                                                                                    .unwrap();
                                let mut chars = [b' '; 12];
                                for (i, v) in visit.get_source_i_p()
                                                   .unwrap()
                                                   .bytes()
                                                   .take(::std::cmp::max(chars.len(), length))
                                                   .enumerate() {
                                    chars[i] = v;
                                }
                                Emit::One(chars, visit.get_ad_revenue())
                            },
                            |a, b| a + b,
                            bibi);
    println!("{} groups", result.len());

    Ok(())
}
