extern crate dx16;
extern crate glob;

use dx16::Dx16Result;
use dx16::mapred;
use dx16::mapred::BI;

use dx16::data::Ranking;

fn main() {
    let set = "5nodes";
    scan(set, "rankings").unwrap();
}

fn scan(set: &str, table: &str) -> Dx16Result<()> {
    let source_root = dx16::data_dir_for("rmp-gz", set, table);
    let glob = source_root.clone() + "/*.rmp.gz";
    let pageranks: BI<BI<Dx16Result<Ranking>>> =
        Box::new(::glob::glob(&glob).unwrap().map(|f| -> BI<Dx16Result<Ranking>> {
            let cmd = ::std::process::Command::new("gzcat")
                          .arg("-d")
                          .arg(f.unwrap())
                          .stdout(::std::process::Stdio::piped())
                          .spawn()
                          .unwrap();
            Box::new(dx16::RankingReader::new(cmd.stdout.unwrap()))
        }));
    let result = mapred::MapReduceOp::map_reduce(|r: Dx16Result<Ranking>| r.unwrap().pagerank > 10,
                                                 pageranks);
    println!("{:?}", result);
    Ok(())
}
