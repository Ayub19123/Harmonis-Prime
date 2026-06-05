//! BRICK-42: Quantum-Incorporated Sovereign Intelligence
//! Hybrid quantum-classical computation + fluid architecture + edge-native cognition + bio-mimetic resilience

pub mod edge;
pub mod fluid;
pub mod quantum;
pub mod resilience;

pub use edge::edge_node::{
    EdgeHealth, EdgeNode, FeatureVector, FederatedRound, LocalModel, ModelDelta,
};
pub use edge::predictive_engine::{PredictionResult, PredictiveEngine, StateSnapshot};
pub use edge::zero_latency_mesh::{MeshHealth, MeshNode, MeshPacket, PayloadType, ZeroLatencyMesh};
pub use fluid::agent_swarm::{
    ConsensusStatus, LiquiditySignal, SwarmAgent, SwarmConsensus, TradeExecution,
};
pub use fluid::fluid_consensus::{CausalOrder, FluidConsensusEngine, GossipMessage, VectorClock};
pub use fluid::tensor_router::{
    NetworkNode, RouteMetrics, TensorDimensions, TensorPacket, TensorRouter,
};
pub use quantum::annealing::{QuantumAnnealingSolver, RouteNode, RoutePlan};
pub use quantum::qkd::{AuditTrailEntry, FinancialQKDNetwork, QKDNetworkNode, QuantumAuditTrail};
pub use quantum::qpu_engine::{
    AnsatzType, Basis, Complex64, Constraint, ConstraintType, FeatureMapType, OpType,
    OptimizationProblem, QKDSession, QMLModel, QPUEngine, QuantumBackend, QuantumOperation,
    QuantumState,
};
