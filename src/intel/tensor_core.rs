use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tensor {
    pub data: Vec<f64>,
    pub shape: Shape,
    pub dtype: DType,
    pub device: Device,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shape {
    pub dims: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DType {
    Float32,
    Float64,
    Int32,
    Int64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Device {
    Cpu,
    Gpu(usize),
}

impl Shape {
    pub fn new(dims: Vec<usize>) -> Self {
        Self { dims }
    }

    pub fn numel(&self) -> usize {
        self.dims.iter().product()
    }

    pub fn ndim(&self) -> usize {
        self.dims.len()
    }

    pub fn strides(&self) -> Vec<usize> {
        let mut strides = vec![1usize; self.dims.len()];
        for i in (0..self.dims.len().saturating_sub(1)).rev() {
            strides[i] = strides[i + 1] * self.dims[i + 1];
        }
        strides
    }

    pub fn reshape(&self, new_dims: Vec<usize>) -> Result<Shape, String> {
        if new_dims.iter().product::<usize>() != self.numel() {
            return Err("Reshape: element count mismatch".to_string());
        }
        Ok(Shape::new(new_dims))
    }
}

impl Tensor {
    pub fn new(data: Vec<f64>, shape: Shape, dtype: DType, device: Device) -> Result<Self, String> {
        if data.len() != shape.numel() {
            return Err("Tensor: data length does not match shape".to_string());
        }
        Ok(Self {
            data,
            shape,
            dtype,
            device,
        })
    }

    pub fn zeros(shape: Shape, dtype: DType, device: Device) -> Self {
        let numel = shape.numel();
        Self {
            data: vec![0.0; numel],
            shape,
            dtype,
            device,
        }
    }

    pub fn ones(shape: Shape, dtype: DType, device: Device) -> Self {
        let numel = shape.numel();
        Self {
            data: vec![1.0; numel],
            shape,
            dtype,
            device,
        }
    }

    pub fn rand(shape: Shape, dtype: DType, device: Device) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let mut data = Vec::with_capacity(shape.numel());
        let mut state = seed;
        for _ in 0..shape.numel() {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let val = (state as f64) / (u64::MAX as f64);
            data.push(val);
        }
        Self {
            data,
            shape,
            dtype,
            device,
        }
    }

    pub fn add(&self, other: &Tensor) -> Result<Tensor, String> {
        if self.shape.dims != other.shape.dims {
            return Err("Add: shape mismatch".to_string());
        }
        let data: Vec<f64> = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a + b)
            .collect();
        Ok(Tensor {
            data,
            shape: self.shape.clone(),
            dtype: self.dtype.clone(),
            device: self.device.clone(),
        })
    }

    pub fn mul(&self, other: &Tensor) -> Result<Tensor, String> {
        if self.shape.dims != other.shape.dims {
            return Err("Mul: shape mismatch".to_string());
        }
        let data: Vec<f64> = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a * b)
            .collect();
        Ok(Tensor {
            data,
            shape: self.shape.clone(),
            dtype: self.dtype.clone(),
            device: self.device.clone(),
        })
    }

    pub fn matmul(&self, other: &Tensor) -> Result<Tensor, String> {
        if self.shape.ndim() != 2 || other.shape.ndim() != 2 {
            return Err("Matmul: requires 2D tensors".to_string());
        }
        let (m, k1) = (self.shape.dims[0], self.shape.dims[1]);
        let (k2, n) = (other.shape.dims[0], other.shape.dims[1]);
        if k1 != k2 {
            return Err("Matmul: inner dimensions mismatch".to_string());
        }

        let mut result = vec![0.0; m * n];
        for i in 0..m {
            for j in 0..n {
                let mut sum = 0.0;
                for k in 0..k1 {
                    sum += self.data[i * k1 + k] * other.data[k * n + j];
                }
                result[i * n + j] = sum;
            }
        }

        Ok(Tensor {
            data: result,
            shape: Shape::new(vec![m, n]),
            dtype: self.dtype.clone(),
            device: self.device.clone(),
        })
    }

    pub fn transpose(&self) -> Result<Tensor, String> {
        if self.shape.ndim() != 2 {
            return Err("Transpose: requires 2D tensor".to_string());
        }
        let (m, n) = (self.shape.dims[0], self.shape.dims[1]);
        let mut data = vec![0.0; m * n];
        for i in 0..m {
            for j in 0..n {
                data[j * m + i] = self.data[i * n + j];
            }
        }
        Ok(Tensor {
            data,
            shape: Shape::new(vec![n, m]),
            dtype: self.dtype.clone(),
            device: self.device.clone(),
        })
    }

    pub fn flatten(&self) -> Tensor {
        Tensor {
            data: self.data.clone(),
            shape: Shape::new(vec![self.shape.numel()]),
            dtype: self.dtype.clone(),
            device: self.device.clone(),
        }
    }

    pub fn get(&self, indices: &[usize]) -> Option<f64> {
        if indices.len() != self.shape.ndim() {
            return None;
        }
        let strides = self.shape.strides();
        let mut idx = 0;
        for (i, &dim_idx) in indices.iter().enumerate() {
            if dim_idx >= self.shape.dims[i] {
                return None;
            }
            idx += dim_idx * strides[i];
        }
        self.data.get(idx).copied()
    }

    pub fn map<F>(&self, f: F) -> Tensor
    where
        F: Fn(f64) -> f64,
    {
        Tensor {
            data: self.data.iter().map(|&x| f(x)).collect(),
            shape: self.shape.clone(),
            dtype: self.dtype.clone(),
            device: self.device.clone(),
        }
    }

    pub fn relu(&self) -> Tensor {
        self.map(|x| x.max(0.0))
    }

    pub fn sigmoid(&self) -> Tensor {
        self.map(|x| 1.0 / (1.0 + (-x).exp()))
    }

    pub fn softmax(&self) -> Tensor {
        let max_val = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let exp_data: Vec<f64> = self.data.iter().map(|&x| (x - max_val).exp()).collect();
        let sum: f64 = exp_data.iter().sum();
        Tensor {
            data: exp_data.iter().map(|&x| x / sum).collect(),
            shape: self.shape.clone(),
            dtype: self.dtype.clone(),
            device: self.device.clone(),
        }
    }

    pub fn sum(&self) -> f64 {
        self.data.iter().sum()
    }

    pub fn mean(&self) -> f64 {
        self.sum() / self.data.len() as f64
    }
}
