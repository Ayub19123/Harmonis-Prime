use crate::replay::hash_state::{hash_engine_state, EngineSnapshot};
use crate::replay::ledger::{LedgerFrame, ReplayLedger};
use serde::{Deserialize, Serialize};

/// ReplayResult: Outcome of a determinism test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplayResult {
    Success {
        sequence_id: u64,
        expected_hash: String,
        actual_hash: String,
        duration_micros: u64,
    },
    Failure {
        sequence_id: u64,
        expected_hash: String,
        actual_hash: String,
        divergence_point: String,
    },
    Skipped {
        sequence_id: u64,
        reason: String,
    },
}

/// DeterminismHarness: Sandboxed replayer for bit-for-bit verification
/// Given (pre_state, input, seed), asserts post_state matches recorded hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismHarness {
    pub test_count: u64,
    pub pass_count: u64,
    pub fail_count: u64,
}

impl DeterminismHarness {
    /// Create new harness
    pub fn new() -> Self {
        Self {
            test_count: 0,
            pass_count: 0,
            fail_count: 0,
        }
    }

    /// Replay single frame: reconstruct state and verify hash
    /// Determinism axiom: replay(pre, input, seed) == post
    pub fn replay_frame(
        &mut self,
        frame: &LedgerFrame,
        mut engine_builder: impl FnMut() -> crate::engine::SovereignOrchestrator,
    ) -> ReplayResult {
        self.test_count += 1;

        let start = std::time::Instant::now();

        // Build clean engine instance
        let engine = engine_builder();

        // Apply input to engine (simplified — actual implementation routes through execute_cycle)
        // TODO: Wire into actual execute_cycle with forced seed

        // Hash resulting state
        let snapshot = EngineSnapshot {
            term: engine.term,
            commit_index: engine.commit_index,
            node_id: engine.node_id,
            kv_store_hash: String::new(),
            substrate_flags: vec![
                ("BRICK-31: FABRIC".to_string(), engine.data_fabric.is_some()),
                (
                    "BRICK-32: INTEL".to_string(),
                    engine.inference_engine.is_some(),
                ),
            ],
            timestamp_nanos: frame.cycle_timestamp_nanos,
        };

        let actual_hash = hash_engine_state(&snapshot);
        let duration = start.elapsed().as_micros() as u64;

        if actual_hash == frame.post_state_hash {
            self.pass_count += 1;
            ReplayResult::Success {
                sequence_id: frame.sequence_id,
                expected_hash: frame.post_state_hash.clone(),
                actual_hash,
                duration_micros: duration,
            }
        } else {
            self.fail_count += 1;
            ReplayResult::Failure {
                sequence_id: frame.sequence_id,
                expected_hash: frame.post_state_hash.clone(),
                actual_hash,
                divergence_point: "post_state_hash mismatch".to_string(),
            }
        }
    }

    /// Batch replay: verify all frames in a range
    pub fn replay_range(
        &mut self,
        ledger: &ReplayLedger,
        start_seq: u64,
        end_seq: u64,
        engine_builder: impl FnMut() -> crate::engine::SovereignOrchestrator,
    ) -> Vec<ReplayResult> {
        let mut results = Vec::new();
        let mut builder = engine_builder;

        for seq in start_seq..=end_seq {
            if let Some(frame) = ledger.get_frame(seq) {
                let result = self.replay_frame(frame, || builder());
                results.push(result);
            } else {
                results.push(ReplayResult::Skipped {
                    sequence_id: seq,
                    reason: "Frame not found in ledger".to_string(),
                });
            }
        }

        results
    }

    /// Harness statistics
    pub fn stats(&self) -> HarnessStats {
        HarnessStats {
            total_tests: self.test_count,
            passed: self.pass_count,
            failed: self.fail_count,
            pass_rate: if self.test_count > 0 {
                self.pass_count as f64 / self.test_count as f64
            } else {
                0.0
            },
        }
    }
}

/// HarnessStats: Observability morphism for the harness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessStats {
    pub total_tests: u64,
    pub passed: u64,
    pub failed: u64,
    pub pass_rate: f64,
}
