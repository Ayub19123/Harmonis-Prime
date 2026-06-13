//! SET-5.3: Checkpoint Engine

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq)]
pub struct Checkpoint {
    pub sequence: u64,
    pub determinism_hash: u64,
    pub entropy_value: f64,
    pub operation_count: u64,
    pub heap_bytes: usize,
}

#[derive(Debug)]
pub struct CheckpointEngine {
    sequence: u64,
    checkpoints: Vec<Checkpoint>,
}

impl CheckpointEngine {
    pub fn new() -> Self {
        Self { sequence: 0, checkpoints: Vec::new() }
    }

    pub fn seal(&mut self, entropy_value: f64, operation_count: u64, heap_bytes: usize) -> Checkpoint {
        let mut hasher = DefaultHasher::new();
        entropy_value.to_bits().hash(&mut hasher);
        operation_count.hash(&mut hasher);
        let hash = hasher.finish();
        let cp = Checkpoint {
            sequence: self.sequence,
            determinism_hash: hash,
            entropy_value,
            operation_count,
            heap_bytes,
        };
        self.sequence += 1;
        self.checkpoints.push(cp.clone());
        cp
    }

    pub fn verify_determinism(&self, a: &Checkpoint, b: &Checkpoint) -> bool {
        a.determinism_hash == b.determinism_hash && a.entropy_value == b.entropy_value && a.operation_count == b.operation_count
    }

    pub fn entropy_variance(&self) -> f64 {
        if self.checkpoints.len() < 2 { return 0.0; }
        let mut sum_sq = 0.0;
        for w in self.checkpoints.windows(2) {
            let diff = w[1].entropy_value - w[0].entropy_value;
            sum_sq += diff * diff;
        }
        sum_sq / (self.checkpoints.len() - 1) as f64
    }

    pub fn checkpoint_count(&self) -> usize { self.checkpoints.len() }
}
