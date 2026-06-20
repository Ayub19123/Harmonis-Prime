//! SET-9: Multi-Domain Telemetry & Energy Balancing — 15 Invariant Tests
//!
//! Invariants:
//!   - multi_domain: 5 tests (enumeration, read, inject, count, determinism)
//!   - thermal_rc: 3 tests (convergence, monotonicity, params immutability)
//!   - jlo_correlation: 4 tests (perfect correlation, anti-correlation,
//!     length error, zero variance error)
//!   - domain_balancer: 3 tests (lowest energy selected, specific placement,
//!     uninitialized error)

use crate::set9_telemetry::*;

// ============================================================
// MULTI-DOMAIN RAPL — 5 tests
// ============================================================

#[test]
fn test_domain_enumeration_complete() {
    let all = RaplDomain::all();
    assert_eq!(all.len(), 5, "Must enumerate exactly 5 domains");
    assert!(all.contains(&RaplDomain::Package));
    assert!(all.contains(&RaplDomain::Core));
    assert!(all.contains(&RaplDomain::Uncore));
    assert!(all.contains(&RaplDomain::Dram));
    assert!(all.contains(&RaplDomain::Psu));
}

#[test]
fn test_read_domain_simulated_zero() {
    let monitor = MultiDomainRapl::new();
    assert_eq!(monitor.read_domain(RaplDomain::Package), 0.0);
    assert_eq!(monitor.read_domain(RaplDomain::Core), 0.0);
}

#[test]
fn test_inject_and_read_deterministic() {
    let mut monitor = MultiDomainRapl::new();
    monitor.inject(RaplDomain::Package, 42.0);
    assert_eq!(monitor.read_domain(RaplDomain::Package), 42.0);
    assert_eq!(monitor.read_domain(RaplDomain::Core), 0.0);
}

#[test]
fn test_domain_count_five() {
    let monitor = MultiDomainRapl::new();
    assert_eq!(monitor.domain_count(), 5);
}

#[test]
fn test_multi_domain_read_all_length() {
    let monitor = MultiDomainRapl::new();
    let all = monitor.read_all();
    assert_eq!(all.len(), 5);
}

// ============================================================
// THERMAL RC — 3 tests
// ============================================================

#[test]
fn test_thermal_convergence_per_domain() {
    let params = ThermalParams::new(300.0, 10.0, 0.1).unwrap();
    let mut thermal = DomainThermalModel::new(params);
    let power = 5.0;
    let expected = thermal.steady_state(power);

    for _ in 0..1000 {
        thermal.step(power, 0.01).unwrap();
    }

    assert!((thermal.temperature() - expected).abs() < 0.01,
        "Must converge to steady state {} (got {})",
        expected, thermal.temperature());
}

#[test]
fn test_thermal_monotonic_power_higher_temp() {
    let p1 = ThermalParams::new(300.0, 10.0, 0.1).unwrap();
    let p2 = ThermalParams::new(300.0, 10.0, 0.1).unwrap();
    let mut t1 = DomainThermalModel::new(p1);
    let mut t2 = DomainThermalModel::new(p2);

    for _ in 0..100 {
        t1.step(5.0, 0.1).unwrap();
        t2.step(10.0, 0.1).unwrap();
    }

    assert!(t2.temperature() > t1.temperature(),
        "Higher power must produce higher temperature");
}

#[test]
fn test_thermal_params_immutable() {
    let params = ThermalParams::new(300.0, 10.0, 1.0).unwrap();
    let thermal = DomainThermalModel::new(params);
    let retrieved = thermal.params();
    assert_eq!(retrieved.ambient_temp, 300.0);
    assert_eq!(retrieved.thermal_resistance, 10.0);
    assert_eq!(retrieved.thermal_capacitance, 1.0);
}

// ============================================================
// JLO CORRELATION — 4 tests
// ============================================================

#[test]
fn test_correlation_perfect_positive() {
    let pair = DomainPair::new(RaplDomain::Package, RaplDomain::Core).unwrap();
    let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let b = vec![2.0, 4.0, 6.0, 8.0, 10.0]; // perfectly correlated
    let corr = JloCorrelation::compute(pair, &a, &b).unwrap();
    assert!((corr.pearson_r - 1.0).abs() < 1e-12,
        "Perfect correlation must yield r=1, got {}", corr.pearson_r);
    assert!(corr.is_strong());
    assert!(corr.is_positive());
}

#[test]
fn test_correlation_perfect_negative() {
    let pair = DomainPair::new(RaplDomain::Package, RaplDomain::Dram).unwrap();
    let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let b = vec![5.0, 4.0, 3.0, 2.0, 1.0]; // perfectly anti-correlated
    let corr = JloCorrelation::compute(pair, &a, &b).unwrap();
    assert!((corr.pearson_r + 1.0).abs() < 1e-12,
        "Perfect anti-correlation must yield r=-1, got {}", corr.pearson_r);
    assert!(corr.is_strong());
    assert!(!corr.is_positive());
}

#[test]
fn test_correlation_length_mismatch_error() {
    let pair = DomainPair::new(RaplDomain::Core, RaplDomain::Uncore).unwrap();
    let a = vec![1.0, 2.0];
    let b = vec![1.0, 2.0, 3.0];
    let result = JloCorrelation::compute(pair, &a, &b);
    assert!(result.is_err(), "Length mismatch must return error");
}

#[test]
fn test_correlation_zero_variance_error() {
    let pair = DomainPair::new(RaplDomain::Package, RaplDomain::Psu).unwrap();
    let a = vec![5.0, 5.0, 5.0]; // zero variance
    let b = vec![1.0, 2.0, 3.0];
    let result = JloCorrelation::compute(pair, &a, &b);
    assert!(result.is_err(), "Zero variance must return error");
}

// ============================================================
// DOMAIN BALANCER — 3 tests
// ============================================================

#[test]
fn test_balancer_selects_lowest_energy() {
    let mut monitor = MultiDomainRapl::new();
    monitor.inject(RaplDomain::Package, 10.0);
    monitor.inject(RaplDomain::Core, 5.0);    // lowest
    monitor.inject(RaplDomain::Uncore, 8.0);
    monitor.inject(RaplDomain::Dram, 12.0);
    monitor.inject(RaplDomain::Psu, 15.0);

    let balancer = DomainBalancer::new(monitor);
    let placement = balancer.place().unwrap();
    assert_eq!(placement.domain, RaplDomain::Core,
        "Must select domain with lowest energy");
    assert_eq!(placement.estimated_energy, 5.0);
}

#[test]
fn test_balancer_place_on_specific() {
    let mut monitor = MultiDomainRapl::new();
    monitor.inject(RaplDomain::Dram, 7.0);

    let balancer = DomainBalancer::new(monitor);
    let placement = balancer.place_on(RaplDomain::Dram).unwrap();
    assert_eq!(placement.domain, RaplDomain::Dram);
    assert_eq!(placement.estimated_energy, 7.0);
}

#[test]
fn test_balancer_uninitialized_error() {
    let monitor = MultiDomainRapl::new(); // all zeros
    let balancer = DomainBalancer::new(monitor);
    let result = balancer.place();
    assert!(result.is_err(), "All zeros must return uninitialized error");
}