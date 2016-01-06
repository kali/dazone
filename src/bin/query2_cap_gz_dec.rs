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
                               .help("pick a set (tiny, 1node, 5nodes)")
                               .takes_value(true))
                      .get_matches();
    let set = matches.value_of("SET").unwrap_or("5nodes");
    scan(set).unwrap();
}

fn scan(set: &str) -> Dx16Result<()> {
    let bibi = dx16::bibi_cap_gz_dec(set, "uservisits");
    let result = map_reduce(|reader: Dx16Result<Reader<OwnedSegments>>| {
                                let reader = reader.unwrap();
                                let visit: ::dx16::cap::user_visits::Reader = reader.get_root()
                                                                                    .unwrap();
                                let mut chars = [b' '; 8];
                                for (i, v) in visit.get_source_i_p()
                                                   .unwrap()
                                                   .bytes()
                                                   .take(chars.len())
                                                   .enumerate() {
                                    chars[i] = v;
                                }
                                Emit::One(chars, visit.get_ad_revenue())
                            },
                            |a, b| *a + *b,
                            bibi);
    println!("{} groups", result.len());

    Ok(())
}
