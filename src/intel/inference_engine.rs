use crate::intel::neural_net::NeuralNet;
use crate::intel::rl_agent::RLAgent;
use crate::intel::tensor_core::{DType, Device, Shape, Tensor};
use serde::{Deserialize, Serialize};

/// Prediction: A single inference result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub model_id: String,
    pub input_id: String,
    pub outputs: Vec<f64>,
    pub confidence: f64,
    pub latency_ms: f64,
    pub timestamp: u64,
}

/// BatchRequest: Multiple inputs for batch inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    pub request_id: String,
    pub inputs: Vec<Vec<f64>>,
    pub model_id: String,
}

/// InferenceEngine: Serves predictions with sub-millisecond latency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceEngine {
    pub models: Vec<NeuralNet>,
    pub model_map: std::collections::HashMap<String, usize>,
    pub total_requests: u64,
    pub total_latency_ms: f64,
    pub device: Device,
}

impl Prediction {
    /// Create prediction
    pub fn new(model_id: String, input_id: String, outputs: Vec<f64>, latency_ms: f64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let confidence = outputs.iter().map(|&x| x * x).sum::<f64>().sqrt();

        Self {
            model_id,
            input_id,
            outputs,
            confidence,
            latency_ms,
            timestamp: now,
        }
    }
}

impl InferenceEngine {
    /// Create inference engine
    pub fn new(device: Device) -> Self {
        Self {
            models: Vec::new(),
            model_map: std::collections::HashMap::new(),
            total_requests: 0,
            total_latency_ms: 0.0,
            device,
        }
    }

    /// Register a model
    pub fn register_model(&mut self, model_id: String, model: NeuralNet) {
        let idx = self.models.len();
        self.models.push(model);
        self.model_map.insert(model_id, idx);
    }

    /// Single prediction — inference with timing
    pub fn predict(&self, model_id: &str, input: &Vec<f64>) -> Result<Prediction, String> {
        let start = std::time::Instant::now();

        let model_idx = self
            .model_map
            .get(model_id)
            .ok_or_else(|| format!("Model {} not found", model_id))?;
        let model = &self.models[*model_idx];

        let input_tensor = Tensor::new(
            input.clone(),
            Shape::new(vec![input.len()]),
            DType::Float64,
            self.device.clone(),
        )?;

        let output = model.predict(&input_tensor)?;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(Prediction::new(
            model_id.to_string(),
            format!("req_{}", self.total_requests),
            output.data,
            latency_ms,
        ))
    }

    /// Batch prediction — process multiple inputs
    pub fn predict_batch(
        &self,
        model_id: &str,
        inputs: &[Vec<f64>],
    ) -> Result<Vec<Prediction>, String> {
        let mut predictions = Vec::new();
        for (i, input) in inputs.iter().enumerate() {
            let pred = self
                .predict(model_id, input)
                .map_err(|e| format!("Batch item {}: {}", i, e))?;
            predictions.push(pred);
        }
        Ok(predictions)
    }

    /// RL policy inference — select action from state
    pub fn policy_action(&self, agent: &RLAgent, state: &[f64]) -> Result<usize, String> {
        agent.get_policy().sample_action(state)
    }

    /// Get average latency
    pub fn avg_latency_ms(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.total_latency_ms / self.total_requests as f64
        }
    }

    /// Model count
    pub fn model_count(&self) -> usize {
        self.models.len()
    }
}
