extern crate dx16;
extern crate glob;

use dx16::Dx16Result;
use dx16::mapred;

use dx16::data::UserVisits;

fn main() {
    let set = "5nodes";
    scan(set).unwrap();
}

fn scan(set: &str) -> Dx16Result<()> {
    let bibi = dx16::bibi_rmp_gz_dec(set, "uservisits");
    let result = mapred::MapReduceOp::map_reduce(|_: Dx16Result<UserVisits>| {
                                                     Box::new(Some(((), 1)).into_iter())
                                                 },
                                                 |a, b| *a + *b,
                                                 bibi);
    println!("{:?}", result);

    Ok(())
}
