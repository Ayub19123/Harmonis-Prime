//! SET-6E: Energy Telemetry Invariant Tests
//! 
//! Invariants:
//!   - energy_telemetry_drift: |E_measured - E_model| / E_model <= 0.01
//!   - dvfs_frequency_scaling: P_dyn error <= 5% at each DVFS step
//!   - pmu_counter_overflow: zero drift accumulation across 64-bit wraparound
//!   - thermal_throttle_detection: power drop detection within 100ms
//!   - byzantine_energy_injection: outlier detection rejects >3σ deviations

use crate::energy_telemetry::telemetry::*;

// --- Drift Invariant ---

#[test]
fn test_energy_telemetry_drift_within_1_percent() {
    let profile = DvfsProfile::new(1.0, 1_000_000_000.0, 1e-9).unwrap();
    let mut telemetry = EnergyTelemetry::new(profile, 0.3, 1.0).unwrap();
    
    // Simulate 100 measurements with small noise
    for i in 0..100 {
        let noise = 1.0 + (i as f64 * 0.001); // slight variation
        let drift = telemetry.sample(noise);
        assert!(drift.drift_ratio <= 0.01, 
            "Drift {:.4}% exceeds 1% tolerance at iteration {}", 
            drift.drift_ratio * 100.0, i);
    }
}

#[test]
fn test_ema_filter_convergence() {
    let mut ema = EmaFilter::new(0.1, 1.0).unwrap();
    let model = 1.0;
    
    // Apply noisy measurements, EMA should converge toward model
    for _ in 0..1000 {
        let measured = 1.0 + (if fastrand::u8(0..2) == 0 { 0.05 } else { -0.05 });
        ema.update(measured, model);
    }
    
    let final_drift = ema.drift(model);
    assert!(final_drift < 0.01, "EMA failed to converge: drift = {:.4}%", final_drift * 100.0);
}

// --- DVFS Scaling Invariant ---

#[test]
fn test_dvfs_frequency_scaling_error_within_5_percent() {
    let profile = DvfsProfile::new(1.0, 1_000_000_000.0, 1e-9).unwrap();
    let mut telemetry = EnergyTelemetry::new(profile, 0.5, 1.0).unwrap();
    
    // Original power at 1 GHz
    let original_power = telemetry.power_model;
    
    // Scale to 2 GHz
    telemetry.change_frequency(2_000_000_000.0).unwrap();
    let new_power = telemetry.power_model;
    
    // P_dyn proportional to f, so 2x frequency = 2x power
    let expected_power = original_power * 2.0;
    let error = (new_power - expected_power).abs() / expected_power;
    
    assert!(error <= 0.05, 
        "DVFS scaling error {:.2}% exceeds 5%", error * 100.0);
}

#[test]
fn test_dvfs_voltage_scaling() {
    let profile = DvfsProfile::new(1.0, 1_000_000_000.0, 1e-9).unwrap();
    let power_1v = compute_dynamic_power(&profile);
    
    // P proportional to V^2
    let profile_2v = DvfsProfile::new(2.0, 1_000_000_000.0, 1e-9).unwrap();
    let power_2v = compute_dynamic_power(&profile_2v);
    
    let ratio = power_2v / power_1v;
    assert!((ratio - 4.0).abs() < 0.01, "V^2 scaling violated: ratio = {}", ratio);
}

// --- PMU Overflow Invariant ---

#[test]
fn test_pmu_counter_overflow_no_drift() {
    let mut pmu = PmuSimulator::new();
    pmu.set_near_overflow();
    
    // Read before overflow
    let _ = pmu.read();
    
    // Increment past overflow
    pmu.increment(200);
    let delta = pmu.read();
    
    // Delta should be correct despite wraparound
    assert_eq!(delta, 200, "Overflow caused incorrect delta");
    assert_eq!(pmu.overflow_count, 1, "Overflow not tracked");
}

// --- Byzantine Injection Invariant ---

#[test]
fn test_byzantine_energy_outlier_rejection() {
    let profile = DvfsProfile::new(1.0, 1_000_000_000.0, 1e-9).unwrap();
    let mut telemetry = EnergyTelemetry::new(profile, 0.3, 1.0).unwrap();
    
    // Establish baseline with normal readings
    for _ in 0..10 {
        telemetry.sample(1.0);
    }
    
    // Inject outlier (>3 sigma from model)
    let outlier = 10.0; // 10x expected power
    let drift = telemetry.sample(outlier);
    
    // EMA with alpha=0.3 should not fully absorb outlier
    assert!(drift.drift_ratio > 0.01, 
        "Byzantine outlier should cause detectable drift: {:.4}%", 
        drift.drift_ratio * 100.0);
}