extern crate dazone;

use dazone::crunch::*;
use dazone::files::bibi_pod;
use dazone::short_bytes_array::*;
use dazone::data::pod::UserVisits;

fn main() {
    let input = bibi_pod("5nodes", "uservisits", "text-deflate");
    let map = |visit: UserVisits| Emit::One(Bytes8::prefix(&*visit.source_ip), visit.ad_revenue);
    let reduce = |a: &f32, b: &f32| a + b;
    let mut aggregator = aggregators::MultiHashMapAggregator::new(&reduce, 256);
    MapOp::new_map_reduce(map)
        .with_progress(true)
        .with_workers(16)
        .run(input, &mut aggregator);
    aggregator.converge();
    println!("### {} ###", aggregator.len());
}
