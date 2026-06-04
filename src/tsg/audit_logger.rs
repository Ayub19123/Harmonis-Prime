use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub operation_id: String,
    pub action_type: String,
    pub authorized: bool,
    pub constraint_hash: String,
    pub pilot_override: bool,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct AuditLedger {
    pub entries: Vec<AuditEntry>,
    pub ledger_hash: String,
    pub epoch: u64,
}

pub struct AuditLogger {
    pub entries: Vec<AuditEntry>,
    pub strict_mode: bool,
}

impl AuditLogger {
    pub fn new(strict_mode: bool) -> Self {
        Self {
            entries: Vec::new(),
            strict_mode,
        }
    }

    pub fn log(
        &mut self,
        operation_id: String,
        action_type: String,
        authorized: bool,
        constraint_hash: String,
    ) -> &AuditEntry {
        let entry = AuditEntry {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            operation_id,
            action_type,
            authorized,
            constraint_hash,
            pilot_override: false,
            metadata: HashMap::new(),
        };
        self.entries.push(entry);
        self.entries.last().unwrap()
    }

    pub fn log_with_override(
        &mut self,
        operation_id: String,
        action_type: String,
        authorized: bool,
        constraint_hash: String,
        pilot_name: String,
    ) -> &AuditEntry {
        let mut entry = AuditEntry {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            operation_id,
            action_type,
            authorized,
            constraint_hash,
            pilot_override: true,
            metadata: HashMap::new(),
        };
        entry
            .metadata
            .insert(String::from("pilot_override"), pilot_name);
        self.entries.push(entry);
        self.entries.last().unwrap()
    }

    pub fn seal_ledger(&self) -> AuditLedger {
        let combined: String = self
            .entries
            .iter()
            .map(|e| format!("{}{}{}", e.operation_id, e.timestamp, e.constraint_hash))
            .collect();
        let hash = format!(
            "{:064x}",
            combined
                .as_bytes()
                .iter()
                .fold(0u64, |a, b| a.wrapping_add(*b as u64))
        );

        AuditLedger {
            entries: self.entries.clone(),
            ledger_hash: hash,
            epoch: self.entries.len() as u64,
        }
    }

    pub fn verify_integrity(&self, ledger: &AuditLedger) -> bool {
        let recomputed = self.seal_ledger();
        recomputed.ledger_hash == ledger.ledger_hash && recomputed.epoch == ledger.epoch
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn last_entry(&self) -> Option<&AuditEntry> {
        self.entries.last()
    }
}
