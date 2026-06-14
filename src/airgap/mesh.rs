//! Network mesh with partition simulation

use crate::airgap::node::PhysicalNode;
use std::collections::HashMap;

/// Network mesh connecting physical nodes
pub struct Mesh {
    nodes: HashMap<u64, PhysicalNode>,
    partitions: Vec<u64>, // Node IDs that are partitioned
}

impl Mesh {
    /// Create new mesh with given nodes
    pub fn new(nodes: Vec<PhysicalNode>) -> Self {
        let mut map = HashMap::new();
        for node in nodes {
            map.insert(node.id, node);
        }
        Self {
            nodes: map,
            partitions: Vec::new(),
        }
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get active (non-partitioned) node count
    pub fn active_node_count(&self) -> usize {
        self.nodes.len() - self.partitions.len()
    }

    /// Simulate network partition on specific node
    pub fn partition_node(&mut self, node_id: u64) -> Result<(), &'static str> {
        if !self.nodes.contains_key(&node_id) {
            return Err("Node not found in mesh");
        }
        if !self.partitions.contains(&node_id) {
            self.partitions.push(node_id);
            if let Some(node) = self.nodes.get_mut(&node_id) {
                node.deactivate_network();
            }
        }
        Ok(())
    }

    /// Reconnect partitioned node
    pub fn reconnect_node(&mut self, node_id: u64) -> Result<(), &'static str> {
        if !self.nodes.contains_key(&node_id) {
            return Err("Node not found in mesh");
        }
        self.partitions.retain(|&id| id != node_id);
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.activate_network();
        }
        Ok(())
    }

    /// Check if node is partitioned
    pub fn is_partitioned(&self, node_id: u64) -> bool {
        self.partitions.contains(&node_id)
    }

    /// Check if mesh has quorum (majority active)
    pub fn has_quorum(&self) -> bool {
        let total = self.nodes.len();
        let active = self.active_node_count();
        active > total / 2
    }
}
