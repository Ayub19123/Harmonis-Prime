//! SET-7C: Thermodynamic Workload Balancing — Invariant Tests
//! 
//! Invariants:
//!   - entropy_zero_for_certain: H = 0 when p = 1 for one outcome
//!   - entropy_maximum_for_uniform: H = ln(N) for uniform distribution
//!   - kl_divergence_zero_for_identical: D_KL(P||P) = 0
//!   - kl_divergence_non_negative: D_KL(P||Q) >= 0
//!   - kl_divergence_undefined_error: Zero in Q where P non-zero → error
//!   - thermal_ambient_stability: Zero power → no temperature change
//!   - thermal_convergence: Model converges to T_ambient + P·R
//!   - thermal_higher_power_higher_temp: Monotonic power-temperature relationship
//!   - drift_detector_identical_no_drift: Same distributions → no drift
//!   - drift_detector_detects_shift: Different distributions → drift
//!   - determinism: Fixed inputs produce fixed outputs

use crate::thermodynamic_balance::*;

// --- Entropy Invariants ---

#[test]
fn test_entropy_zero_for_certain_distribution() {
    let engine = EntropyEngine::new();
    let p = vec![1.0, 0.0, 0.0];
    let h = engine.shannon_entropy(&p);
    assert!(h.abs() < 1e-12, "Entropy of certain distribution must be zero, got {}", h);
}

#[test]
fn test_entropy_maximum_for_uniform() {
    let engine = EntropyEngine::new();
    let n = 4;
    let p = vec![1.0 / n as f64; n];
    let h = engine.shannon_entropy(&p);
    let expected = (n as f64).ln();
    assert!((h - expected).abs() < 1e-12,
        "Uniform entropy should be ln({}) = {}, got {}", n, expected, h);
}

#[test]
fn test_entropy_empty_distribution() {
    let engine = EntropyEngine::new();
    let p: Vec<f64> = vec![];
    let h = engine.shannon_entropy(&p);
    assert_eq!(h, 0.0, "Empty distribution entropy must be zero");
}

#[test]
fn test_entropy_convention_zero_times_ln_zero() {
    let engine = EntropyEngine::new();
    let p = vec![0.0, 1.0];
    let h = engine.shannon_entropy(&p);
    assert!(h.abs() < 1e-12, "0·ln(0) convention must yield 0, got {}", h);
}

// --- KL Divergence Invariants ---

#[test]
fn test_kl_divergence_zero_for_identical() {
    let engine = EntropyEngine::new();
    let p = vec![0.25, 0.25, 0.25, 0.25];
    let kl = engine.kl_divergence(&p, &p).unwrap();
    assert!(kl.abs() < 1e-12, "D_KL(P||P) must be zero, got {}", kl);
}

#[test]
fn test_kl_divergence_non_negative() {
    let engine = EntropyEngine::new();
    let p = vec![0.5, 0.5];
    let q = vec![0.3, 0.7];
    let kl = engine.kl_divergence(&p, &q).unwrap();
    assert!(kl >= 0.0, "D_KL(P||Q) must be non-negative, got {}", kl);
}

#[test]
fn test_kl_divergence_length_mismatch_error() {
    let engine = EntropyEngine::new();
    let p = vec![0.5, 0.5];
    let q = vec![0.3, 0.4, 0.3];
    let result = engine.kl_divergence(&p, &q);
    assert!(result.is_err(), "Length mismatch must return error");
}

#[test]
fn test_kl_divergence_undefined_when_q_zero() {
    let engine = EntropyEngine::new();
    let p = vec![0.5, 0.5];
    let q = vec![0.0, 1.0];
    let result = engine.kl_divergence(&p, &q);
    assert!(result.is_err(), "Zero in Q where P non-zero must return error");
}

// --- Thermal Model Invariants ---

#[test]
fn test_thermal_ambient_stability() {
    let mut thermal = ThermalModel::new(300.0, 10.0, 1.0).unwrap();
    let temp_before = thermal.temperature();
    thermal.step(0.0, 1.0).unwrap();
    let temp_after = thermal.temperature();
    assert!((temp_before - temp_after).abs() < 1e-12,
        "Zero power must not change temperature: {} -> {}", temp_before, temp_after);
}

#[test]
fn test_thermal_convergence_to_steady_state() {
    let mut thermal = ThermalModel::new(300.0, 10.0, 0.1).unwrap();
    let power = 5.0;
    let expected_steady = thermal.steady_state(power);

    for _ in 0..1000 {
        thermal.step(power, 0.01).unwrap();
    }

    let final_temp = thermal.temperature();
    assert!((final_temp - expected_steady).abs() < 0.01,
        "Must converge to steady state {} (got {}), error {}",
        expected_steady, final_temp, (final_temp - expected_steady).abs());
}

#[test]
fn test_thermal_higher_power_higher_temp() {
    let mut thermal1 = ThermalModel::new(300.0, 10.0, 0.1).unwrap();
    let mut thermal2 = ThermalModel::new(300.0, 10.0, 0.1).unwrap();

    for _ in 0..100 {
        thermal1.step(5.0, 0.1).unwrap();
        thermal2.step(10.0, 0.1).unwrap();
    }

    assert!(thermal2.temperature() > thermal1.temperature(),
        "Higher power must produce higher temperature: {} vs {}",
        thermal2.temperature(), thermal1.temperature());
}

#[test]
fn test_thermal_negative_dt_error() {
    let mut thermal = ThermalModel::new(300.0, 10.0, 1.0).unwrap();
    let result = thermal.step(1.0, -1.0);
    assert!(result.is_err(), "Negative dt must return error");
}

#[test]
fn test_thermal_invalid_construction_error() {
    assert!(ThermalModel::new(0.0, 10.0, 1.0).is_err(), "Zero ambient must error");
    assert!(ThermalModel::new(300.0, 0.0, 1.0).is_err(), "Zero resistance must error");
    assert!(ThermalModel::new(300.0, 10.0, 0.0).is_err(), "Zero capacitance must error");
}

// --- Workload Drift Invariants ---

#[test]
fn test_drift_detector_identical_no_drift() {
    let detector = WorkloadDriftDetector::new(0.01).unwrap();
    let p = vec![0.25, 0.25, 0.25, 0.25];
    let drift = detector.detect_drift(&p, &p).unwrap();
    assert!(!drift, "Identical distributions must not show drift");
}

#[test]
fn test_drift_detector_detects_shift() {
    let detector = WorkloadDriftDetector::new(0.01).unwrap();
    let expected = vec![0.25, 0.25, 0.25, 0.25];
    let observed = vec![0.1, 0.1, 0.1, 0.7];
    let drift = detector.detect_drift(&expected, &observed).unwrap();
    assert!(drift, "Shifted distribution must show drift");
}

#[test]
fn test_drift_detector_threshold_respects_bound() {
    let detector = WorkloadDriftDetector::new(0.5).unwrap();
    let expected = vec![0.25, 0.25, 0.25, 0.25];
    let observed = vec![0.2, 0.2, 0.3, 0.3];
    let drift = detector.detect_drift(&expected, &observed).unwrap();
    assert!(!drift, "Small shift below threshold must not flag drift");
}

#[test]
fn test_drift_magnitude_finite() {
    let detector = WorkloadDriftDetector::new(0.01).unwrap();
    let expected = vec![0.25, 0.25, 0.25, 0.25];
    let observed = vec![0.1, 0.1, 0.1, 0.7];
    let mag = detector.drift_magnitude(&expected, &observed).unwrap();
    assert!(mag.is_finite(), "Drift magnitude must be finite");
    assert!(mag > 0.0, "Drift magnitude must be positive for different distributions");
}

#[test]
fn test_drift_detector_invalid_threshold_error() {
    assert!(WorkloadDriftDetector::new(-0.1).is_err(), "Negative threshold must error");
}

// --- Determinism Invariant ---

#[test]
fn test_thermodynamic_determinism() {
    let engine = EntropyEngine::new();
    let p = vec![0.2, 0.3, 0.5];
    let h1 = engine.shannon_entropy(&p);
    let h2 = engine.shannon_entropy(&p);
    assert_eq!(h1, h2, "Entropy must be deterministic for fixed inputs");
}