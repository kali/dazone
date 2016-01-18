extern crate dazone;
extern crate glob;

use dazone::Dx16Result;
use dazone::mapred;
use dazone::mapred::BI;

fn main() {
    let set = "5nodes";
    scan(set, "rankings").unwrap();
}

fn scan(set: &str, table: &str) -> Dx16Result<()> {
    let source_root = dazone::data_dir_for("rmp-gz", set, table);
    let glob = source_root.clone() + "/*.rmp.gz";
    let pageranks: BI<BI<Dx16Result<u32>>> =
        Box::new(::glob::glob(&glob).unwrap().map(|f| -> BI<Dx16Result<u32>> {
            let cmd = ::std::process::Command::new("gzcat")
                          .arg("-d")
                          .arg(f.unwrap())
                          .stdout(::std::process::Stdio::piped())
                          .spawn()
                          .unwrap();
            Box::new(dazone::rmp_read::RankingRMPReader::new(cmd.stdout.unwrap()))
        }));
    let result = mapred::FilterCountOp::filter_count(|r: Dx16Result<u32>| r.unwrap() > 10,
                                                     pageranks);
    println!("{:?}", result);
    Ok(())
}
