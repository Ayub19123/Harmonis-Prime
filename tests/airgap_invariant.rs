//! CATEGORY 4: Air-Gapped Sovereignty Invariant Tests
//! Validates zero external dependency execution and deterministic offline output.

use proptest::prelude::*;
use sovereign_core::sovereign::isolation::{
    SovereignEnvironment, SyntheticDataset, IsolationBarrier, ExternalOperation
};

proptest! {
    /// INVARIANT: Environment is air-gapped after seal
    #[test]
    fn airgap_seal_invariant(
        ops in 0usize..10,
    ) {
        let mut barrier = IsolationBarrier::new();
        barrier.seal();
        
        // Any external operation after seal must fail
        for i in 0..ops {
            let result = barrier.record(ExternalOperation::HttpRequest { 
                url: format!("http://example{}.com", i) 
            });
            prop_assert!(result.is_err(), "Isolation barrier allowed operation {} after seal", i);
        }
        
        prop_assert!(barrier.verify_airgapped(), "Barrier not air-gapped after seal");
        prop_assert_eq!(barrier.violation_count(), 0, "Violations recorded despite errors");
    }

    /// INVARIANT: Synthetic dataset generation is deterministic
    #[test]
    fn synthetic_determinism_invariant(
        label in "[a-z]{1,10}",
        size in 10usize..1000,
    ) {
        let dataset_a = SyntheticDataset::generate(&label, size);
        let dataset_b = SyntheticDataset::generate(&label, size);
        
        prop_assert_eq!(dataset_a.payload_stream, dataset_b.payload_stream, "Payloads non-deterministic");
        prop_assert_eq!(dataset_a.entropy_profile, dataset_b.entropy_profile, "Entropy profiles non-deterministic");
        prop_assert_eq!(dataset_a.quantum_seeds, dataset_b.quantum_seeds, "Seeds non-deterministic");
    }

    /// INVARIANT: Sovereignty heartbeat completes without external dependencies
    #[test]
    fn heartbeat_zero_dependency_invariant(
        dataset_size in 50usize..500,
    ) {
        let mut env = SovereignEnvironment::new().expect("Valid environment");
        env.seal();
        
        let dataset = SyntheticDataset::generate("airgap_test", dataset_size);
        let report = env.heartbeat(&dataset).expect("Heartbeat must succeed");
        
        prop_assert!(report.isolation_verified, "Isolation not verified after heartbeat");
        prop_assert!(report.dag_messages_processed > 0, "No DAG messages processed");
        prop_assert!(report.entropy_states_recorded > 0, "No entropy states recorded");
        prop_assert!(report.quantum_collapses_executed > 0, "No quantum collapses executed");
        prop_assert!(report.energy_jlo > 0.0, "No energy measured");
        prop_assert!(!report.determinism_hash.is_empty(), "No determinism hash produced");
    }

    /// INVARIANT: Same dataset produces same determinism hash (reproducibility)
    #[test]
    fn heartbeat_reproducibility_invariant(
        dataset_size in 50usize..500,
    ) {
        let mut env_a = SovereignEnvironment::new().expect("Valid environment");
        env_a.seal();
        let mut env_b = SovereignEnvironment::new().expect("Valid environment");
        env_b.seal();
        
        let dataset = SyntheticDataset::generate("repro_test", dataset_size);
        let report_a = env_a.heartbeat(&dataset).expect("Heartbeat A must succeed");
        let report_b = env_b.heartbeat(&dataset).expect("Heartbeat B must succeed");
        
        // prop_assert! with == borrows; prop_assert_eq! would move the Strings
        prop_assert!(report_a.determinism_hash == report_b.determinism_hash,
            "Reproducibility broken: {} vs {}", report_a.determinism_hash, report_b.determinism_hash);
    }
}