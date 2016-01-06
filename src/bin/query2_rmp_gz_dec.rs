extern crate dx16;
extern crate glob;
extern crate capnp;

use dx16::Dx16Result;
use dx16::mapred::*;

fn main() {
    let set = "5nodes";
    scan(set).unwrap();
}

fn scan(set: &str) -> Dx16Result<()> {
    let bibi = dx16::rmp_read::bibi_rmp_gz_dec(set, "uservisits");
    let result = map_reduce(|reader: Dx16Result<::dx16::data::UserVisits>| {
                                let reader = reader.unwrap();
                                Emit::One(reader.source_ip.chars().take(8).collect::<String>(),
                                          reader.ad_revenue)
                            },
                            |a, b| *a + *b,
                            bibi);
    println!("{} groups", result.len());

    Ok(())
}
