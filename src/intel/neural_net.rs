use crate::intel::tensor_core::{DType, Device, Shape, Tensor};
use serde::{Deserialize, Serialize};

/// Layer: A differentiable transformation — f: ℝⁿ → ℝᵐ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub weights: Tensor,
    pub bias: Tensor,
    pub activation: Activation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Activation {
    Linear,
    ReLU,
    Sigmoid,
    Tanh,
    Softmax,
}

/// NeuralNet: Composition of layers — f = fₙ ∘ fₙ₋₁ ∘ ... ∘ f₁
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralNet {
    pub layers: Vec<Layer>,
    pub input_dim: usize,
    pub output_dim: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Optimizer {
    SGD {
        learning_rate: f64,
    },
    Adam {
        learning_rate: f64,
        beta1: f64,
        beta2: f64,
        epsilon: f64,
    },
}

impl Layer {
    /// Create fully-connected layer — y = σ(W·x + b)
    pub fn new(
        input_dim: usize,
        output_dim: usize,
        activation: Activation,
        device: Device,
    ) -> Self {
        let mut weights = Tensor::rand(
            Shape::new(vec![output_dim, input_dim]),
            DType::Float64,
            device.clone(),
        );
        // Xavier initialization
        let scale = (6.0 / (input_dim + output_dim) as f64).sqrt();
        weights = weights.map(|x| (x - 0.5) * 2.0 * scale);

        let bias = Tensor::zeros(Shape::new(vec![output_dim]), DType::Float64, device.clone());

        Self {
            weights,
            bias,
            activation,
        }
    }

    /// Forward pass — y = σ(W·x + b)
    pub fn forward(&self, input: &Tensor) -> Result<Tensor, String> {
        let z = self.weights.matmul(input)?;
        let z_plus_b = z.add(&self.bias)?;

        match self.activation {
            Activation::Linear => Ok(z_plus_b),
            Activation::ReLU => Ok(z_plus_b.relu()),
            Activation::Sigmoid => Ok(z_plus_b.sigmoid()),
            Activation::Tanh => Ok(z_plus_b.map(|x| x.tanh())),
            Activation::Softmax => Ok(z_plus_b.softmax()),
        }
    }
}

impl NeuralNet {
    /// Create network from layer dimensions
    pub fn new(
        layer_dims: Vec<usize>,
        activations: Vec<Activation>,
        device: Device,
    ) -> Result<Self, String> {
        if layer_dims.len() < 2 {
            return Err("NeuralNet: need at least input and output layers".to_string());
        }
        if activations.len() != layer_dims.len() - 1 {
            return Err("NeuralNet: activation count must be layer count - 1".to_string());
        }

        let mut layers = Vec::new();
        for i in 0..layer_dims.len() - 1 {
            layers.push(Layer::new(
                layer_dims[i],
                layer_dims[i + 1],
                activations[i].clone(),
                device.clone(),
            ));
        }

        Ok(Self {
            input_dim: layer_dims[0],
            output_dim: layer_dims[layer_dims.len() - 1],
            layers,
        })
    }

    /// Forward propagation through all layers
    pub fn forward(&self, input: &Tensor) -> Result<Tensor, String> {
        let mut current = input.clone();
        for (i, layer) in self.layers.iter().enumerate() {
            current = layer
                .forward(&current)
                .map_err(|e| format!("Layer {}: {}", i, e))?;
        }
        Ok(current)
    }

    /// Predict: single forward pass with validation
    pub fn predict(&self, input: &Tensor) -> Result<Tensor, String> {
        if input.shape.dims[0] != self.input_dim {
            return Err(format!(
                "Predict: expected input dim {}, got {}",
                self.input_dim, input.shape.dims[0]
            ));
        }
        self.forward(input)
    }

    /// Layer count
    pub fn depth(&self) -> usize {
        self.layers.len()
    }
}
