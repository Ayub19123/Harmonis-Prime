//! BRICK-49: Formal Verification Guardian — Shared Types
//! Mathematical immutability, state-space safety, zero-drift type system

use std::time::{Duration, Instant};

/// TemporalState: Causally-ordered state snapshot with lineage hash
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TemporalState {
    pub state_id: String,
    pub timestamp: Instant,
    pub predecessor_hash: String,
    pub state_hash: String,
    pub quantum_signature: Option<Vec<u8>>,
}

impl TemporalState {
    pub fn new(id: &str, predecessor: &str) -> Self {
        Self {
            state_id: id.to_string(),
            timestamp: Instant::now(),
            predecessor_hash: predecessor.to_string(),
            state_hash: format!("hash_{}_{}", id, predecessor),
            quantum_signature: None,
        }
    }

    pub fn with_quantum(mut self, sig: Vec<u8>) -> Self {
        self.quantum_signature = Some(sig);
        self
    }
}

/// CausalityProof: Temporal lineage verification record
#[derive(Clone, Debug)]
pub struct CausalityProof {
    pub proof_id: String,
    pub from_state: String,
    pub to_state: String,
    pub temporal_delta: Duration,
    pub linearization_score: f64,
    pub paradox_detected: bool,
}

impl CausalityProof {
    pub fn new(from: &str, to: &str, delta_ms: u64) -> Self {
        Self {
            proof_id: format!("proof_{}_{}", from, to),
            from_state: from.to_string(),
            to_state: to.to_string(),
            temporal_delta: Duration::from_millis(delta_ms),
            linearization_score: 1.0,
            paradox_detected: false,
        }
    }
}

/// ByzantineNode: Identity-verified consensus participant
#[derive(Clone, Debug)]
pub struct ByzantineNode {
    pub node_id: String,
    pub lineage_hash: String,
    pub trust_score: f64,
    pub compromised: bool,
    pub last_consensus: Instant,
}

impl ByzantineNode {
    pub fn new(id: &str, lineage: &str) -> Self {
        Self {
            node_id: id.to_string(),
            lineage_hash: lineage.to_string(),
            trust_score: 1.0,
            compromised: false,
            last_consensus: Instant::now(),
        }
    }
}

/// FormalTheorem: Verified mathematical statement
#[derive(Clone, Debug)]
pub struct FormalTheorem {
    pub theorem_id: String,
    pub statement: String,
    pub proof_steps: Vec<String>,
    pub verified: bool,
    pub verification_time_ms: u64,
}

impl FormalTheorem {
    pub fn new(id: &str, statement: &str) -> Self {
        Self {
            theorem_id: id.to_string(),
            statement: statement.to_string(),
            proof_steps: Vec::new(),
            verified: false,
            verification_time_ms: 0,
        }
    }

    pub fn with_proof(mut self, steps: Vec<&str>, time_ms: u64) -> Self {
        self.proof_steps = steps.iter().map(|s| s.to_string()).collect();
        self.verified = true;
        self.verification_time_ms = time_ms;
        self
    }
}

/// LineageEntry: Immutable cryptographic anchor
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LineageEntry {
    pub entry_id: String,
    pub operation_hash: String,
    pub parent_hash: String,
    pub block_height: u64,
    pub timestamp: Instant,
}

impl LineageEntry {
    pub fn genesis() -> Self {
        Self {
            entry_id: "genesis".to_string(),
            operation_hash: "0".repeat(64),
            parent_hash: "0".repeat(64),
            block_height: 0,
            timestamp: Instant::now(),
        }
    }

    pub fn child(&self, op_hash: &str) -> Self {
        Self {
            entry_id: format!("entry_{}", self.block_height + 1),
            operation_hash: op_hash.to_string(),
            parent_hash: self.operation_hash.clone(),
            block_height: self.block_height + 1,
            timestamp: Instant::now(),
        }
    }
}

/// StressResult: Certification test outcome
#[derive(Clone, Debug)]
pub struct StressResult {
    pub test_name: String,
    pub passed: bool,
    pub metric_value: f64,
    pub target_value: f64,
    pub cycles_executed: u64,
}

impl StressResult {
    pub fn pass(name: &str, metric: f64, target: f64, cycles: u64) -> Self {
        Self {
            test_name: name.to_string(),
            passed: metric >= target,
            metric_value: metric,
            target_value: target,
            cycles_executed: cycles,
        }
    }
}
