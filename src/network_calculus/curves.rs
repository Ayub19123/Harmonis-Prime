//! SET-6D: Min-Plus Network Calculus - Deterministic Latency Bounds
//!
//! Mathematical model:
//!   Arrival curve alpha(t) = rate * t + burst  (affine, sub-additive)
//!   Service curve beta(t)  = rate * t          (strict service)
//!   Delay bound W = horizontal deviation
//!
//! For strict service curves: W = burst / service_rate

use std::f64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrivalCurve {
    pub rate: f64,
    pub burst: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServiceCurve {
    pub rate: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelayBound {
    pub bound_ns: f64,
    pub arrival_rate: f64,
    pub service_rate: f64,
    pub burst: f64,
    pub stable: bool,
}

impl ArrivalCurve {
    pub fn new(rate: f64, burst: f64) -> Result<Self, &'static str> {
        if rate < 0.0 || burst < 0.0 {
            return Err("Arrival curve parameters must be non-negative");
        }
        if !rate.is_finite() || !burst.is_finite() {
            return Err("Arrival curve parameters must be finite");
        }
        Ok(Self { rate, burst })
    }

    pub fn evaluate(&self, t: f64) -> f64 {
        self.rate * t + self.burst
    }

    pub fn check_subadditive(&self, t: f64, s: f64) -> bool {
        let lhs = self.evaluate(t + s);
        let rhs = self.evaluate(t) + self.evaluate(s) - self.burst;
        lhs <= rhs + 1e-9
    }
}

impl ServiceCurve {
    pub fn new(rate: f64) -> Result<Self, &'static str> {
        if rate < 0.0 {
            return Err("Service rate must be non-negative");
        }
        if !rate.is_finite() {
            return Err("Service rate must be finite");
        }
        Ok(Self { rate })
    }

    pub fn evaluate(&self, t: f64) -> f64 {
        if t <= 0.0 {
            0.0
        } else {
            self.rate * t
        }
    }
}

pub fn compute_delay_bound(
    arrival: &ArrivalCurve,
    service: &ServiceCurve,
) -> Result<DelayBound, &'static str> {
    if service.rate <= arrival.rate {
        return Err("Service rate must exceed arrival rate for stability");
    }

    let rate_diff = service.rate - arrival.rate;
    let bound = arrival.burst / rate_diff;
    let bound_ns = bound * 1_000_000_000.0;

    Ok(DelayBound {
        bound_ns,
        arrival_rate: arrival.rate,
        service_rate: service.rate,
        burst: arrival.burst,
        stable: true,
    })
}

pub fn token_bucket_arrival(rate: f64, depth: f64) -> Result<ArrivalCurve, &'static str> {
    ArrivalCurve::new(rate, depth)
}

pub fn leaky_bucket_regulator(input: f64, max_burst: f64, current_bucket: f64) -> (f64, f64) {
    let new_bucket = (current_bucket + input).min(max_burst);
    let output = if new_bucket <= max_burst { input } else { 0.0 };
    (output, new_bucket)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrival_curve_basic() {
        let alpha = ArrivalCurve::new(1000.0, 500.0).unwrap();
        assert_eq!(alpha.evaluate(0.0), 500.0);
        assert_eq!(alpha.evaluate(1.0), 1500.0);
    }

    #[test]
    fn test_service_curve_basic() {
        let beta = ServiceCurve::new(2000.0).unwrap();
        assert_eq!(beta.evaluate(0.0), 0.0);
        assert_eq!(beta.evaluate(1.0), 2000.0);
    }

    #[test]
    fn test_delay_bound_stable() {
        let alpha = ArrivalCurve::new(1_000_000_000.0, 1000.0).unwrap();
        let beta = ServiceCurve::new(10_000_000_000.0).unwrap();
        let bound = compute_delay_bound(&alpha, &beta).unwrap();
        assert!(bound.bound_ns < 1000.0);
        assert!(bound.stable);
    }

    #[test]
    fn test_delay_bound_unstable() {
        let alpha = ArrivalCurve::new(10_000_000_000.0, 1000.0).unwrap();
        let beta = ServiceCurve::new(1_000_000_000.0).unwrap();
        assert!(compute_delay_bound(&alpha, &beta).is_err());
    }

    #[test]
    fn test_subadditivity() {
        let alpha = ArrivalCurve::new(1000.0, 500.0).unwrap();
        assert!(alpha.check_subadditive(1.0, 2.0));
    }

    #[test]
    fn test_token_bucket() {
        let tb = token_bucket_arrival(1_000_000.0, 500.0).unwrap();
        assert_eq!(tb.rate, 1_000_000.0);
    }

    #[test]
    fn test_leaky_bucket() {
        let (output, bucket) = leaky_bucket_regulator(100.0, 500.0, 400.0);
        assert_eq!(output, 100.0);
        assert_eq!(bucket, 500.0);
    }
}
