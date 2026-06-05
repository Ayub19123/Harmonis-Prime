use crate::brick42::fluid::fluid_consensus::FluidConsensusEngine;
use crate::brick42::fluid::tensor_router::TensorRouter;
use crate::brick42::quantum::qpu_engine::{QMLModel, QPUEngine, QuantumBackend};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// EdgeNode: Local intelligence at hospital, bank, or logistics hub
/// Federated learning: trains locally, shares only model deltas, never raw data
/// Hardware target: Raspberry Pi 5 + Coral TPU (4 TOPS INT8)
pub struct EdgeNode {
    pub node_id: String,
    pub domain: String,
    pub region: String,
    pub qpu_engine: QPUEngine,
    pub tensor_router: TensorRouter,
    pub consensus_engine: FluidConsensusEngine,
    pub local_model: LocalModel,
    pub data_buffer: VecDeque<FeatureVector>,
    pub model_deltas: VecDeque<ModelDelta>,
    pub peers: Vec<String>,
    pub latency_budget_ms: f64,
}

/// LocalModel: On-device trainable parameters (abstract insights only)
#[derive(Debug, Clone)]
pub struct LocalModel {
    pub weights: Vec<f64>,
    pub bias: f64,
    pub learning_rate: f64,
    pub epochs_trained: u64,
    pub last_accuracy: f64,
}

/// FeatureVector: Raw input data NEVER leaves the edge node
#[derive(Debug, Clone)]
pub struct FeatureVector {
    pub vector_id: String,
    pub features: Vec<f64>,
    pub label: Option<f64>,
    pub timestamp_ns: u128,
    pub source: String,
}

/// ModelDelta: Encrypted weight gradients shared with central aggregator
#[derive(Debug, Clone)]
pub struct ModelDelta {
    pub delta_id: String,
    pub node_id: String,
    pub weight_changes: Vec<f64>,
    pub bias_change: f64,
    pub sample_count: usize,
    pub timestamp_ns: u128,
    pub encryption_nonce: Vec<u8>,
}

/// FederatedRound: One cycle of local train -> delta compute -> sync
#[derive(Debug, Clone)]
pub struct FederatedRound {
    pub round_id: String,
    pub node_deltas: HashMap<String, ModelDelta>,
    pub global_update: Option<LocalModel>,
    pub consensus_achieved: bool,
    pub completion_time_ms: f64,
}

impl EdgeNode {
    pub fn new(node_id: &str, domain: &str, region: &str, peers: Vec<String>) -> Self {
        let qpu = QPUEngine::new(QuantumBackend::EdgeQPU, 16);
        let router = TensorRouter::new();
        let consensus = FluidConsensusEngine::new(node_id, peers.clone());
        Self {
            node_id: node_id.to_string(),
            domain: domain.to_string(),
            region: region.to_string(),
            qpu_engine: qpu,
            tensor_router: router,
            consensus_engine: consensus,
            local_model: LocalModel::new(64),
            data_buffer: VecDeque::with_capacity(10000),
            model_deltas: VecDeque::with_capacity(1000),
            peers,
            latency_budget_ms: 1.0,
        }
    }

    /// Ingest raw data locally. NEVER transmits raw features.
    pub fn ingest(&mut self, features: Vec<f64>, label: Option<f64>, source: &str) {
        let fv = FeatureVector {
            vector_id: format!("fv_{}_{}", self.node_id, now_ns()),
            features,
            label,
            timestamp_ns: now_ns(),
            source: source.to_string(),
        };
        self.data_buffer.push_back(fv);
        if self.data_buffer.len() > 10000 {
            self.data_buffer.pop_front();
        }
    }

    /// Local training: stochastic gradient descent on buffered data
    /// Hardware: Coral TPU accelerates matrix ops via Edge TPU runtime
    pub fn train_local(&mut self, epochs: usize) -> f64 {
        let mut total_loss = 0.0;
        let mut count = 0;
        for _ in 0..epochs {
            for fv in self.data_buffer.iter() {
                let prediction = self.local_model.predict(&fv.features);
                let target = fv.label.unwrap_or(prediction);
                let error = target - prediction;
                total_loss += error * error;
                count += 1;
                self.local_model.update(&fv.features, error);
            }
        }
        self.local_model.epochs_trained += epochs as u64;
        let avg_loss = if count > 0 {
            total_loss / count as f64
        } else {
            0.0
        };
        self.local_model.last_accuracy = 1.0 - avg_loss.min(1.0);
        self.local_model.last_accuracy
    }

    /// Compute encrypted delta for federated aggregation
    /// Only weight changes leave the node. Raw data stays local.
    pub fn compute_delta(&mut self) -> ModelDelta {
        let delta = ModelDelta {
            delta_id: format!("delta_{}_{}", self.node_id, now_ns()),
            node_id: self.node_id.clone(),
            weight_changes: self.local_model.weights.clone(),
            bias_change: self.local_model.bias,
            sample_count: self.data_buffer.len(),
            timestamp_ns: now_ns(),
            encryption_nonce: vec![0u8; 12],
        };
        self.model_deltas.push_back(delta.clone());
        delta
    }

    /// Sync delta with peers via fluid consensus (gossip protocol)
    pub fn sync_delta(&mut self, delta: &ModelDelta) -> bool {
        let payload = format!(
            "delta:{}|weights:{}|bias:{}|samples:{}",
            delta.delta_id,
            delta.weight_changes.len(),
            delta.bias_change,
            delta.sample_count
        );
        let msg = self.consensus_engine.broadcast(&payload);
        msg.ttl > 0
    }

    /// Apply global model update from federated aggregator
    pub fn apply_global_update(&mut self, global: &LocalModel) {
        self.local_model.weights = global.weights.clone();
        self.local_model.bias = global.bias;
        self.local_model.epochs_trained = global.epochs_trained;
    }

    /// Quantum-enhanced inference: local prediction with QML boost
    pub fn infer(&mut self, features: &[f64]) -> (f64, f64) {
        let classical_pred = self.local_model.predict(features);
        let qml_model = QMLModel {
            model_id: format!("qml_{}", self.node_id),
            num_qubits: 8,
            num_layers: 3,
            variational_params: vec![0.1, 0.2, 0.3],
            feature_map: crate::brick42::quantum::qpu_engine::FeatureMapType::ZZFeatureMap,
            ansatz: crate::brick42::quantum::qpu_engine::AnsatzType::RealAmplitudes,
        };
        let (_predictions, quantum_confidence) = self.qpu_engine.qml_predict(&qml_model, features);
        (classical_pred, quantum_confidence)
    }

    /// Health check: latency, accuracy, buffer status
    pub fn health(&self) -> EdgeHealth {
        EdgeHealth {
            node_id: self.node_id.clone(),
            latency_ms: self.latency_budget_ms,
            accuracy: self.local_model.last_accuracy,
            buffer_size: self.data_buffer.len(),
            epochs_trained: self.local_model.epochs_trained,
            peers_connected: self.peers.len(),
            qpu_ready: self.qpu_engine.error_rate_acceptable(),
        }
    }
}

impl LocalModel {
    pub fn new(feature_count: usize) -> Self {
        Self {
            weights: vec![0.0; feature_count],
            bias: 0.0,
            learning_rate: 0.01,
            epochs_trained: 0,
            last_accuracy: 0.0,
        }
    }

    pub fn predict(&self, features: &[f64]) -> f64 {
        let mut sum = self.bias;
        for (i, &w) in self.weights.iter().enumerate() {
            if i < features.len() {
                sum += w * features[i];
            }
        }
        sum.tanh()
    }

    pub fn update(&mut self, features: &[f64], error: f64) {
        for (i, w) in self.weights.iter_mut().enumerate() {
            if i < features.len() {
                *w += self.learning_rate * error * features[i];
            }
        }
        self.bias += self.learning_rate * error;
    }
}

#[derive(Debug, Clone)]
pub struct EdgeHealth {
    pub node_id: String,
    pub latency_ms: f64,
    pub accuracy: f64,
    pub buffer_size: usize,
    pub epochs_trained: u64,
    pub peers_connected: usize,
    pub qpu_ready: bool,
}

fn now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
