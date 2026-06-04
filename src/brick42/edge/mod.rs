pub mod edge_node;
pub mod predictive_engine;
pub mod zero_latency_mesh;

pub use edge_node::{EdgeHealth, EdgeNode, FeatureVector, FederatedRound, LocalModel, ModelDelta};
pub use predictive_engine::{PredictionResult, PredictiveEngine, StateSnapshot};
pub use zero_latency_mesh::{MeshHealth, MeshNode, MeshPacket, PayloadType, ZeroLatencyMesh};
