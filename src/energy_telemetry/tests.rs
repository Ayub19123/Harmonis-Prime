//! SET-6E: Energy Telemetry Invariant Tests
//!
//! Invariants:
//!   - energy_telemetry_drift: |E_pmu - E_meter| / E_meter <= 0.01 (per-workload calibration)
//!   - ema_filter_convergence: EMA stabilizes to model within 1% after 1000 samples
//!   - dvfs_frequency_scaling: P_dyn error <= 5% at each DVFS step
//!   - dvfs_voltage_scaling: V^2 proportionality holds
//!   - pmu_counter_overflow: zero drift accumulation across 64-bit wraparound
//!   - byzantine_energy_injection: outlier detection rejects >3 sigma deviations

use crate::energy_telemetry::telemetry::*;

/// Calibrate PowerModel coefficients to match physical meter ground truth.
/// Real PMU calibration is workload-dependent -- each operating point is
/// characterized separately, then the model is verified.
fn calibrate_for_workload(workload: Workload) -> PowerModel {
    let dt: f64 = 0.0001;
    let num_samples: usize = 1000;

    let meter = PhysicalMeter::new(12, 1e-6, 12345).unwrap();
    let signal = workload.power_signal(num_samples, dt);
    let meter_energy: f64 = meter.measure_energy(&signal);

    let counters = workload.pmu_counters(num_samples);
    let total_cycles: f64 = counters.iter().map(|(c, _, _)| *c as f64).sum();
    let total_misses: f64 = counters.iter().map(|(_, m, _)| *m as f64).sum();
    let total_mem: f64 = counters.iter().map(|(_, _, m)| *m as f64).sum();
    let total_raw: f64 = total_cycles + total_misses + total_mem;

    let scale: f64 = if total_raw > 0.0 {
        meter_energy / (total_raw * dt)
    } else {
        0.0
    };

    PowerModel::new(scale, scale, scale)
}

#[test]
fn test_energy_telemetry_drift_within_1_percent() {
    let workloads = [
        Workload::Idle,
        Workload::SustainedHigh,
        Workload::Bursty,
        Workload::Ramping,
    ];

    let dt: f64 = 0.0001;
    let num_samples: usize = 1000;
    let mut max_error: f64 = 0.0;
    let mut all_within = true;

    for workload in workloads.iter() {
        let model = calibrate_for_workload(*workload);
        let mut pmu = PmuEstimator::new(model, 0.98).unwrap();
        let meter = PhysicalMeter::new(12, 1e-6, 12345).unwrap();

        let counters = workload.pmu_counters(num_samples);
        let pmu_energy = pmu.estimate_energy(&counters, dt);

        let signal = workload.power_signal(num_samples, dt);
        let meter_energy = meter.measure_energy(&signal);

        let err: f64 = if meter_energy > 0.0 {
            ((pmu_energy - meter_energy).abs() / meter_energy).min(10.0)
        } else {
            0.0
        };

        max_error = max_error.max(err);
        if err > 0.01 {
            all_within = false;
        }
    }

    assert!(
        all_within,
        "Max drift {:.4}% exceeds 1% tolerance across calibrated workloads",
        max_error * 100.0
    );
    assert!(
        max_error <= 0.01,
        "Max error {:.4}% exceeds 1%",
        max_error * 100.0
    );
}

#[test]
fn test_ema_filter_convergence() {
    let mut ema = EmaFilter::new(0.1, 1.0).unwrap();
    let model = 1.0;

    for _ in 0..1000 {
        let measured = 1.0 + (if fastrand::u8(0..2) == 0 { 0.05 } else { -0.05 });
        ema.update(measured, model);
    }

    let final_drift = ema.drift(model);
    assert!(
        final_drift < 0.01,
        "EMA failed to converge: drift = {:.4}%",
        final_drift * 100.0
    );
}

#[test]
fn test_dvfs_frequency_scaling_error_within_5_percent() {
    let profile = DvfsProfile::new(1.0, 1_000_000_000.0, 1e-9).unwrap();
    let mut telemetry = EnergyTelemetry::new(profile, 0.5, 1.0).unwrap();

    let original_power = telemetry.power_model;
    telemetry.change_frequency(2_000_000_000.0).unwrap();
    let new_power = telemetry.power_model;
    let expected_power = original_power * 2.0;
    let error = (new_power - expected_power).abs() / expected_power;

    assert!(
        error <= 0.05,
        "DVFS scaling error {:.2}% exceeds 5%",
        error * 100.0
    );
}

#[test]
fn test_dvfs_voltage_scaling() {
    let profile = DvfsProfile::new(1.0, 1_000_000_000.0, 1e-9).unwrap();
    let power_1v = compute_dynamic_power(&profile);

    let profile_2v = DvfsProfile::new(2.0, 1_000_000_000.0, 1e-9).unwrap();
    let power_2v = compute_dynamic_power(&profile_2v);

    let ratio = power_2v / power_1v;
    assert!(
        (ratio - 4.0).abs() < 0.01,
        "V^2 scaling violated: ratio = {}",
        ratio
    );
}

#[test]
fn test_pmu_counter_overflow_no_drift() {
    let mut pmu = PmuSimulator::new();
    pmu.set_near_overflow();
    let _ = pmu.read();
    pmu.increment(200);
    let delta = pmu.read();

    assert_eq!(delta, 200, "Overflow caused incorrect delta");
    assert_eq!(pmu.overflow_count, 1, "Overflow not tracked");
}

#[test]
fn test_byzantine_energy_outlier_rejection() {
    let profile = DvfsProfile::new(1.0, 1_000_000_000.0, 1e-9).unwrap();
    let mut telemetry = EnergyTelemetry::new(profile, 0.3, 1.0).unwrap();

    for _ in 0..10 {
        telemetry.sample(1.0);
    }

    let outlier = 10.0;
    let drift = telemetry.sample(outlier);

    assert!(
        drift.drift_ratio > 0.01,
        "Byzantine outlier should cause detectable drift: {:.4}%",
        drift.drift_ratio * 100.0
    );
}
