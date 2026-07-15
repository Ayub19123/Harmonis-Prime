//! M2.7.14: Benchmark Execution Layer
//!
//! The missing SAT execution standard layer.
//! Provides batch execution, baseline comparison, metrics export, and regression intelligence.

pub mod comparator;
pub mod exporter;
pub mod history;
pub mod runner;

#[cfg(test)]
mod tests {
    use super::comparator::{par2_score, BaselineComparator};
    use super::exporter::{export_csv, export_json};
    use super::history::VersionHistory;
    use super::runner::{BenchmarkConfig, BenchmarkRunner};
    use std::collections::HashMap;

    #[test]
    fn test_benchmark_config_default() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.timeout_secs, 300);
        assert!(config.memory_limit_mb.is_none());
        assert!(config.cpu_affinity.is_none());
    }

    #[test]
    fn test_benchmark_runner_new() {
        let config = BenchmarkConfig::default();
        let runner = BenchmarkRunner::new(config);
        assert_eq!(runner.config.timeout_secs, 300);
    }

    #[test]
    fn test_par2_score_all_pass() {
        let runtimes = vec![1000u64, 2000u64, 3000u64];
        let score = par2_score(&runtimes, 5000);
        assert!((score - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_par2_score_with_timeout() {
        let runtimes = vec![1000u64, 6000u64]; // One timeout at 5000ms threshold
        let score = par2_score(&runtimes, 5000);
        // (1.0 + 10.0) / 2 = 5.5
        assert!((score - 5.5).abs() < 0.001);
    }

    #[test]
    fn test_baseline_comparator_compare() {
        let mut baseline = HashMap::new();
        baseline.insert("test1.cnf".to_string(), 1000u64);
        baseline.insert("test2.cnf".to_string(), 2000u64);

        let mut current = HashMap::new();
        current.insert("test1.cnf".to_string(), 1100u64); // 10% slower
        current.insert("test2.cnf".to_string(), 1900u64); // 5% faster

        let comparator = BaselineComparator::new(5.0);
        let deltas = comparator.compare(&baseline, &current);

        assert_eq!(deltas.len(), 2);
        let regressed = comparator.flag_regressions(&deltas);
        assert_eq!(regressed.len(), 1);
        assert_eq!(regressed[0].instance, "test1.cnf");
    }

    #[test]
    fn test_export_json_empty() {
        let runs = vec![];
        let temp = std::env::temp_dir().join("test_empty.json");
        let _ = std::fs::remove_file(&temp);
        export_json(&runs, &temp).unwrap();
        let content = std::fs::read_to_string(&temp).unwrap();
        assert_eq!(content.trim(), "[]");
        let _ = std::fs::remove_file(&temp);
    }

    #[test]
    fn test_export_csv_empty() {
        let runs = vec![];
        let temp = std::env::temp_dir().join("test_empty.csv");
        let _ = std::fs::remove_file(&temp);
        export_csv(&runs, &temp).unwrap();
        let content = std::fs::read_to_string(&temp).unwrap();
        assert!(content.contains("instance"));
        let _ = std::fs::remove_file(&temp);
    }

    #[test]
    fn test_version_history_open_and_record() {
        let temp = std::env::temp_dir().join("test_m2714.db");
        let _ = std::fs::remove_file(&temp);

        let history = VersionHistory::open(&temp).unwrap();
        history
            .record_run(
                "v6.2.0-M2.7.14",
                "/path/to/test.cnf",
                "abc123",
                "SAT",
                100,
                200,
                50,
                5,
                1024,
                1500,
                Some(true),
                false,
                false,
            )
            .unwrap();

        let runs = history.query_tag("v6.2.0-M2.7.14").unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].0, "/path/to/test.cnf");
        assert_eq!(runs[0].6, 1500); // wall_time_ms

        let _ = std::fs::remove_file(&temp);
    }
}
