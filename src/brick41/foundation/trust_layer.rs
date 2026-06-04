use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// TrustLayer: Immutable, cryptographically-linked audit system
/// BRICK-41 Phase 1: Foundation — Trust + Memory + Ledger
/// Every entry is hashed, chained, and tamper-evident
#[derive(Debug, Clone)]
pub struct TrustLayer {
    chain: Vec<AuditEntry>,
    head_hash: String,
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp_ns: u128,
    pub event_type: String,
    pub actor: String,
    pub action: String,
    pub context: String,
    pub previous_hash: String,
    pub entry_hash: String,
    pub signature: String,
}

#[derive(Debug, Clone)]
pub struct TrustVerification {
    pub valid: bool,
    pub chain_length: usize,
    pub integrity_score: f64,
    pub tampered_indices: Vec<usize>,
}

impl TrustLayer {
    pub fn new() -> Self {
        let genesis = Self::create_genesis();
        let genesis_hash = genesis.entry_hash.clone();
        Self {
            chain: vec![genesis],
            head_hash: genesis_hash,
        }
    }

    fn create_genesis() -> AuditEntry {
        let timestamp = Self::now_ns();
        let hash = Self::hash_entry(&timestamp, "GENESIS", "system", "init", "origin", "0");
        AuditEntry {
            timestamp_ns: timestamp,
            event_type: "GENESIS".to_string(),
            actor: "system".to_string(),
            action: "init".to_string(),
            context: "origin".to_string(),
            previous_hash: "0".to_string(),
            entry_hash: hash.clone(),
            signature: hash,
        }
    }

    pub fn append(&mut self, actor: &str, action: &str, context: &str) -> &AuditEntry {
        let timestamp = Self::now_ns();
        let prev_hash = self.head_hash.clone();
        let hash = Self::hash_entry(&timestamp, "AUDIT", actor, action, context, &prev_hash);

        let entry = AuditEntry {
            timestamp_ns: timestamp,
            event_type: "AUDIT".to_string(),
            actor: actor.to_string(),
            action: action.to_string(),
            context: context.to_string(),
            previous_hash: prev_hash,
            entry_hash: hash.clone(),
            signature: hash.clone(),
        };

        self.chain.push(entry);
        self.head_hash = hash;
        self.chain.last().unwrap()
    }

    pub fn verify(&self) -> TrustVerification {
        let mut tampered = Vec::new();
        let mut valid = true;

        for i in 1..self.chain.len() {
            let prev = &self.chain[i - 1];
            let curr = &self.chain[i];

            if curr.previous_hash != prev.entry_hash {
                tampered.push(i);
                valid = false;
            }

            let expected_hash = Self::hash_entry(
                &curr.timestamp_ns,
                &curr.event_type,
                &curr.actor,
                &curr.action,
                &curr.context,
                &curr.previous_hash,
            );

            if curr.entry_hash != expected_hash {
                if !tampered.contains(&i) {
                    tampered.push(i);
                }
                valid = false;
            }
        }

        let integrity = if self.chain.len() > 1 {
            1.0 - (tampered.len() as f64 / (self.chain.len() - 1) as f64)
        } else {
            1.0
        };

        TrustVerification {
            valid,
            chain_length: self.chain.len(),
            integrity_score: integrity,
            tampered_indices: tampered,
        }
    }

    pub fn replay(&self, from_index: usize) -> Vec<AuditEntry> {
        self.chain.iter().skip(from_index).cloned().collect()
    }

    pub fn head(&self) -> &AuditEntry {
        self.chain.last().unwrap()
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    fn hash_entry(
        timestamp: &u128,
        event_type: &str,
        actor: &str,
        action: &str,
        context: &str,
        prev_hash: &str,
    ) -> String {
        let data = format!(
            "{}:{}:{}:{}:{}:{}",
            timestamp, event_type, actor, action, context, prev_hash
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn now_ns() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }
}
