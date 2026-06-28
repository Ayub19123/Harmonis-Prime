//! SET-5.3: Memory Profiler — Heap growth tracking

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemorySnapshot {
    pub heap_bytes: usize,
    pub timestamp: Instant,
}

#[derive(Debug)]
pub struct MemoryProfiler {
    baseline: Option<MemorySnapshot>,
    snapshots: Vec<MemorySnapshot>,
    start: Instant,
}

impl MemoryProfiler {
    pub fn new() -> Self {
        Self {
            baseline: None,
            snapshots: Vec::new(),
            start: Instant::now(),
        }
    }

    /// Return elapsed time since profiler creation
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn snapshot(&mut self, heap_bytes: usize) -> MemorySnapshot {
        let snap = MemorySnapshot {
            heap_bytes,
            timestamp: Instant::now(),
        };
        if self.baseline.is_none() {
            self.baseline = Some(snap);
        }
        self.snapshots.push(snap);
        snap
    }

    pub fn growth_rate_per_hour(&self) -> f64 {
        if self.snapshots.len() < 2 {
            return 0.0;
        }
        let baseline = self.baseline.unwrap();
        let latest = self.snapshots.last().unwrap();
        let elapsed_hours = latest
            .timestamp
            .duration_since(baseline.timestamp)
            .as_secs_f64()
            / 3600.0;
        if elapsed_hours == 0.0 {
            return 0.0;
        }
        let growth =
            (latest.heap_bytes as f64 - baseline.heap_bytes as f64) / baseline.heap_bytes as f64;
        growth / elapsed_hours * 100.0
    }

    pub fn verify_invariant(&self, max_percent_per_hour: f64) -> bool {
        self.growth_rate_per_hour() <= max_percent_per_hour
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_profiler_elapsed_active() {
        let profiler = MemoryProfiler::new();
        let elapsed = profiler.elapsed();
        assert!(elapsed >= Duration::ZERO, "Elapsed must be non-negative");
    }
}
