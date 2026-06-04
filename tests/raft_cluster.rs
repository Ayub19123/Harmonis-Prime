use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct InMemoryTransport {
    pub node_id: u64,
    pub inbox: Arc<Mutex<VecDeque<RaftMessage>>>,
    pub peers: HashMap<u64, Arc<Mutex<VecDeque<RaftMessage>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RaftMessage {
    AppendEntries {
        term: u64,
        leader_id: u64,
        prev_log_index: u64,
        prev_log_term: u64,
        entries: Vec<LogEntry>,
        leader_commit: u64,
    },
    AppendEntriesResponse {
        term: u64,
        success: bool,
        match_index: u64,
    },
    RequestVote {
        term: u64,
        candidate_id: u64,
        last_log_index: u64,
        last_log_term: u64,
    },
    RequestVoteResponse {
        term: u64,
        vote_granted: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub index: u64,
    pub term: u64,
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct RaftNode {
    pub node_id: u64,
    pub current_term: u64,
    pub voted_for: Option<u64>,
    pub log: Vec<LogEntry>,
    pub commit_index: u64,
    pub last_applied: u64,
    pub state: NodeState,
    pub transport: InMemoryTransport,
    pub votes_received: u64,
    pub election_timeout: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeState {
    Follower,
    Candidate,
    Leader,
}

impl RaftNode {
    pub fn new(node_id: u64, transport: InMemoryTransport) -> Self {
        Self {
            node_id,
            current_term: 0,
            voted_for: None,
            log: vec![LogEntry {
                index: 0,
                term: 0,
                command: "genesis".to_string(),
            }],
            commit_index: 0,
            last_applied: 0,
            state: NodeState::Follower,
            transport,
            votes_received: 0,
            election_timeout: Instant::now() + Duration::from_millis(150 + (node_id * 50) as u64),
        }
    }

    pub fn propose(&mut self, command: &str) -> Option<LogEntry> {
        if self.state != NodeState::Leader {
            return None;
        }
        let entry = LogEntry {
            index: self.log.len() as u64,
            term: self.current_term,
            command: command.to_string(),
        };
        self.log.push(entry.clone());
        self.replicate_to_peers();
        Some(entry)
    }

    pub fn replicate_to_peers(&mut self) {
        for (_peer_id, inbox) in &self.transport.peers {
            let msg = RaftMessage::AppendEntries {
                term: self.current_term,
                leader_id: self.node_id,
                prev_log_index: self.log.len() as u64 - 1,
                prev_log_term: self.log.last().unwrap().term,
                entries: self.log.clone(),
                leader_commit: self.commit_index,
            };
            inbox.lock().unwrap().push_back(msg);
        }
    }

    pub fn handle_message(&mut self, msg: RaftMessage) {
        match msg {
            RaftMessage::AppendEntries {
                term,
                entries,
                leader_commit,
                ..
            } => {
                if term >= self.current_term {
                    self.current_term = term;
                    self.state = NodeState::Follower;
                    self.voted_for = None;
                    self.election_timeout =
                        Instant::now() + Duration::from_millis(150 + (self.node_id * 50) as u64);
                    if entries.len() > self.log.len() {
                        self.log = entries.clone();
                    }
                    if leader_commit > self.commit_index {
                        self.commit_index = leader_commit.min(self.log.len() as u64 - 1);
                    }
                }
            }
            RaftMessage::RequestVote {
                term,
                candidate_id,
                last_log_index,
                last_log_term,
            } => {
                if term > self.current_term {
                    self.current_term = term;
                    self.voted_for = None;
                    self.state = NodeState::Follower;
                }
                let vote_granted = if term >= self.current_term
                    && (self.voted_for.is_none() || self.voted_for == Some(candidate_id))
                    && (last_log_index >= self.log.len() as u64 - 1)
                    && (last_log_term >= self.log.last().unwrap().term)
                {
                    self.voted_for = Some(candidate_id);
                    true
                } else {
                    false
                };
                let response = RaftMessage::RequestVoteResponse {
                    term: self.current_term,
                    vote_granted,
                };
                if let Some(inbox) = self.transport.peers.get(&candidate_id) {
                    inbox.lock().unwrap().push_back(response);
                }
            }
            RaftMessage::RequestVoteResponse { term, vote_granted } => {
                if term > self.current_term {
                    self.current_term = term;
                    self.state = NodeState::Follower;
                    self.voted_for = None;
                    return;
                }
                if vote_granted && self.state == NodeState::Candidate {
                    self.votes_received += 1;
                }
            }
            _ => {}
        }
    }

    pub fn start_election(&mut self) {
        self.state = NodeState::Candidate;
        self.current_term += 1;
        self.voted_for = Some(self.node_id);
        self.votes_received = 1;
        self.election_timeout =
            Instant::now() + Duration::from_millis(150 + (self.node_id * 50) as u64);
        for (_peer_id, inbox) in &self.transport.peers {
            let msg = RaftMessage::RequestVote {
                term: self.current_term,
                candidate_id: self.node_id,
                last_log_index: self.log.len() as u64 - 1,
                last_log_term: self.log.last().unwrap().term,
            };
            inbox.lock().unwrap().push_back(msg);
        }
    }

    pub fn check_election_timeout(&mut self) -> bool {
        if Instant::now() > self.election_timeout && self.state != NodeState::Leader {
            self.start_election();
            true
        } else {
            false
        }
    }

    pub fn check_majority(&mut self, cluster_size: usize) -> bool {
        if self.state == NodeState::Candidate
            && self.votes_received >= (cluster_size / 2 + 1) as u64
        {
            self.state = NodeState::Leader;
            self.votes_received = 0;
            true
        } else {
            false
        }
    }
}

pub struct TestCluster {
    pub nodes: HashMap<u64, RaftNode>,
    pub transports: HashMap<u64, InMemoryTransport>,
}

impl TestCluster {
    pub fn new(size: usize) -> Self {
        let mut transports = HashMap::new();
        let mut inboxes = HashMap::new();
        for i in 0..size {
            let inbox = Arc::new(Mutex::new(VecDeque::new()));
            inboxes.insert(i as u64, inbox);
        }
        for i in 0..size {
            let mut peers = HashMap::new();
            for j in 0..size {
                if i != j {
                    peers.insert(j as u64, inboxes.get(&(j as u64)).unwrap().clone());
                }
            }
            let transport = InMemoryTransport {
                node_id: i as u64,
                inbox: inboxes.get(&(i as u64)).unwrap().clone(),
                peers,
            };
            transports.insert(i as u64, transport);
        }
        let mut nodes = HashMap::new();
        for i in 0..size {
            let node = RaftNode::new(i as u64, transports.get(&(i as u64)).unwrap().clone());
            nodes.insert(i as u64, node);
        }
        Self { nodes, transports }
    }

    pub fn tick_node(&mut self, node_id: u64) {
        let node = self.nodes.get_mut(&node_id).unwrap();
        let inbox = node.transport.inbox.clone();
        let messages: Vec<RaftMessage> = inbox.lock().unwrap().drain(..).collect();
        for msg in messages {
            node.handle_message(msg);
        }
        node.check_election_timeout();
    }

    pub fn tick_all(&mut self) {
        let ids: Vec<u64> = self.nodes.keys().cloned().collect();
        for id in ids {
            self.tick_node(id);
        }
    }

    pub fn propose_to_leader(&mut self, command: &str) -> Option<LogEntry> {
        for node in self.nodes.values_mut() {
            if node.state == NodeState::Leader {
                return node.propose(command);
            }
        }
        None
    }

    pub fn get_leader(&self) -> Option<u64> {
        for (id, node) in &self.nodes {
            if node.state == NodeState::Leader {
                return Some(*id);
            }
        }
        None
    }

    pub fn are_logs_consistent(&self) -> bool {
        let first_log = self.nodes.values().next().unwrap().log.clone();
        for node in self.nodes.values() {
            if node.log != first_log {
                return false;
            }
        }
        true
    }

    pub fn force_leader(&mut self, node_id: u64) {
        for (id, node) in self.nodes.iter_mut() {
            if *id == node_id {
                node.state = NodeState::Leader;
                node.current_term += 1;
            } else {
                node.state = NodeState::Follower;
                node.current_term = 0;
            }
        }
    }
}

#[cfg(test)]
mod raft_cluster_tests {
    use super::*;

    #[test]
    fn test_3_node_leader_election() {
        let mut cluster = TestCluster::new(3);
        cluster.nodes.get_mut(&0).unwrap().start_election();
        cluster.tick_all();
        cluster.tick_all();
        let node0 = cluster.nodes.get_mut(&0).unwrap();
        assert!(
            node0.check_majority(3),
            "Node 0 should win election with majority"
        );
        assert_eq!(node0.state, NodeState::Leader);
        println!("TEST 1: Leader election - Node 0 elected with majority");
    }

    #[test]
    fn test_log_replication_across_cluster() {
        let mut cluster = TestCluster::new(3);
        cluster.force_leader(0);
        let entry = cluster.propose_to_leader("config_update_v1").unwrap();
        assert_eq!(entry.command, "config_update_v1");
        cluster.tick_all();
        cluster.tick_all();
        assert!(
            cluster.are_logs_consistent(),
            "All nodes should have identical logs"
        );
        for node in cluster.nodes.values() {
            assert!(
                node.log.iter().any(|e| e.command == "config_update_v1"),
                "Node {} should have config_update_v1",
                node.node_id
            );
        }
        println!("TEST 2: Log replication - command replicated to all 3 nodes");
    }

    #[test]
    fn test_leader_failover() {
        let mut cluster = TestCluster::new(3);
        cluster.force_leader(0);
        assert_eq!(cluster.get_leader(), Some(0));
        cluster.nodes.get_mut(&0).unwrap().state = NodeState::Follower;
        cluster.nodes.get_mut(&1).unwrap().start_election();
        cluster.tick_all();
        cluster.tick_all();
        let node1 = cluster.nodes.get_mut(&1).unwrap();
        assert!(
            node1.check_majority(3),
            "Node 1 should win failover election"
        );
        assert_eq!(node1.state, NodeState::Leader);
        println!("TEST 3: Leader failover - Node 0 fails, Node 1 elected");
    }

    #[test]
    fn test_replay_correctness_from_genesis() {
        let mut cluster = TestCluster::new(3);
        cluster.force_leader(0);
        cluster.propose_to_leader("trade_batch_A");
        cluster.tick_all();
        cluster.propose_to_leader("trade_batch_B");
        cluster.tick_all();
        cluster.propose_to_leader("compliance_check");
        cluster.tick_all();
        assert!(cluster.are_logs_consistent());
        let leader = cluster.nodes.get(&0).unwrap();
        assert_eq!(leader.log.len(), 4);
        assert_eq!(leader.log[1].command, "trade_batch_A");
        assert_eq!(leader.log[2].command, "trade_batch_B");
        assert_eq!(leader.log[3].command, "compliance_check");
        let replayed: Vec<String> = leader
            .log
            .iter()
            .skip(1)
            .map(|e| e.command.clone())
            .collect();
        assert_eq!(
            replayed,
            vec!["trade_batch_A", "trade_batch_B", "compliance_check"]
        );
        println!("TEST 4: Replay correctness - deterministic replay from genesis");
    }
}
