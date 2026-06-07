// src/bin/benchmark.rs
// HONEST GRAPH BENCHMARK — SharedMemoryGraph operations.

use std::env;
use std::hint::black_box;
use sovereign_core::brick51::shared_memory_graph::SharedMemoryGraph;
use sovereign_core::benchmark::mlperf_telemetry;

fn run_graph_iteration(seed: u64) {
    let mut graph = SharedMemoryGraph::new(0, 1);
    let key = format!("key_{}", seed);
    let value = seed.to_string();
    let clock = vec![seed];
    graph.insert(&key, &value, clock);
    let _ = graph.get(&key);
    black_box(&graph);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let iterations: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(100);
    let seed: u64 = args.get(2)
        .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0x51C3_2026_0613);

    println!("🧱 HARMONIS PRIME — GRAPH BENCHMARK");
    println!("   Iterations: {}", iterations);
    println!("   Seed: 0x{:016X}", seed);
    println!("   Workload: SharedMemoryGraph insert + get");
    println!("   Hardware: SIMULATED (consumer laptop, stock config)");
    println!("   Energy: NOT MEASURED (honest null)");
    println!();

    let start_total = std::time::Instant::now();
    let results = mlperf_telemetry::run_mlperf_benchmark(
        iterations,
        seed,
        "harmonis_shared_memory_graph",
        run_graph_iteration,
    );
    let total_elapsed = start_total.elapsed();

    mlperf_telemetry::write_mlperf_json(&results, "metrics.json");
    mlperf_telemetry::write_limitations_md("LIMITATIONS.md");

    let latencies: Vec<u64> = results.iter().map(|r| r.latency_ns).collect();
    let min_latency = latencies.iter().min().unwrap_or(&0);
    let max_latency = latencies.iter().max().unwrap_or(&0);
    let avg_latency = latencies.iter().sum::<u64>() / latencies.len() as u64;

    println!("✅ BENCHMARK COMPLETE");
    println!("   Total time: {:.6}s", total_elapsed.as_secs_f64());
    println!("   Iterations: {}", results.len());
    println!("   Min latency: {} ns", min_latency);
    println!("   Max latency: {} ns", max_latency);
    println!("   Avg latency: {} ns", avg_latency);
    println!();
    println!("   Output: metrics.json");
    println!("   Limitations: LIMITATIONS.md");
    println!();
    println!("🧱 SOVEREIGN PRINCIPLE: Claims = Measurements. Nothing more.");
}


