//! SET-5.1: Sovereign Node — Individual node in distributed simulation
//! Invariant: Each node maintains local DAG consistency with global convergence

use std::time::Instant;
use crate::mesh::dag::{CognitiveMesh, Message, MessageId, NodeId, DagReceipt, DagError};
use crate::thermo::entropy::{EntropyTracker, ThermodynamicState};

/// Node state in the distributed simulation
#[derive(Debug, Clone)]
pub struct NodeState {
    pub node_id: NodeId,
    pub mesh: CognitiveMesh,
    pub entropy_tracker: EntropyTracker,
    pub message_log: Vec<MessageId>,
    pub last_heartbeat: Instant,
    pub is_byzantine: bool,
    pub is_offline: bool,
}

/// Telemetry from a single node operation
#[derive(Debug, Clone)]
pub struct NodeTelemetry {
    pub node_id: NodeId,
    pub operation: String,
    pub latency_micros: u64,
    pub entropy_delta: f64,
    pub success: bool,
    pub timestamp: Instant,
}

impl NodeState {
    pub fn new(node_id: NodeId, genesis: Message) -> Result<Self, crate::thermo::entropy::ThermoError> {
        let mesh = CognitiveMesh::new(genesis).map_err(|_| {
            crate::thermo::entropy::ThermoError::InvalidTemperature(0.0) // placeholder
        })?;
        
        Ok(Self {
            node_id,
            mesh,
            entropy_tracker: EntropyTracker::new(300.0)?,
            message_log: Vec::new(),
            last_heartbeat: Instant::now(),
            is_byzantine: false,
            is_offline: false,
        })
    }

    /// Append message to local DAG with telemetry
    pub fn process_message(&mut self, msg: Message) -> Result<(DagReceipt, NodeTelemetry), DagError> {
        let start = Instant::now();
        
        let result = self.mesh.append_message(msg);
        let latency = start.elapsed();
        
        let telemetry = NodeTelemetry {
            node_id: self.node_id,
            operation: "dag_append".to_string(),
            latency_micros: latency.as_micros() as u64,
            entropy_delta: 0.0, // populated by caller
            success: result.is_ok(),
            timestamp: Instant::now(),
        };
        
        if let Ok(ref receipt) = result {
            self.message_log.push(receipt.message_id);
            self.last_heartbeat = Instant::now();
        }
        
        result.map(|r| (r, telemetry))
    }

    /// Record entropy state for thermodynamic tracking
    pub fn record_entropy(&mut self, state: ThermodynamicState) -> Result<(), crate::thermo::entropy::ThermoError> {
        self.entropy_tracker.record(state)?;
        Ok(())
    }

    /// Simulate Byzantine behavior: flip message payload
    pub fn byzantine_corrupt(&self, mut msg: Message) -> Message {
        if self.is_byzantine {
            for byte in msg.payload.iter_mut() {
                *byte = byte.wrapping_mul(7).wrapping_add(13);
            }
        }
        msg
    }
}