extern crate dx16;
extern crate glob;
extern crate capnp;
extern crate capdata as cap;


use dx16::Dx16Result;

use capnp::serialize::OwnedSegments;
use capnp::message::Reader;

use dx16::mapred::FilterCountOp;

fn main() {
    let set = "5nodes";
    scan(set, "rankings").unwrap();
}

fn scan(set: &str, table: &str) -> Dx16Result<()> {
    let data = dx16::bibi_cap_gz_dec(set, table);
    let result = FilterCountOp::filter_count(|r: Dx16Result<Reader<OwnedSegments>>| {
                                                 let r = r.unwrap();
                                                 let ranking: cap::ranking::Reader =
                                                     r.get_root()
                                                      .unwrap();
                                                 ranking.get_pagerank() > 10
                                             },
                                             data);
    println!("{:?}", result);
    Ok(())
}
