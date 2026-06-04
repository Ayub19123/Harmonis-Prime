use super::qpu_engine::{QKDSession, QPUEngine, QuantumBackend};

pub struct QKDNetworkNode {
    pub node_id: String,
    pub engine: QPUEngine,
    pub key_registry: std::collections::HashMap<String, Vec<u8>>,
    pub session_log: Vec<QKDSession>,
}

#[derive(Debug, Clone)]
pub struct AuditTrailEntry {
    pub entry_id: String,
    pub timestamp_ns: u128,
    pub action: String,
    pub actor: String,
    pub data_hash: String,
    pub qkd_session_id: String,
}

pub struct QuantumAuditTrail {
    pub entries: Vec<AuditTrailEntry>,
    pub merkle_root: String,
}

impl QKDNetworkNode {
    pub fn new(node_id: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
            engine: QPUEngine::new(QuantumBackend::Simulated, 64),
            key_registry: std::collections::HashMap::new(),
            session_log: Vec::new(),
        }
    }

    pub fn establish_secure_channel(&mut self, peer_id: &str, key_length: usize) -> QKDSession {
        let session = self
            .engine
            .generate_qkd_key(&self.node_id, peer_id, key_length);
        self.key_registry
            .insert(peer_id.to_string(), session.sifted_key.clone());
        self.session_log.push(session.clone());
        session
    }

    pub fn verify_channel_integrity(&self, peer_id: &str) -> bool {
        if let Some(session) = self
            .session_log
            .iter()
            .rev()
            .find(|s| s.bob_node == peer_id)
        {
            !session.privacy_amplification_applied && session.error_rate < 0.11
        } else {
            false
        }
    }

    pub fn encrypt_trade_payload(&self, peer_id: &str, payload: &str) -> Option<Vec<u8>> {
        let key = self.key_registry.get(peer_id)?;
        let mut encrypted = Vec::new();
        for (i, byte) in payload.bytes().enumerate() {
            encrypted.push(byte ^ key[i % key.len()]);
        }
        Some(encrypted)
    }
}
impl QuantumAuditTrail {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            merkle_root: String::new(),
        }
    }

    pub fn append_entry(&mut self, action: &str, actor: &str, data_hash: &str, session_id: &str) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let entry = AuditTrailEntry {
            entry_id: format!(
                "audit_{}_{}",
                actor,
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ),
            timestamp_ns: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            action: action.to_string(),
            actor: actor.to_string(),
            data_hash: data_hash.to_string(),
            qkd_session_id: session_id.to_string(),
        };
        self.entries.push(entry);
        self.recompute_merkle();
    }

    fn recompute_merkle(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        for entry in &self.entries {
            entry.data_hash.hash(&mut hasher);
        }
        self.merkle_root = format!("merkle_{:x}", hasher.finish());
    }
}

pub struct FinancialQKDNetwork {
    pub nodes: std::collections::HashMap<String, QKDNetworkNode>,
    pub audit_trail: QuantumAuditTrail,
}

impl FinancialQKDNetwork {
    pub fn new() -> Self {
        Self {
            nodes: std::collections::HashMap::new(),
            audit_trail: QuantumAuditTrail::new(),
        }
    }

    pub fn register_node(&mut self, node_id: &str) {
        self.nodes
            .insert(node_id.to_string(), QKDNetworkNode::new(node_id));
    }

    pub fn execute_trade(&mut self, from: &str, to: &str, trade_hash: &str) -> bool {
        let node = self.nodes.get_mut(from).unwrap();
        let session = node.establish_secure_channel(to, 256);
        if session.privacy_amplification_applied {
            self.audit_trail
                .append_entry("TRADE_BLOCKED", from, trade_hash, &session.session_id);
            return false;
        }
        self.audit_trail
            .append_entry("TRADE_EXECUTED", from, trade_hash, &session.session_id);
        true
    }
}
