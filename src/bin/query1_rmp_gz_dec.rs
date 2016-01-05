extern crate dx16;
extern crate glob;

use dx16::Dx16Result;
use dx16::mapred;

use dx16::data::Ranking;

fn main() {
    let set = "5nodes";
    scan(set, "rankings").unwrap();
}

fn scan(set: &str, table: &str) -> Dx16Result<()> {
    let data = dx16::rmp_read::bibi_rmp_gz_dec(set, table);
    let result = mapred::FilterCountOp::filter_count(|r: Dx16Result<Ranking>| {
                                                         r.unwrap().pagerank > 10
                                                     },
                                                     data);
    println!("{:?}", result);
    Ok(())
}
