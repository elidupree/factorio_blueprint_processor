#[macro_use]
extern crate criterion;

use criterion::{Benchmark, Criterion};
use std::time::Duration;

use serde_json::Result;

use factorio_blueprint_processor::belt_routing;
use factorio_blueprint_processor::blueprint::*;

fn bench_belt_routing(criterion: &mut Criterion) {
  criterion.bench(
    "belt_routing_1",
    Benchmark::new("belt_routing_2", |bencher| {
      bencher.iter(|| belt_routing::route_blueprint_thingy())
    })
    .sample_size(5),
  );
}

criterion_group!(benches, bench_belt_routing);
criterion_main!(benches);
