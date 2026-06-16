//! SET-6D: Network Calculus Invariant Tests
//! 
//! Invariants:
//!   - delay_bound: W <= calculated supremum (no queueing violation)
//!   - bandwidth_guarantee: service_rate >= arrival_rate (stability)
//!   - jitter_control: inter-packet variance bounded by token bucket

use crate::network_calculus::curves::*;

// --- Delay Bound Invariant ---

#[test]
fn test_delay_bound_sub_microsecond() {
    // 1 Gbps arrival, 10 Gbps service, 1000 byte burst
    let alpha = ArrivalCurve::new(1_000_000_000.0, 1000.0).unwrap();
    let beta = ServiceCurve::new(10_000_000_000.0).unwrap();
    
    let bound = compute_delay_bound(&alpha, &beta).unwrap();
    
    // W = 1000 / (10G - 1G) = 111.1 ns — sub-microsecond
    assert!(bound.bound_ns < 1000.0, 
        "Delay bound {:.2} ns exceeds 1 us", bound.bound_ns);
    assert!(bound.bound_ns > 0.0, 
        "Delay bound must be positive");
    assert!(bound.stable, 
        "System must be stable");
}

#[test]
fn test_delay_bound_unstable_rejected() {
    // 10 Gbps arrival, 1 Gbps service — unstable
    let alpha = ArrivalCurve::new(10_000_000_000.0, 1000.0).unwrap();
    let beta = ServiceCurve::new(1_000_000_000.0).unwrap();
    
    let result = compute_delay_bound(&alpha, &beta);
    assert!(result.is_err(), 
        "Unstable system (arrival > service) must be rejected");
}

// --- Bandwidth Guarantee Invariant ---

#[test]
fn test_bandwidth_guarantee_stable() {
    // Service rate must strictly exceed arrival rate
    let alpha = ArrivalCurve::new(1_000_000.0, 500.0).unwrap();
    let beta = ServiceCurve::new(2_000_000.0).unwrap();
    
    let bound = compute_delay_bound(&alpha, &beta).unwrap();
    assert!(bound.stable);
    assert!(bound.service_rate > bound.arrival_rate);
}

#[test]
fn test_bandwidth_guarantee_equal_rates_unstable() {
    // Equal rates = zero rate difference = infinite delay
    let alpha = ArrivalCurve::new(1_000_000.0, 500.0).unwrap();
    let beta = ServiceCurve::new(1_000_000.0).unwrap();
    
    let result = compute_delay_bound(&alpha, &beta);
    assert!(result.is_err(), 
        "Equal rates must be rejected (infinite delay)");
}

// --- Jitter Control Invariant ---

#[test]
fn test_jitter_token_bucket_bounds_burst() {
    // Token bucket with rate 1 Mbps, depth 1000 bytes
    let tb = token_bucket_arrival(1_000_000.0, 1000.0).unwrap();
    
    // At t=0, max burst is 1000 bytes
    assert_eq!(tb.evaluate(0.0), 1000.0);
    
    // At t=1ms, cumulative is 1000 + 1000 = 2000 bytes
    assert_eq!(tb.evaluate(0.001), 2000.0);
}

#[test]
fn test_jitter_leaky_bucket_regulation() {
    // Bucket capacity 500 bytes, current 400 bytes
    let (output, bucket) = leaky_bucket_regulator(100.0, 500.0, 400.0);
    assert_eq!(output, 100.0); // fits
    assert_eq!(bucket, 500.0); // 400 + 100 = 500 (capped)
    
    // Overflow: bucket 450, input 100, max 500
    let (output2, bucket2) = leaky_bucket_regulator(100.0, 500.0, 450.0);
    assert_eq!(output2, 100.0); // still fits (450+100=550, capped to 500, output 100)
    assert_eq!(bucket2, 500.0); // capped
}

// --- Mathematical Foundation Invariant ---

#[test]
fn test_subadditivity_network_calculus_valid() {
    // Sub-additivity is required for network calculus
    let alpha = ArrivalCurve::new(1000.0, 500.0).unwrap();
    
    // Test multiple (t,s) pairs
    assert!(alpha.check_subadditive(0.0, 0.0));
    assert!(alpha.check_subadditive(1.0, 2.0));
    assert!(alpha.check_subadditive(10.0, 5.0));
    assert!(alpha.check_subadditive(0.001, 0.002));
}