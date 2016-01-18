extern crate dazone;
extern crate glob;

use dazone::Dx16Result;
use dazone::mapred;

use dazone::data::Ranking;

fn main() {
    let set = "5nodes";
    scan(set, "rankings").unwrap();
}

fn scan(set: &str, table: &str) -> Dx16Result<()> {
    let data = dazone::rmp_read::bibi_rmp_gz_dec(set, table);
    let result = mapred::FilterCountOp::filter_count(|r: Dx16Result<Ranking>| {
                                                         r.unwrap().pagerank > 10
                                                     },
                                                     data);
    println!("{:?}", result);
    Ok(())
}
