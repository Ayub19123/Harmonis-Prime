//! HBS-1.2: DAG-Enforced Mesh Topology
//! Invariant: G = (V, E) is a directed acyclic graph at all times
//! Violation of acyclicity is a system-halting error

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::is_cyclic_directed;
use std::collections::HashMap;
use std::time::Instant;

/// Unique identifier for mesh nodes (peers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

/// Unique identifier for messages in the DAG
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageId(pub u64);

/// Weighted edge representing a causal dependency
#[derive(Debug, Clone)]
pub struct CausalLink {
    pub latency_micros: u64,
    pub entropy_delta: f64,
}

/// A message in the cognitive mesh
#[derive(Debug, Clone)]
pub struct Message {
    pub id: MessageId,
    pub payload: Vec<u8>,
    pub parents: Vec<MessageId>,
    pub timestamp: Instant,
    pub source: NodeId,
}

/// Receipt proving successful DAG insertion
#[derive(Debug, Clone)]
pub struct DagReceipt {
    pub message_id: MessageId,
    pub insertion_time: Instant,
    pub path_depth: usize,
    pub validation_duration_micros: u64,
}

/// Core invariant violation
#[derive(Debug, thiserror::Error)]
pub enum DagError {
    #[error("Cycle detected: message {offending_message:?} would create cycle via parents {proposed_parents:?}")]
    CycleViolation {
        offending_message: MessageId,
        proposed_parents: Vec<MessageId>,
    },
    #[error("Parent not found: {0:?}")]
    ParentNotFound(MessageId),
    #[error("Duplicate message: {0:?}")]
    DuplicateMessage(MessageId),
}

#[derive(Debug, Default)]
pub struct MeshMetrics {
    pub total_messages: u64,
    pub total_rejections: u64,
    pub max_depth_observed: usize,
    pub avg_insertion_latency_micros: f64,
}

/// The cognitive mesh — DAG-enforced peer-to-peer topology
pub struct CognitiveMesh {
    graph: DiGraph<MessageId, CausalLink>,
    node_indices: HashMap<MessageId, NodeIndex>,
    genesis: MessageId,
    metrics: MeshMetrics,
}

impl CognitiveMesh {
    pub fn new(genesis: Message) -> Result<Self, DagError> {
        let mut graph = DiGraph::new();
        let genesis_idx = graph.add_node(genesis.id);
        
        let mut node_indices = HashMap::new();
        node_indices.insert(genesis.id, genesis_idx);
        
        Ok(Self {
            graph,
            node_indices,
            genesis: genesis.id,
            metrics: MeshMetrics::default(),
        })
    }
    
    /// CORE INVARIANT: insert only if acyclicity is preserved
    pub fn append_message(&mut self, msg: Message) -> Result<DagReceipt, DagError> {
        let start = Instant::now();
        
        // Rule 1: No duplicates
        if self.node_indices.contains_key(&msg.id) {
            return Err(DagError::DuplicateMessage(msg.id));
        }
        
        // Rule 2: All parents must exist
        for parent in &msg.parents {
            if !self.node_indices.contains_key(parent) {
                return Err(DagError::ParentNotFound(*parent));
            }
        }
        
        // Rule 3: Pre-check acyclicity on clone
        let mut test_graph = self.graph.clone();
        let msg_idx = test_graph.add_node(msg.id);
        
        for parent in &msg.parents {
            let parent_idx = self.node_indices[parent];
            test_graph.add_edge(parent_idx, msg_idx, CausalLink {
                latency_micros: 0,
                entropy_delta: 0.0,
            });
        }
        
        if is_cyclic_directed(&test_graph) {
            self.metrics.total_rejections += 1;
            return Err(DagError::CycleViolation {
                offending_message: msg.id,
                proposed_parents: msg.parents.clone(),
            });
        }
        
        // COMMIT: Graph is verified acyclic
        let committed_idx = self.graph.add_node(msg.id);
        for parent in &msg.parents {
            let parent_idx = self.node_indices[parent];
            self.graph.add_edge(parent_idx, committed_idx, CausalLink {
                latency_micros: start.elapsed().as_micros() as u64,
                entropy_delta: 0.0,
            });
        }
        
        self.node_indices.insert(msg.id, committed_idx);
        self.metrics.total_messages += 1;
        
        let path_depth = self.longest_path_to(msg.id);
        if path_depth > self.metrics.max_depth_observed {
            self.metrics.max_depth_observed = path_depth;
        }
        
        let elapsed = start.elapsed();
        self.update_avg_latency(elapsed.as_micros() as f64);
        
        Ok(DagReceipt {
            message_id: msg.id,
            insertion_time: Instant::now(),
            path_depth,
            validation_duration_micros: elapsed.as_micros() as u64,
        })
    }
    
    /// Compute longest path from genesis to target message
    fn longest_path_to(&self, target: MessageId) -> usize {
        let target_idx = match self.node_indices.get(&target) {
            Some(idx) => *idx,
            None => return 0,
        };
        
        if target == self.genesis {
            return 0;
        }
        
        let mut max_depth = 0;
        for parent in self.graph.neighbors_directed(target_idx, petgraph::Direction::Incoming) {
            let parent_id = self.graph[parent];
            let parent_depth = self.longest_path_to(parent_id);
            max_depth = max_depth.max(parent_depth + 1);
        }
        
        max_depth
    }
    
    fn update_avg_latency(&mut self, new_latency: f64) {
        let n = self.metrics.total_messages as f64;
        let old_avg = self.metrics.avg_insertion_latency_micros;
        self.metrics.avg_insertion_latency_micros = 
            (old_avg * (n - 1.0) + new_latency) / n;
    }
    
    /// INVARIANT CHECK: Verify graph is acyclic (expensive, for tests only)
    pub fn verify_acyclic(&self) -> bool {
        !is_cyclic_directed(&self.graph)
    }
    
    pub fn metrics(&self) -> &MeshMetrics {
        &self.metrics
    }
}




