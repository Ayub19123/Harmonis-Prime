//! SET-10: Fusion Layer -- 11 Invariant Tests
//!
//! Invariants:
//!   - theta_approx: 3 tests (monotonicity, positive derivative, finite evaluation)
//!   - extended_series: 2 tests (convergence sigma>1, error bound decreases with N)
//!   - thermal_bridge: 3 tests (energy rises with temp, finite output, positive base)
//!   - entropy_placement: 3 tests (lowest energy selected, uninitialized error, entropy threshold)

use crate::set10_fusion::*;

// ============================================================
// THETA APPROXIMATION -- 3 tests
// ============================================================

#[test]
fn test_theta_monotonicity() {
    let theta = ThetaApproximation::new();
    let t1 = 100.0;
    let t2 = 200.0;
    assert!(
        theta.is_monotonic(t1, t2).unwrap(),
        "Theta(t) must be monotonically increasing"
    );
}

#[test]
fn test_theta_derivative_positive() {
    let theta = ThetaApproximation::new();
    let t = 100.0;
    let d = theta.derivative(t).unwrap();
    assert!(
        d > 0.0,
        "dtheta/dt must be positive for t > 1/(2*pi), got {}",
        d
    );
}

#[test]
fn test_theta_finite_evaluation() {
    let theta = ThetaApproximation::new();
    let t = 1000.0;
    let val = theta.evaluate(t).unwrap();
    assert!(val.is_finite(), "theta(t) must be finite for finite t");
    assert!(val > 0.0, "theta(t) must be positive for large t");
}

// ============================================================
// EXTENDED DIRICHLET SERIES -- 2 tests
// ============================================================

#[test]
fn test_extended_series_converges_sigma_gt_one() {
    let series = ExtendedDirichletSeries::new(100).unwrap();
    let (real, imag) = series.evaluate(2.0, 0.0).unwrap();
    assert!(
        real.is_finite() && imag.is_finite(),
        "Series must converge for sigma > 1"
    );
    // zeta(2) = pi^2/6 ~ 1.6449
    assert!(
        (real - std::f64::consts::PI.powi(2) / 6.0).abs() < 0.02,
        "Truncated zeta(2) should approximate pi^2/6"
    );
}

#[test]
fn test_extended_series_rejects_sigma_le_one() {
    let series = ExtendedDirichletSeries::new(100).unwrap();
    let result = series.evaluate(1.0, 0.0);
    assert!(
        result.is_err(),
        "Sigma = 1 must return error -- honest divergence"
    );
}

// ============================================================
// THERMAL BRIDGE -- 3 tests
// ============================================================

#[test]
fn test_thermal_bridge_energy_rises_with_temperature() {
    let params = crate::set9_telemetry::thermal_rc::ThermalParams::new(300.0, 10.0, 0.1).unwrap();
    let mut bridge1 = PimThermalBridge::new(params, 1.0e-6, 1.0e-8).unwrap();
    let mut bridge2 = PimThermalBridge::new(params, 1.0e-6, 1.0e-8).unwrap();

    // Run bridge1 with higher power
    for _ in 0..100 {
        let _ = bridge1.step_and_estimate(10.0, 0.1).unwrap();
    }
    // Run bridge2 with lower power
    for _ in 0..100 {
        let _ = bridge2.step_and_estimate(5.0, 0.1).unwrap();
    }

    let energy1 = bridge1.step_and_estimate(0.0, 0.01).unwrap();
    let energy2 = bridge2.step_and_estimate(0.0, 0.01).unwrap();

    assert!(
        energy1 > energy2,
        "Higher thermal history must produce higher energy estimate"
    );
}

#[test]
fn test_thermal_bridge_finite_output() {
    let params = crate::set9_telemetry::thermal_rc::ThermalParams::new(300.0, 10.0, 1.0).unwrap();
    let mut bridge = PimThermalBridge::new(params, 1.0e-6, 1.0e-8).unwrap();
    let energy = bridge.step_and_estimate(5.0, 0.1).unwrap();
    assert!(energy.is_finite(), "Energy estimate must be finite");
    assert!(energy > 0.0, "Energy estimate must be positive");
}

#[test]
fn test_thermal_bridge_rejects_invalid_params() {
    let params = crate::set9_telemetry::thermal_rc::ThermalParams::new(300.0, 10.0, 1.0).unwrap();
    assert!(
        PimThermalBridge::new(params, 0.0, 1.0e-8).is_err(),
        "Zero base energy must error"
    );
    assert!(
        PimThermalBridge::new(params, 1.0e-6, -1.0).is_err(),
        "Negative thermal coefficient must error"
    );
}

// ============================================================
// ENTROPY PLACEMENT -- 3 tests
// ============================================================

#[test]
fn test_entropy_placement_selects_lowest_energy() {
    let mut monitor = crate::set9_telemetry::multi_domain::MultiDomainRapl::new();
    monitor.inject(
        crate::set9_telemetry::multi_domain::RaplDomain::Package,
        10.0,
    );
    monitor.inject(crate::set9_telemetry::multi_domain::RaplDomain::Core, 5.0); // lowest
    monitor.inject(crate::set9_telemetry::multi_domain::RaplDomain::Uncore, 8.0);
    monitor.inject(crate::set9_telemetry::multi_domain::RaplDomain::Dram, 12.0);
    monitor.inject(crate::set9_telemetry::multi_domain::RaplDomain::Psu, 15.0);

    let placement = EntropyPimPlacement::new(monitor, 0.1).unwrap();
    let result = placement.place(100.0).unwrap();
    assert_eq!(
        result.domain,
        crate::set9_telemetry::multi_domain::RaplDomain::Core,
        "Must select domain with lowest energy"
    );
    assert_eq!(result.estimated_energy, 5.0);
}

#[test]
fn test_entropy_placement_uninitialized_error() {
    let monitor = crate::set9_telemetry::multi_domain::MultiDomainRapl::new(); // all zeros
    let placement = EntropyPimPlacement::new(monitor, 0.1).unwrap();
    let result = placement.place(100.0);
    assert!(result.is_err(), "All zeros must return uninitialized error");
}

#[test]
fn test_entropy_placement_threshold_respected() {
    let mut monitor = crate::set9_telemetry::multi_domain::MultiDomainRapl::new();
    monitor.inject(
        crate::set9_telemetry::multi_domain::RaplDomain::Package,
        10.0,
    );

    let placement = EntropyPimPlacement::new(monitor, 1000.0).unwrap(); // high threshold
    let sufficient = placement.entropy_sufficient(100.0).unwrap();
    // theta(100) ~ 176.6, which is < 1000, so this should be false
    assert!(
        !sufficient,
        "theta(100) ~ 176.6 must be below threshold 1000"
    );
}
