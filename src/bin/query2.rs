#![feature(clone_from_slice)]

extern crate dx16;
extern crate capnp;
#[macro_use]
extern crate clap;
extern crate time;
extern crate num_cpus;

use dx16::Dx16Result;
use dx16::mapred::*;

use capnp::serialize::OwnedSegments;
use capnp::message::Reader;

fn main() {
    let app = clap_app!( query2 =>
        (@arg SET: -s --set +takes_value "(tiny, 1node, 5nodes")
        (@arg INPUT: -i --input +takes_value "(cap, rmp)")
        (@arg CHUNKS: -c --chunks +takes_value "all")
        (@arg LENGTH: -l --length +takes_value "(8, 10 or 12)")
        (@arg REDUCE: -r --reduce +takes_value "(hash, hashes, tries)")
        (@arg BUCKETS: -b --buckets +takes_value "reduce buckets (256)")
        (@arg PROGRESS: -p --progress "show progressbar")
        (@arg MEMORY: -m --memory "monitor memory usage")
        (@arg WORKERS: -w --workers +takes_value "worker threads (num_cpu*2)")
    );
    let matches = app.get_matches();
    let set = matches.value_of("SET").unwrap_or("5nodes");
    let chunks: usize = matches.value_of("CHUNKS").unwrap_or("999999999").parse().unwrap();
    let progress = matches.is_present("PROGRESS");
    let length: usize = matches.value_of("LENGTH").unwrap_or("8").parse().unwrap();
    let buckets: usize = matches.value_of("BUCKETS").unwrap_or("256").parse().unwrap();
    let workers: usize = matches.value_of("WORKERS")
                                .map(|a| a.parse::<usize>().unwrap())
                                .unwrap_or(::num_cpus::get() * 2);

    let bibi: BI<BI<([u8; 12], f32)>> = match matches.value_of("INPUT").unwrap_or("cap") {
        "cap" => {
            Box::new(dx16::bibi_cap_gz_dec(set, "uservisits")
                         .take(chunks)
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
            Box::new(dx16::rmp_read::bibi_rmp_gz_dec(set, "uservisits").take(chunks)
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

    if matches.is_present("MEMORY") {
        ::dx16::rusage::start_monitor(::std::time::Duration::from_secs(10));
    }

    let strategy = matches.value_of("REDUCE").unwrap_or("hashes");
    let groups = match strategy {
        "hash" => {
            let mut aggregator = ::dx16::aggregators::HashMapAggregator::new(&r);
            MapOp::new_map_reduce(|(a, b)| Emit::One(a, b))
                .with_progress(progress)
                .with_workers(workers)
                .run(bibi, &mut aggregator);
            aggregator.converge();
            aggregator.len()
        }
        "hashes" => {
            let mut aggregator = ::dx16::aggregators::MultiHashMapAggregator::new(&r, buckets);
            MapOp::new_map_reduce(|(a, b)| Emit::One(a, b))
                .with_progress(progress)
                .with_workers(workers)
                .run(bibi, &mut aggregator);
            aggregator.converge();
            aggregator.len()
        }
        "tries" => {
            let mut aggregator = ::dx16::aggregators::MultiTrieAggregator::new(&r, buckets);
            MapOp::new_map_reduce(|(a, b): ([u8; 12], f32)| Emit::One(a.to_vec(), b))
                .with_progress(progress)
                .with_workers(workers)
                .run(bibi, &mut aggregator);
            aggregator.converge();
            aggregator.len()
        }
        s => panic!("unknown strategy {}", s),
    };
    let t2 = ::time::get_time();

    let usage = ::dx16::rusage::get_rusage();
    let vmsize = ::dx16::rusage::get_memory_usage().unwrap().virtual_size;
    println!("set: {:6} chunks: {:4} length: {:2} strat: {:6} buckets: {:4} workers: {:4} \
              groups: {:9} rss_mb: {:5} vmmsize_mb: {:5} utime_s: {:5} stime_s: {:5} ctime_s: \
              {:5}",
             set,
             chunks,
             length,
             strategy,
             buckets,
             workers,
             groups,
             usage.ru_maxrss / 1024 / 1024,
             vmsize / 1024 / 1024,
             usage.ru_utime.tv_sec,
             usage.ru_stime.tv_sec,
             (t2 - t1).num_seconds());

}
