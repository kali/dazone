extern crate dx16;
extern crate glob;
extern crate capnp;

use dx16::Dx16Result;
use dx16::mapred::*;

use capnp::serialize::OwnedSegments;
use capnp::message::Reader;

fn main() {
    let set = "5nodes";
    scan(set).unwrap();
}

fn scan(set: &str) -> Dx16Result<()> {
    let bibi = dx16::bibi_cap_gz_dec(set, "uservisits");
    let result = map_reduce(|reader: Dx16Result<Reader<OwnedSegments>>| {
                                let reader = reader.unwrap();
                                let visit: ::dx16::cap::user_visits::Reader = reader.get_root()
                                                                                    .unwrap();
                                Emit::One(visit.get_source_i_p()
                                               .unwrap()
                                               .chars()
                                               .take(8)
                                               .collect::<String>(),
                                          visit.get_ad_revenue())
                            },
                            |a, b| *a + *b,
                            bibi);
    println!("{} groups", result.len());

    Ok(())
}
