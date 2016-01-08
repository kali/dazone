#![feature(clone_from_slice)]

extern crate dx16;
extern crate glob;
extern crate capnp;
extern crate clap;
extern crate libc;
extern crate time;

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
                      .arg(Arg::with_name("INPUT")
                               .short("i")
                               .long("input")
                               .help("cap or rmp")
                               .takes_value(true))
                      .arg(Arg::with_name("LENGTH")
                               .short("l")
                               .long("length")
                               .help("length (8, 10, or 12)")
                               .takes_value(true))
                      .arg(Arg::with_name("REDUCE")
                               .short("r")
                               .long("reduce")
                               .help("reduce strategy: mono multi")
                               .takes_value(true))
                      .arg(Arg::with_name("REDUCE PAR")
                               .short("p")
                               .long("parallelism")
                               .help("multi reduce par factor")
                               .takes_value(true))
                      .arg(Arg::with_name("QUIET")
                               .short("q")
                               .long("quiet")
                               .help("hide progress bar")
                               .takes_value(false))
                      .get_matches();
    let set = matches.value_of("SET").unwrap_or("5nodes");
    let quiet = matches.is_present("QUIET");
    let length: usize = matches.value_of("LENGTH").unwrap_or("8").parse().unwrap();
    let par: usize = matches.value_of("REDUCE PAR").unwrap_or("16").parse().unwrap();
    let bibi: BI<BI<([u8; 12], f32)>> = match matches.value_of("INPUT").unwrap_or("cap") {
        "cap" => {
            Box::new(dx16::bibi_cap_gz_dec(set, "uservisits")
                         .map(move |chunk| -> BI<([u8; 12], f32)> {
                             Box::new(chunk.map(move |reader: Dx16Result<Reader<OwnedSegments>>| {
                                 let reader = reader.unwrap();
                                 let visit: ::dx16::cap::user_visits::Reader = reader.get_root()
                                                                                     .unwrap();
                                 let mut chars = [b' '; 12];
                                 chars.clone_from_slice(visit.get_source_i_p().unwrap().as_bytes());
                                 for i in length..chars.len() {
                                     chars[i] = b' ';
                                 }
                                 (chars, visit.get_ad_revenue())
                             }))
                         }))
        }
        "rmp" => {
            Box::new(dx16::rmp_read::bibi_rmp_gz_dec(set, "uservisits")
                         .map(move |chunk| -> BI<([u8; 12], f32)> {
                             Box::new(chunk.map(move |reader: Dx16Result<::dx16::data::UserVisits>| {
                                 let visit = reader.unwrap();
                                 let mut chars = [b' '; 12];
                                 chars.clone_from_slice(visit.source_ip.as_bytes());
                                 for i in length..chars.len() {
                                     chars[i] = b' ';
                                 }
                                (chars, visit.ad_revenue)
                             }))
                         }))
        }
        any => panic!("unknown input format {}", any),
    };
    let r = |a: &f32, b: &f32| a + b;
    let t1 = ::time::get_time();

    let strategy = matches.value_of("REDUCE").unwrap_or("hashes");
    let len = match strategy {
        "mono" => {
            let mut aggregator = ::dx16::aggregators::HashMapAggregator::new(&r);
            MapOp::new_map_reduce(|(a, b)| Emit::One(a, b)).run(bibi, &mut aggregator, quiet);
            aggregator.converge();
            aggregator.len()
        }
        "hashes" => {
            let mut aggregator = ::dx16::aggregators::MultiHashMapAggregator::new(&r, par);
            MapOp::new_map_reduce(|(a, b)| Emit::One(a, b)).run(bibi, &mut aggregator, quiet);
            aggregator.converge();
            aggregator.len()
        }
        "tries" => {
            let mut aggregator = ::dx16::aggregators::MultiTrieAggregator::new(&r, par);
            MapOp::new_map_reduce(|(a, b): ([u8; 12], f32)| Emit::One(a.to_vec(), b))
                .run(bibi, &mut aggregator, quiet);
            aggregator.converge();
            aggregator.len()
        }
        s => panic!("unknown strategy {}", s),
    };

    let t2 = ::time::get_time();
    let mut usage = ::libc::rusage {
        ru_idrss: 0,
        ru_nvcsw: 0,
        ru_ixrss: 0,
        ru_isrss: 0,
        ru_inblock: 0,
        ru_minflt: 0,
        ru_oublock: 0,
        ru_nivcsw: 0,
        ru_stime: libc::timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        ru_nswap: 0,
        ru_maxrss: 0,
        ru_majflt: 0,
        ru_msgrcv: 0,
        ru_msgsnd: 0,
        ru_utime: libc::timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        ru_nsignals: 0,
    };
    unsafe {
        ::libc::getrusage(::libc::RUSAGE_SELF, &mut usage);
    }
    println!("set:{} length:{} strat:{} par:{} groups:{} rss_mb:{} utime_s:{} stime_s:{} \
              ctime_s:{}",
             set,
             length,
             strategy,
             par,
             len,
             usage.ru_maxrss / 1024 / 1024,
             usage.ru_utime.tv_sec,
             usage.ru_stime.tv_sec,
             (t2 - t1).num_seconds());

}
