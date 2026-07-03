//! M2.7.14: Benchmark Execution Layer — CLI Binary
//!
//! The unified benchmarking operating system for Harmonis Prime.
//! Usage: cargo run --bin benchmark_runner -- --input-dir <DIR> --format <json|csv>

use clap::Parser;
use sovereign_core::benchmark::comparator::{par2_score, BaselineComparator};
use sovereign_core::benchmark::exporter::{export_csv, export_json};
use sovereign_core::benchmark::history::VersionHistory;
use sovereign_core::benchmark::runner::{BenchmarkConfig, BenchmarkRunner};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "benchmark_runner")]
#[command(about = "M2.7.14: Harmonis Prime Benchmark Execution Layer")]
struct Cli {
    #[arg(short, long, help = "Directory containing .cnf benchmark instances")]
    input_dir: PathBuf,

    #[arg(
        short,
        long,
        default_value = "benchmark_output",
        help = "Output directory"
    )]
    output_dir: PathBuf,

    #[arg(
        short,
        long,
        default_value = "json",
        help = "Output format: json or csv"
    )]
    format: String,

    #[arg(short, long, help = "SQLite database path for version history")]
    db_path: Option<PathBuf>,

    #[arg(short, long, help = "Git tag for baseline comparison")]
    baseline_tag: Option<String>,

    #[arg(
        short,
        long,
        default_value_t = 300,
        help = "Timeout per instance in seconds"
    )]
    timeout: u64,

    #[arg(short, long, help = "Current git tag for recording")]
    git_tag: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    std::fs::create_dir_all(&cli.output_dir)?;

    let config = BenchmarkConfig {
        timeout_secs: cli.timeout,
        output_dir: cli.output_dir.clone(),
        ..BenchmarkConfig::default()
    };

    let runner = BenchmarkRunner::new(config);
    let results = runner.run_batch(&cli.input_dir);

    let mut successful_runs = Vec::new();
    let mut failed_count = 0usize;

    for result in results {
        match result {
            Ok(run) => successful_runs.push(run),
            Err(e) => {
                eprintln!("M2.7.14: Benchmark failed: {}", e);
                failed_count += 1;
            }
        }
    }

    println!("M2.7.14 Benchmark Execution Complete");
    println!(
        "  Total instances: {}",
        successful_runs.len() + failed_count
    );
    println!("  Successful: {}", successful_runs.len());
    println!("  Failed: {}", failed_count);

    // Export metrics
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    match cli.format.as_str() {
        "json" => {
            let path = cli.output_dir.join(format!("benchmark_{}.json", timestamp));
            export_json(&successful_runs, &path)?;
            println!("  JSON export: {}", path.display());
        }
        "csv" => {
            let path = cli.output_dir.join(format!("benchmark_{}.csv", timestamp));
            export_csv(&successful_runs, &path)?;
            println!("  CSV export: {}", path.display());
        }
        _ => eprintln!("Warning: Unknown format '{}', skipping export", cli.format),
    }

    // Extract db_path once to avoid partial move of cli
    let db_path_opt = cli.db_path.as_ref();

    // Record to version history
    if let Some(db_path) = db_path_opt {
        let history = VersionHistory::open(db_path)?;
        let tag = cli.git_tag.as_deref().unwrap_or("unknown");

        for run in &successful_runs {
            history.record_run(
                tag,
                &run.instance_path.to_string_lossy(),
                &run.instance_hash,
                &format!("{:?}", run.result),
                run.decisions,
                run.propagations,
                run.conflicts,
                run.restarts,
                run.peak_memory_kb,
                run.wall_time_ms,
                run.proof_valid,
                run.timed_out,
                run.memory_exceeded,
            )?;
        }
        println!(
            "  Recorded {} runs to {}",
            successful_runs.len(),
            db_path.display()
        );
    }

    // Baseline comparison
    if let Some(baseline_tag) = cli.baseline_tag {
        if let Some(db_path) = db_path_opt {
            let history = VersionHistory::open(db_path)?;
            let baseline = history.query_tag(&baseline_tag)?;
            let current: HashMap<String, u64> = successful_runs
                .iter()
                .map(|r| {
                    (
                        r.instance_path.to_string_lossy().to_string(),
                        r.wall_time_ms,
                    )
                })
                .collect();
            let baseline_map: HashMap<String, u64> = baseline
                .iter()
                .map(|(path, _, _, _, _, _, time)| (path.clone(), *time))
                .collect();

            let comparator = BaselineComparator::new(5.0); // 5% epsilon
            let deltas = comparator.compare(&baseline_map, &current);
            let regressions = comparator.flag_regressions(&deltas);

            println!("  Baseline comparison against {}:", baseline_tag);
            println!("    Instances compared: {}", deltas.len());
            println!("    Regressions flagged: {}", regressions.len());

            if !regressions.is_empty() {
                println!("    REGRESSIONS DETECTED:");
                for r in &regressions {
                    println!(
                        "      {}: +{:.2}% ({}ms -> {}ms)",
                        r.instance, r.delta_pct, r.baseline_time_ms, r.current_time_ms
                    );
                }
            }
        } else {
            eprintln!("Warning: --db-path required for baseline comparison");
        }
    }

    // Par-2 score
    if !successful_runs.is_empty() {
        let runtimes: Vec<u64> = successful_runs.iter().map(|r| r.wall_time_ms).collect();
        let timeout_ms = cli.timeout * 1000;
        let par2 = par2_score(&runtimes, timeout_ms);
        println!("  Par-2 Score: {:.3}s", par2);
    }

    Ok(())
}
