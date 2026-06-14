//! SET-5.1: Multi-Node Simulation Invariants
//! Validates consensus correctness under scaling, Byzantine noise, and node failures.

use proptest::prelude::*;
use sovereign_core::simulation::cluster::{ClusterSimulation, ClusterConfig};

proptest! {
    /// INVARIANT: Consensus achieves liveness with < 1/3 Byzantine nodes
    #[test]
    fn consensus_liveness_under_byzantine(
        node_count in 3usize..50,
        byzantine_ratio in 0.0_f64..0.33,
        message_count in 10usize..1000,
    ) {
        let config = ClusterConfig {
            node_count,
            byzantine_ratio,
            offline_ratio: 0.0,
            message_count,
            max_latency_micros: 50000,
        };
        
        let mut sim = ClusterSimulation::new(config).expect("Valid cluster");
        let report = sim.run().expect("Simulation completes");
        
        prop_assert!(report.consensus_achieved, "Consensus failed with {} nodes, {} Byzantine ratio", 
            node_count, byzantine_ratio);
        prop_assert!(report.successful_appends > 0, "No messages appended");
    }

    /// INVARIANT: Byzantine detection triggers at â‰¥ 41% malicious nodes (reliable threshold)
    #[test]
    fn byzantine_detection_threshold(
        node_count in 20usize..50,
        byzantine_ratio in 0.41_f64..0.45,
        message_count in 200usize..500,
    ) {
        let config = ClusterConfig {
            node_count,
            byzantine_ratio,
            offline_ratio: 0.0,
            message_count,
            max_latency_micros: 50000,
        };
        
        let mut sim = ClusterSimulation::new(config).expect("Valid cluster");
        let report = sim.run().expect("Simulation completes");
        
        prop_assert!(report.byzantine_detected, 
            "Failed to detect Byzantine noise at ratio {}", byzantine_ratio);
    }

    /// INVARIANT: Latency remains bounded under node failure
    #[test]
    fn latency_bounded_under_failure(
        node_count in 5usize..30,
        offline_ratio in 0.1_f64..0.25,
        message_count in 50usize..500,
    ) {
        let config = ClusterConfig {
            node_count,
            byzantine_ratio: 0.0,
            offline_ratio,
            message_count,
            max_latency_micros: 50000,
        };
        
        let mut sim = ClusterSimulation::new(config).expect("Valid cluster");
        let report = sim.run().expect("Simulation completes");
        
        prop_assert!(report.max_latency_micros <= 50000, 
            "Latency exceeded bound: {} Âµs", report.max_latency_micros);
        prop_assert!(report.avg_latency_micros > 0.0, "Zero average latency");
    }

    /// INVARIANT: Determinism hash is reproducible for identical configurations
    #[test]
    fn cluster_reproducibility_invariant(
        node_count in 3usize..20,
        message_count in 10usize..100,
    ) {
        let config = ClusterConfig {
            node_count,
            byzantine_ratio: 0.0,
            offline_ratio: 0.0,
            message_count,
            max_latency_micros: 50000,
        };
        
        let mut sim_a = ClusterSimulation::new(config.clone()).expect("Valid cluster A");
        let report_a = sim_a.run().expect("Simulation A completes");
        
        let mut sim_b = ClusterSimulation::new(config.clone()).expect("Valid cluster B");
        let report_b = sim_b.run().expect("Simulation B completes");
        
        prop_assert!(report_a.determinism_hash == report_b.determinism_hash,
            "Cluster simulation non-deterministic: {} vs {}",
            report_a.determinism_hash, report_b.determinism_hash);
    }
}

