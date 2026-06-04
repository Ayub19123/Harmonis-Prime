//! BRICK-51.1: Statistical Test Runner
//! Reproducible benchmarks with confidence bounds

use rand::rngs::StdRng;
use rand::SeedableRng;
use std::f64;

pub struct StatisticalRunner {
    seed: u64,
    rng: StdRng,
}

impl StatisticalRunner {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Run a test function n times, return mean ± std_dev
    pub fn benchmark<F>(&mut self, n: u64, mut f: F) -> (f64, f64, Vec<f64>)
    where
        F: FnMut(&mut StdRng) -> f64,
    {
        let mut results = Vec::with_capacity(n as usize);
        for _ in 0..n {
            let result = f(&mut self.rng);
            results.push(result);
        }

        let mean = results.iter().sum::<f64>() / results.len() as f64;
        let variance =
            results.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / results.len() as f64;
        let std_dev = variance.sqrt();

        (mean, std_dev, results)
    }

    /// Verify a probabilistic property with confidence
    pub fn verify_property<F>(&mut self, n: u64, threshold: f64, mut f: F) -> (f64, bool)
    where
        F: FnMut(&mut StdRng) -> bool,
    {
        let passes = (0..n).filter(|_| f(&mut self.rng)).count() as f64;
        let rate = passes / n as f64;
        (rate, rate >= threshold)
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }
}
