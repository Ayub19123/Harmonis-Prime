use crate::intel::neural_net::{Activation, NeuralNet};
use crate::intel::tensor_core::{DType, Device, Shape, Tensor};
use serde::{Deserialize, Serialize};

/// Policy: π(a|s) — probability distribution over actions given state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub network: NeuralNet,
    pub state_dim: usize,
    pub action_dim: usize,
}

/// Reward: R(s, a) → ℝ — scalar feedback signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reward {
    pub value: f64,
    pub timestamp: u64,
    pub context: String,
}

/// Environment: The world the agent interacts with
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub state_dim: usize,
    pub action_dim: usize,
    pub current_state: Vec<f64>,
    pub step_count: u64,
}

/// RLAgent: Learns optimal policy through interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLAgent {
    pub policy: Policy,
    pub gamma: f64,
    pub learning_rate: f64,
    pub experience_buffer: Vec<(Vec<f64>, usize, f64, Vec<f64>)>,
    pub buffer_size: usize,
}

/// Simple deterministic pseudo-random — no external crate
fn random_f64() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut state = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    (state as f64) / (u64::MAX as f64)
}

impl Policy {
    /// Create policy network — π(a|s) = softmax(W·s + b)
    pub fn new(
        state_dim: usize,
        action_dim: usize,
        hidden_dims: Vec<usize>,
        device: Device,
    ) -> Result<Self, String> {
        let mut layer_dims = vec![state_dim];
        layer_dims.extend(hidden_dims);
        layer_dims.push(action_dim);

        let mut activations = vec![Activation::ReLU; layer_dims.len() - 2];
        activations.push(Activation::Softmax);

        let network = NeuralNet::new(layer_dims, activations, device)?;

        Ok(Self {
            network,
            state_dim,
            action_dim,
        })
    }

    /// Sample action from policy — a ~ π(·|s)
    pub fn sample_action(&self, state: &[f64]) -> Result<usize, String> {
        let state_tensor = Tensor::new(
            state.to_vec(),
            Shape::new(vec![state.len()]),
            DType::Float64,
            Device::Cpu,
        )?;

        let probs = self.network.forward(&state_tensor)?;
        let mut cumsum = 0.0;
        let threshold = random_f64();
        for (i, &p) in probs.data.iter().enumerate() {
            cumsum += p;
            if cumsum >= threshold {
                return Ok(i);
            }
        }
        Ok(probs.data.len() - 1)
    }

    /// Get action probabilities — π(a|s) for all a
    pub fn get_probs(&self, state: &[f64]) -> Result<Vec<f64>, String> {
        let state_tensor = Tensor::new(
            state.to_vec(),
            Shape::new(vec![state.len()]),
            DType::Float64,
            Device::Cpu,
        )?;

        let probs = self.network.forward(&state_tensor)?;
        Ok(probs.data.clone())
    }
}

impl Environment {
    /// Create environment
    pub fn new(state_dim: usize, action_dim: usize) -> Self {
        Self {
            state_dim,
            action_dim,
            current_state: vec![0.0; state_dim],
            step_count: 0,
        }
    }

    /// Reset environment — s₀ ~ p(s₀)
    pub fn reset(&mut self) {
        self.current_state = vec![0.0; self.state_dim];
        self.step_count = 0;
    }

    /// Step — s', r = env.step(s, a)
    pub fn step(&mut self, action: usize) -> (Vec<f64>, Reward) {
        self.step_count += 1;
        for i in 0..self.state_dim {
            self.current_state[i] += (action as f64) * 0.1;
            self.current_state[i] = self.current_state[i].clamp(-10.0, 10.0);
        }

        let reward_value = -self.current_state.iter().map(|&x| x * x).sum::<f64>();
        let reward = Reward {
            value: reward_value,
            timestamp: self.step_count,
            context: format!("step_{}", self.step_count),
        };

        (self.current_state.clone(), reward)
    }
}

impl RLAgent {
    /// Create RL agent
    pub fn new(
        state_dim: usize,
        action_dim: usize,
        hidden_dims: Vec<usize>,
        gamma: f64,
        learning_rate: f64,
        buffer_size: usize,
        device: Device,
    ) -> Result<Self, String> {
        let policy = Policy::new(state_dim, action_dim, hidden_dims, device)?;

        Ok(Self {
            policy,
            gamma,
            learning_rate,
            experience_buffer: Vec::new(),
            buffer_size,
        })
    }

    /// Store experience — (s, a, r, s')
    pub fn store_experience(
        &mut self,
        state: Vec<f64>,
        action: usize,
        reward: f64,
        next_state: Vec<f64>,
    ) {
        self.experience_buffer
            .push((state, action, reward, next_state));
        if self.experience_buffer.len() > self.buffer_size {
            self.experience_buffer.remove(0);
        }
    }

    /// Compute discounted returns — Gₜ = Σₖ γᵏ Rₜ₊ₖ
    pub fn compute_returns(&self, rewards: &[f64]) -> Vec<f64> {
        let mut returns = vec![0.0; rewards.len()];
        let mut running_return = 0.0;
        for t in (0..rewards.len()).rev() {
            running_return = rewards[t] + self.gamma * running_return;
            returns[t] = running_return;
        }
        returns
    }

    /// Get policy reference
    pub fn get_policy(&self) -> &Policy {
        &self.policy
    }

    /// Buffer size
    pub fn buffer_len(&self) -> usize {
        self.experience_buffer.len()
    }
}
