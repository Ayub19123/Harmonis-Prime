use crate::brick42::quantum::qpu_engine::{QPUEngine, QuantumBackend};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub struct TensorDimensions {
    pub batch: usize,
    pub features: usize,
    pub sequence: usize,
    pub priority: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteMetrics {
    pub congestion: f64,
    pub latency_ms: f64,
    pub security_score: f64,
    pub quantum_confidence: f64,
}

#[derive(Debug, Clone)]
pub struct TensorPacket {
    pub packet_id: String,
    pub source_node: String,
    pub target_node: String,
    pub dimensions: TensorDimensions,
    pub payload: Vec<f64>,
    pub route_path: Vec<String>,
    pub timestamp_ns: u128,
    pub ttl: u32,
}

#[derive(Debug, Clone)]
pub struct NetworkNode {
    pub node_id: String,
    pub region: String,
    pub metrics: RouteMetrics,
    pub neighbors: Vec<String>,
    pub qpu_available: bool,
}

pub struct TensorRouter {
    pub nodes: HashMap<String, NetworkNode>,
    pub routing_table: HashMap<String, Vec<String>>,
    pub packet_log: VecDeque<TensorPacket>,
    pub qpu_engine: QPUEngine,
}

impl TensorRouter {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            routing_table: HashMap::new(),
            packet_log: VecDeque::with_capacity(10000),
            qpu_engine: QPUEngine::new(QuantumBackend::Simulated, 64),
        }
    }

    pub fn register_node(
        &mut self,
        node_id: &str,
        region: &str,
        metrics: RouteMetrics,
        neighbors: Vec<String>,
        qpu: bool,
    ) {
        let node = NetworkNode {
            node_id: node_id.to_string(),
            region: region.to_string(),
            metrics,
            neighbors,
            qpu_available: qpu,
        };
        self.nodes.insert(node_id.to_string(), node);
        self.recompute_routes();
    }

    pub fn route_tensor(&mut self, packet: TensorPacket) -> Vec<String> {
        let mut path = vec![packet.source_node.clone()];
        let mut current = packet.source_node.clone();
        let mut visited = HashMap::new();
        visited.insert(current.clone(), true);
        while current != packet.target_node && path.len() < packet.ttl as usize {
            let next_hop =
                self.select_next_hop(&current, &packet.target_node, &packet.dimensions, &visited);
            match next_hop {
                Some(node) => {
                    path.push(node.clone());
                    visited.insert(node.clone(), true);
                    current = node;
                }
                None => break,
            }
        }
        let mut logged = packet;
        logged.route_path = path.clone();
        logged.timestamp_ns = now_ns();
        self.packet_log.push_back(logged);
        path
    }

    fn select_next_hop(
        &self,
        current: &str,
        _target: &str,
        dims: &TensorDimensions,
        visited: &HashMap<String, bool>,
    ) -> Option<String> {
        let current_node = self.nodes.get(current)?;
        let mut best_neighbor: Option<String> = None;
        let mut best_score = f64::INFINITY;
        for neighbor_id in &current_node.neighbors {
            if visited.contains_key(neighbor_id) {
                continue;
            }
            let neighbor = match self.nodes.get(neighbor_id) {
                Some(n) => n,
                None => continue,
            };
            let congestion_cost = neighbor.metrics.congestion * 0.35;
            let latency_cost = neighbor.metrics.latency_ms * 0.25;
            let security_cost = (1.0 - neighbor.metrics.security_score) * 0.20;
            let quantum_boost = if neighbor.qpu_available {
                neighbor.metrics.quantum_confidence * 0.20
            } else {
                0.0
            };
            let priority_penalty = if dims.priority > 7 { -0.15 } else { 0.0 };
            let score =
                congestion_cost + latency_cost + security_cost - quantum_boost + priority_penalty;
            if score < best_score {
                best_score = score;
                best_neighbor = Some(neighbor_id.clone());
            }
        }
        best_neighbor
    }

    pub fn recompute_routes(&mut self) {
        self.routing_table.clear();
        for (node_id, node) in &self.nodes {
            let mut routes = Vec::new();
            for neighbor_id in &node.neighbors {
                if self.nodes.contains_key(neighbor_id) {
                    routes.push(neighbor_id.clone());
                }
            }
            self.routing_table.insert(node_id.clone(), routes);
        }
    }

    pub fn get_mesh_health(&self) -> f64 {
        if self.nodes.is_empty() {
            return 0.0;
        }
        let total_score: f64 = self
            .nodes
            .values()
            .map(|n| {
                n.metrics.security_score
                    + (1.0 - n.metrics.congestion)
                    + (100.0 / n.metrics.latency_ms.max(1.0))
            })
            .sum();
        total_score / self.nodes.len() as f64
    }
}

fn now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
