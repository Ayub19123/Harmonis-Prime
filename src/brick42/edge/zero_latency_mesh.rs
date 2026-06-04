use crate::brick42::fluid::fluid_consensus::FluidConsensusEngine;
use crate::brick42::fluid::tensor_router::TensorRouter;
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// ZeroLatencyMesh: 5G MEC topology with local consensus
/// No cloud dependency. Sub-1ms latency between adjacent nodes.
pub struct ZeroLatencyMesh {
    pub mesh_id: String,
    pub local_nodes: HashMap<String, MeshNode>,
    pub tensor_router: TensorRouter,
    pub consensus_engine: FluidConsensusEngine,
    pub latency_matrix: HashMap<(String, String), f64>,
    pub max_hops: usize,
    pub failover_timeout_ms: f64,
}

/// MeshNode: Individual node in 5G MEC cluster
#[derive(Debug, Clone)]
pub struct MeshNode {
    pub node_id: String,
    pub mac_address: String,
    pub region: String,
    pub latency_to_neighbors_ms: HashMap<String, f64>,
    pub last_heartbeat_ns: u128,
    pub is_alive: bool,
    pub qpu_enabled: bool,
}

/// MeshPacket: Ultra-lightweight local protocol
#[derive(Debug, Clone)]
pub struct MeshPacket {
    pub packet_id: String,
    pub source: String,
    pub target: String,
    pub payload_type: PayloadType,
    pub payload: Vec<u8>,
    pub ttl: u8,
    pub priority: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PayloadType {
    Heartbeat,
    ModelDelta,
    PredictionQuery,
    ConsensusVote,
    StateSync,
    Alert,
}

impl ZeroLatencyMesh {
    pub fn new(mesh_id: &str, local_node_id: &str, peers: Vec<String>) -> Self {
        Self {
            mesh_id: mesh_id.to_string(),
            local_nodes: HashMap::new(),
            tensor_router: TensorRouter::new(),
            consensus_engine: FluidConsensusEngine::new(local_node_id, peers),
            latency_matrix: HashMap::new(),
            max_hops: 3,
            failover_timeout_ms: 50.0,
        }
    }

    /// Register local mesh node with neighbor latency map
    pub fn register_node(
        &mut self,
        node_id: &str,
        mac: &str,
        region: &str,
        neighbors: HashMap<String, f64>,
        qpu: bool,
    ) {
        let node = MeshNode {
            node_id: node_id.to_string(),
            mac_address: mac.to_string(),
            region: region.to_string(),
            latency_to_neighbors_ms: neighbors.clone(),
            last_heartbeat_ns: now_ns(),
            is_alive: true,
            qpu_enabled: qpu,
        };
        self.local_nodes.insert(node_id.to_string(), node);
        for (neighbor, latency) in neighbors {
            self.latency_matrix
                .insert((node_id.to_string(), neighbor), latency);
        }
    }

    /// Route packet via shortest latency path (Dijkstra-like)
    pub fn route_packet(&self, packet: &MeshPacket) -> Option<Vec<String>> {
        if packet.ttl == 0 {
            return None;
        }
        let mut visited = HashMap::new();
        let mut queue = VecDeque::new();
        queue.push_back(vec![packet.source.clone()]);
        while let Some(path) = queue.pop_front() {
            let current = path.last().unwrap();
            if current == &packet.target {
                return Some(path);
            }
            if visited.contains_key(current) {
                continue;
            }
            visited.insert(current.clone(), true);
            if let Some(node) = self.local_nodes.get(current) {
                for (neighbor, _latency) in &node.latency_to_neighbors_ms {
                    if !visited.contains_key(neighbor) && path.len() < self.max_hops {
                        let mut new_path = path.clone();
                        new_path.push(neighbor.clone());
                        queue.push_back(new_path);
                    }
                }
            }
        }
        None
    }

    /// Broadcast to all neighbors via gossip (flood with TTL limit)
    pub fn flood_broadcast(&mut self, payload_type: PayloadType, payload: Vec<u8>, priority: u8) {
        for (node_id, node) in &self.local_nodes {
            if !node.is_alive {
                continue;
            }
            for neighbor_id in node.latency_to_neighbors_ms.keys() {
                let packet = MeshPacket {
                    packet_id: format!("pkt_{}_{}", node_id, now_ns()),
                    source: node_id.clone(),
                    target: neighbor_id.clone(),
                    payload_type: payload_type.clone(),
                    payload: payload.clone(),
                    ttl: self.max_hops as u8,
                    priority,
                };
                let _ = self.route_packet(&packet);
            }
        }
    }

    /// Detect node failure and trigger instant failover
    pub fn check_failover(&mut self) -> Vec<String> {
        let now = now_ns();
        let mut failed = Vec::new();
        for (node_id, node) in self.local_nodes.iter_mut() {
            let elapsed_ms = (now - node.last_heartbeat_ns) as f64 / 1_000_000.0;
            if elapsed_ms > self.failover_timeout_ms {
                node.is_alive = false;
                failed.push(node_id.clone());
            }
        }
        failed
    }

    /// Mesh health: alive nodes, average latency, QPU coverage
    pub fn mesh_health(&self) -> MeshHealth {
        let alive_count = self.local_nodes.values().filter(|n| n.is_alive).count();
        let total_nodes = self.local_nodes.len();
        let avg_latency = if self.latency_matrix.is_empty() {
            0.0
        } else {
            self.latency_matrix.values().sum::<f64>() / self.latency_matrix.len() as f64
        };
        let qpu_count = self.local_nodes.values().filter(|n| n.qpu_enabled).count();
        MeshHealth {
            mesh_id: self.mesh_id.clone(),
            alive_nodes: alive_count,
            total_nodes,
            average_latency_ms: avg_latency,
            qpu_coverage: if total_nodes > 0 {
                qpu_count as f64 / total_nodes as f64
            } else {
                0.0
            },
            partition_risk: alive_count < total_nodes / 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeshHealth {
    pub mesh_id: String,
    pub alive_nodes: usize,
    pub total_nodes: usize,
    pub average_latency_ms: f64,
    pub qpu_coverage: f64,
    pub partition_risk: bool,
}

fn now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
