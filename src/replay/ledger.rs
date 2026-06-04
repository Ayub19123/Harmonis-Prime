use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;

/// LedgerFrame: One immutable observation of an execution cycle
/// Frame = (seq, input, seed, pre_hash, post_hash, timestamp)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerFrame {
    pub sequence_id: u64,
    pub input_payload: Vec<u8>,
    pub rand_seed: u64,
    pub pre_state_hash: String,
    pub post_state_hash: String,
    pub cycle_timestamp_nanos: u64,
    pub cycle_duration_micros: u64,
}

/// ReplayConfig: Configuration for the replay ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayConfig {
    pub max_frames: usize,
    pub enable_hashing: bool,
    pub enable_compression: bool,
    pub ring_buffer_size: usize,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            max_frames: 1_000_000,
            enable_hashing: true,
            enable_compression: false,
            ring_buffer_size: 10_000,
        }
    }
}

/// ReplayLedger: Append-only ring buffer of execution frames
/// Mathematical invariant: frames[0..n] is strictly monotonic in sequence_id
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayLedger {
    pub frames: VecDeque<LedgerFrame>,
    pub config: ReplayConfig,
    pub total_cycles: u64,
    pub ledger_hash_chain: String,
}

impl ReplayLedger {
    /// Create new ledger with configuration
    pub fn new(config: ReplayConfig) -> Self {
        Self {
            frames: VecDeque::with_capacity(config.ring_buffer_size),
            config,
            total_cycles: 0,
            ledger_hash_chain: "0".repeat(64),
        }
    }

    /// Append frame — O(1) amortized
    /// Invariant: sequence_id = total_cycles + 1 (strictly monotonic)
    pub fn append(&mut self, frame: LedgerFrame) -> Result<(), String> {
        if frame.sequence_id != self.total_cycles + 1 {
            return Err(format!(
                "Sequence gap: expected {}, got {}",
                self.total_cycles + 1,
                frame.sequence_id
            ));
        }

        // Update hash chain: H(prev_chain || frame_hash)
        let frame_json = serde_json::to_string(&frame).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(self.ledger_hash_chain.as_bytes());
        hasher.update(frame_json.as_bytes());
        self.ledger_hash_chain = format!("{:x}", hasher.finalize());

        // Ring buffer eviction
        if self.frames.len() >= self.config.ring_buffer_size {
            self.frames.pop_front();
        }

        self.frames.push_back(frame);
        self.total_cycles += 1;

        Ok(())
    }

    /// Get frame by sequence ID
    pub fn get_frame(&self, sequence_id: u64) -> Option<&LedgerFrame> {
        self.frames.iter().find(|f| f.sequence_id == sequence_id)
    }

    /// Get latest frame
    pub fn latest(&self) -> Option<&LedgerFrame> {
        self.frames.back()
    }

    /// Get frame range
    pub fn range(&self, start: u64, end: u64) -> Vec<&LedgerFrame> {
        self.frames
            .iter()
            .filter(|f| f.sequence_id >= start && f.sequence_id <= end)
            .collect()
    }

    /// Verify hash chain integrity
    pub fn verify_chain(&self) -> bool {
        let mut chain = "0".repeat(64);
        for frame in self.frames.iter() {
            let frame_json = serde_json::to_string(frame).unwrap_or_default();
            let mut hasher = Sha256::new();
            hasher.update(chain.as_bytes());
            hasher.update(frame_json.as_bytes());
            chain = format!("{:x}", hasher.finalize());
        }
        chain == self.ledger_hash_chain
    }

    /// Ledger statistics
    pub fn stats(&self) -> LedgerStats {
        LedgerStats {
            total_frames: self.total_cycles,
            buffered_frames: self.frames.len() as u64,
            hash_chain_valid: self.verify_chain(),
            genesis_hash: "0".repeat(64),
            current_hash: self.ledger_hash_chain.clone(),
        }
    }
}

/// LedgerStats: Observability morphism for the ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerStats {
    pub total_frames: u64,
    pub buffered_frames: u64,
    pub hash_chain_valid: bool,
    pub genesis_hash: String,
    pub current_hash: String,
}
