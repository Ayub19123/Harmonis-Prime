use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub struct VectorClock {
    pub node_id: String,
    pub clocks: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct GossipMessage {
    pub message_id: String,
    pub sender_id: String,
    pub payload: String,
    pub vector_clock: VectorClock,
    pub timestamp_ns: u128,
    pub ttl: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CausalOrder {
    Before,
    After,
    Concurrent,
}

pub struct FluidConsensusEngine {
    pub node_id: String,
    pub peers: Vec<String>,
    pub vector_clock: VectorClock,
    pub message_log: VecDeque<GossipMessage>,
    pub state: HashMap<String, String>,
    pub byzantine_threshold: usize,
}

impl VectorClock {
    pub fn new(node_id: &str) -> Self {
        let mut clocks = HashMap::new();
        clocks.insert(node_id.to_string(), 0);
        Self {
            node_id: node_id.to_string(),
            clocks,
        }
    }

    pub fn increment(&mut self) {
        let count = self.clocks.entry(self.node_id.clone()).or_insert(0);
        *count += 1;
    }

    pub fn merge(&mut self, other: &VectorClock) {
        for (node, time) in &other.clocks {
            let entry = self.clocks.entry(node.clone()).or_insert(0);
            if *time > *entry {
                *entry = *time;
            }
        }
    }

    pub fn compare(&self, other: &VectorClock) -> Option<CausalOrder> {
        let mut less = false;
        let mut greater = false;
        let all_nodes: HashSet<String> = self
            .clocks
            .keys()
            .chain(other.clocks.keys())
            .cloned()
            .collect();
        for node in all_nodes {
            let a = self.clocks.get(&node).unwrap_or(&0);
            let b = other.clocks.get(&node).unwrap_or(&0);
            if a < b {
                less = true;
            }
            if a > b {
                greater = true;
            }
        }
        if less && !greater {
            Some(CausalOrder::Before)
        } else if greater && !less {
            Some(CausalOrder::After)
        } else if !less && !greater {
            Some(CausalOrder::Concurrent)
        } else {
            None
        }
    }
}

impl FluidConsensusEngine {
    pub fn new(node_id: &str, peers: Vec<String>) -> Self {
        let n = peers.len() + 1;
        Self {
            node_id: node_id.to_string(),
            peers,
            vector_clock: VectorClock::new(node_id),
            message_log: VecDeque::with_capacity(10000),
            state: HashMap::new(),
            byzantine_threshold: n / 3,
        }
    }

    pub fn broadcast(&mut self, payload: &str) -> GossipMessage {
        self.vector_clock.increment();
        let msg = GossipMessage {
            message_id: format!("gossip_{}_{}", self.node_id, now_ns()),
            sender_id: self.node_id.clone(),
            payload: payload.to_string(),
            vector_clock: self.vector_clock.clone(),
            timestamp_ns: now_ns(),
            ttl: 7,
        };
        self.message_log.push_back(msg.clone());
        msg
    }

    pub fn receive(&mut self, msg: &GossipMessage) -> bool {
        if msg.ttl == 0 {
            return false;
        }
        self.vector_clock.merge(&msg.vector_clock);
        self.state
            .insert(msg.message_id.clone(), msg.payload.clone());
        true
    }

    pub fn is_byzantine_safe(&self, votes: &HashMap<String, bool>) -> bool {
        let total = votes.len();
        let yes = votes.values().filter(|&&v| v).count();
        let no = total - yes;
        let f = yes.min(no);
        f <= self.byzantine_threshold && total > 2 * self.byzantine_threshold
    }

    pub fn get_causal_state(&self, key: &str) -> Option<String> {
        self.state.get(key).cloned()
    }
}

fn now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
