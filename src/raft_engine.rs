use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub index: u64,
    pub term: u64,
    pub data: String,
    pub checksum: String,
}

#[allow(dead_code)]
pub struct RaftEngine {
    node_id: u64,
    peers: Vec<u64>,
    log: VecDeque<LogEntry>,
    commit_index: u64,
    last_applied: u64,
    current_term: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppendResult {
    pub success: bool,
    pub index: u64,
    pub term: u64,
}

impl RaftEngine {
    pub fn new(node_id: u64, peers: Vec<u64>) -> Self {
        RaftEngine {
            node_id,
            peers,
            log: VecDeque::new(),
            commit_index: 0,
            last_applied: 0,
            current_term: 1,
        }
    }

    pub fn append(&mut self, term: u64, data: &str) -> AppendResult {
        let index = self.log.len() as u64 + 1;
        let checksum = format!("{:x}", md5::compute(data));
        let entry = LogEntry {
            index,
            term,
            data: data.to_string(),
            checksum,
        };
        self.log.push_back(entry);
        self.current_term = term;
        AppendResult {
            success: true,
            index,
            term,
        }
    }

    pub fn commit(&mut self, up_to_index: u64) -> Vec<(u64, String)> {
        let mut committed = Vec::new();
        while self.commit_index < up_to_index && self.commit_index < self.log.len() as u64 {
            self.commit_index += 1;
            if let Some(entry) = self.log.get(self.commit_index as usize - 1) {
                committed.push((entry.index, entry.data.clone()));
            }
        }
        committed
    }

    pub fn get_commit_index(&self) -> u64 {
        self.commit_index
    }
    pub fn get_current_term(&self) -> u64 {
        self.current_term
    }

    pub fn get_log_len(&self) -> usize {
        self.log.len()
    }
}
