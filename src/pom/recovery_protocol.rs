use crate::pom::operational_memory::{compute_hash, OperationalEntry};
use crate::pom::operational_wal::OperationalWal;
use crate::pom::snapshot_engine::SnapshotEngine;

#[derive(Debug)]
pub enum RecoveryError {
    CausalIntegrityViolation {
        sequence: u64,
        expected: [u8; 32],
        found: [u8; 32],
    },
    SnapshotCorruption {
        epoch: u64,
        reason: String,
    },
    WalReplayError {
        segment: u64,
        reason: String,
    },
}

pub struct RecoveryProtocol;

impl RecoveryProtocol {
    pub fn recover(
        snapshot_engine: &SnapshotEngine,
        wal: &OperationalWal,
    ) -> Result<Vec<OperationalEntry>, RecoveryError> {
        let mut state: Vec<OperationalEntry> = Vec::new();
        let mut last_hash = [0u8; 32];

        // Step 1: Load latest snapshot if available
        match snapshot_engine.load_latest_snapshot() {
            Ok(Some(snapshot)) => {
                // Deserialize snapshot data into entries
                let entries: Vec<OperationalEntry> = serde_json::from_slice(&snapshot.data)
                    .map_err(|e| RecoveryError::SnapshotCorruption {
                        epoch: snapshot.epoch_id,
                        reason: e.to_string(),
                    })?;

                if let Some(last) = entries.last() {
                    last_hash = compute_hash(last);
                }
                state = entries;
            }
            Ok(None) => {
                // No snapshot — start from genesis
                last_hash = [0u8; 32];
            }
            Err(e) => {
                return Err(RecoveryError::SnapshotCorruption {
                    epoch: 0,
                    reason: e.to_string(),
                });
            }
        }

        // Step 2: Replay WAL from snapshot boundary
        let mut expected_sequence = state.len() as u64 + 1;
        let mut last_entry_valid = true;

        wal.replay(|entry| {
            if !last_entry_valid {
                return;
            }

            // Verify sequence continuity
            if entry.sequence != expected_sequence {
                last_entry_valid = false;
                return;
            }

            // Verify causal link
            if entry.sequence > 1 {
                if entry.parent_hash != last_hash {
                    last_entry_valid = false;
                    return;
                }
            }

            // Verify entry self-integrity
            let computed = compute_hash(entry);
            if entry.entry_hash != [0u8; 32] && entry.entry_hash != computed {
                // entry_hash of [0;32] means not yet computed (legacy entries)
                last_entry_valid = false;
                return;
            }

            last_hash = compute_hash(entry);
            expected_sequence += 1;
            state.push(entry.clone());
        })
        .map_err(|e| RecoveryError::WalReplayError {
            segment: 0,
            reason: e.to_string(),
        })?;

        if !last_entry_valid {
            return Err(RecoveryError::CausalIntegrityViolation {
                sequence: expected_sequence,
                expected: last_hash,
                found: [0u8; 32],
            });
        }

        Ok(state)
    }

    pub fn verify_chain(entries: &[OperationalEntry]) -> Result<(), RecoveryError> {
        if entries.is_empty() {
            return Ok(());
        }

        // Verify genesis
        if entries[0].sequence != 1 && entries[0].parent_hash != [0u8; 32] {
            return Err(RecoveryError::CausalIntegrityViolation {
                sequence: entries[0].sequence,
                expected: [0u8; 32],
                found: entries[0].parent_hash,
            });
        }

        // Verify chain
        for i in 1..entries.len() {
            let parent = &entries[i - 1];
            let child = &entries[i];

            let expected_parent_hash = compute_hash(parent);
            if child.parent_hash != expected_parent_hash {
                return Err(RecoveryError::CausalIntegrityViolation {
                    sequence: child.sequence,
                    expected: expected_parent_hash,
                    found: child.parent_hash,
                });
            }

            if child.sequence != parent.sequence + 1 {
                return Err(RecoveryError::CausalIntegrityViolation {
                    sequence: child.sequence,
                    expected: [0u8; 32],
                    found: [0u8; 32],
                });
            }
        }

        Ok(())
    }
}
