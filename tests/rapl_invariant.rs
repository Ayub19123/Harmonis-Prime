//! SET-5.2: RAPL Hardware-in-the-Loop Invariant Tests

use sovereign_core::energy::rapl_bindings::JloCorrelation;

#[test]
fn jlo_correlation_math_invariant() {
    let software = vec![1e-6, 2e-6, 3e-6, 4e-6];
    let hardware = vec![1.1e-6, 1.9e-6, 3.2e-6, 4.2e-6];
    let corr = JloCorrelation::compute(&software, &hardware).unwrap();
    assert!(corr.correlation_coefficient.abs() <= 1.0);
    assert_eq!(corr.samples_count, software.len());
}

#[test]
fn perfect_correlation_invariant() {
    let software = vec![1e-6, 2e-6, 3e-6];
    let hardware = software.clone();
    let corr = JloCorrelation::compute(&software, &hardware).unwrap();
    assert!(corr.correlation_coefficient.abs() >= 0.99);
    assert!(corr.error_bound <= 0.01);
    assert!(corr.verify_invariant());
}

#[test]
fn rapl_fallback_on_windows() {
    use sovereign_core::energy::monitor::EnergyMonitor;
    use sovereign_core::energy::rapl_bindings::{RaplDomain, RaplHardwareMonitor};
    let mut monitor = RaplHardwareMonitor::new(RaplDomain::Package);
    let sample = monitor.sample("test_op");
    assert_eq!(
        sample.joules, 0.0,
        "Non-Linux RAPL should report 0.0 joules"
    );
    let report = monitor.report();
    assert_eq!(report.total_joules, 0.0, "Total joules should be 0.0");
    // On Windows, no samples are recorded because reading returns None, so total_operations may be 0
    // That's acceptable.
}

#[test]
fn jlo_error_bound_invariant() {
    let software_jlo = 2e-6;
    let hardware_jlo = 2.2e-6;
    let n = 100;
    let software: Vec<f64> = (0..n).map(|_| software_jlo).collect();
    let hardware: Vec<f64> = (0..n).map(|_| hardware_jlo).collect();
    let corr = JloCorrelation::compute(&software, &hardware).unwrap();
    let error = (software_jlo - hardware_jlo).abs() / hardware_jlo;
    if error <= 0.20 {
        assert!(corr.verify_invariant());
    }
}
