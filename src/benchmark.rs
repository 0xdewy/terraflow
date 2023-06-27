use bevy::prelude::*;
use std::time::{Duration, Instant};

#[derive(Debug, Default, Resource)]
pub struct BenchmarkResource {
    start_time: Option<Instant>,
    elapsed_time: Duration,
}

pub fn start_benchmark(mut benchmark: ResMut<BenchmarkResource>) {
    benchmark.start_time = Some(Instant::now());
}

pub fn end_benchmark(mut benchmark: ResMut<BenchmarkResource>) {
    if let Some(start_time) = benchmark.start_time {
        benchmark.elapsed_time = start_time.elapsed();
        println!("Benchmark elapsed time: {:?}", benchmark.elapsed_time);
        benchmark.start_time = None; // reset the start time
    }
}
