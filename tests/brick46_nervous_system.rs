use sovereign_core::brick42::fluid::tensor_router::TensorRouter;
use sovereign_core::brick42::quantum::qpu_engine::{QPUEngine, QuantumBackend};
use sovereign_core::brick46::cognitive::{CognitiveConfig, CognitiveModel, CognitiveNode};
use sovereign_core::brick46::fluid_flow::{FluidFlowEngine, TenantConfig};
use sovereign_core::brick46::homeostasis::{HomeostasisConfig, HomeostasisLoop};
use sovereign_core::brick46::quantum_synapse::{EvictionPolicy, QuantumSynapse};
use sovereign_core::brick46::sensorimotor::{
    BackpressurePolicy, ErrorSurgeReflex, LatencyReflex, SensorimotorMesh,
};
use sovereign_core::brick46::types::*;

#[test]
fn test_homeostasis_continuous_slo_scoring() {
    let config = HomeostasisConfig::production_default();
    let mut homeo = HomeostasisLoop::new(config);
    let snapshot = homeo.record_observation(0.001, 50.0, 0.5, 15000.0);
    assert!(snapshot.is_healthy());
    assert!(snapshot.slo_score > 0.9);
    let degraded = homeo.record_observation(0.05, 2000.0, 1.2, 1000.0);
    assert!(!degraded.is_healthy());
    assert!(degraded.violations.len() >= 2);
    for _ in 0..100 {
        homeo.record_observation(0.001, 50.0, 0.5, 15000.0);
    }
    assert!(homeo.history().len() <= 10000);
}

#[test]
fn test_sensorimotor_reflex_arcs_with_backpressure() {
    let mut mesh = SensorimotorMesh::new(100, BackpressurePolicy::DropOldest);
    mesh.register_reflex("latency".to_string(), LatencyReflex::new(100.0));
    mesh.register_reflex("error".to_string(), ErrorSurgeReflex::new(0.01));
    let event1 = ReflexEvent::new("node_1", "latency_spike", 150.0);
    let event2 = ReflexEvent::new("node_2", "error_surge", 0.02);
    assert!(mesh.ingest(event1));
    assert!(mesh.ingest(event2));
    let signals = mesh.process_queue();
    assert_eq!(signals.len(), 2);
    assert!(signals.iter().any(|s| s.anomaly_score > 0.8));
    for i in 0..200 {
        mesh.ingest(ReflexEvent::new(&format!("node_{}", i), "test", 1.0));
    }
    assert!(mesh.queue_depth() <= 100);
}

#[test]
fn test_quantum_synapse_superpositional_cache() {
    let qpu = QPUEngine::new(QuantumBackend::Simulated, 16);
    let router = TensorRouter::new();
    let mut synapse = QuantumSynapse::new(qpu, router, 100);
    let key = QStateKey {
        context: "test_context".to_string(),
        dimension: 8,
    };
    let value1 = synapse.evaluate_superposition(key.clone());
    assert_eq!(value1.amplitudes.len(), 8);
    assert!(value1.confidence >= 0.0 && value1.confidence <= 1.0);
    let value2 = synapse.evaluate_superposition(key.clone());
    assert_eq!(value1.amplitudes, value2.amplitudes);
    let (size, hits, misses) = synapse.cache_stats();
    assert_eq!(size, 1);
    assert_eq!(hits, 1);
    assert_eq!(misses, 1);
    for i in 0..150 {
        let k = QStateKey {
            context: format!("context_{}", i),
            dimension: 4,
        };
        synapse.evaluate_superposition(k);
    }
    let (size_after, _, _) = synapse.cache_stats();
    assert!(size_after <= 100);
}

#[test]
fn test_cognitive_anomaly_synthesis() {
    let mut config = CognitiveConfig::production_default();
    config.anomaly_threshold = 0.3;
    let mut cognitive = CognitiveNode::new(config);
    let health = HealthSnapshot {
        timestamp: std::time::Instant::now(),
        slo_score: 0.7,
        error_rate: 0.08,
        latency_p99_ms: 3000.0,
        energy_budget_used: 0.95,
        violations: vec!["error_rate_exceeded".to_string()],
    };
    let result = cognitive.analyze(&health, &[]);
    assert!(result.is_some());
    let signal = result.unwrap();
    assert!(signal.anomaly_score >= 0.3);
    assert_eq!(
        signal.recommended_action.unwrap(),
        "emergency_circuit_break"
    );
}

#[test]
fn test_fluid_flow_zero_friction_api() {
    let mut engine = FluidFlowEngine::new();
    engine.register_tenant(
        "finance",
        TenantConfig {
            max_payload_bytes: 1024 * 1024,
            allowed_operations: vec!["predict".to_string(), "query".to_string()],
            cognitive_enhanced: true,
        },
    );
    let request = ApiFlowRequest::new("finance", "predict", 512);
    let hint = CognitiveSignal::new("corr_1", "Latency spike detected", 0.8)
        .with_action("scale_compute_pool");
    let response = engine.handle_request(&request, Some(&hint));
    assert!(response.success);
    assert!(response.message.contains("cognitive_hint"));
    let bad = engine.handle_request(&ApiFlowRequest::new("unknown", "predict", 100), None);
    assert!(!bad.success);
    let big = engine.handle_request(
        &ApiFlowRequest::new("finance", "predict", 2 * 1024 * 1024),
        None,
    );
    assert!(!big.success);
    let op = engine.handle_request(&ApiFlowRequest::new("finance", "delete", 100), None);
    assert!(!op.success);
}

#[test]
fn test_full_nervous_system_cascade() {
    let mut homeo = HomeostasisLoop::new(HomeostasisConfig::production_default());
    let mut mesh = SensorimotorMesh::new(50, BackpressurePolicy::Shed(0.1));
    let mut cognitive = CognitiveNode::new(CognitiveConfig::production_default());
    let mut fluid = FluidFlowEngine::new();
    fluid.register_tenant(
        "ops",
        TenantConfig {
            max_payload_bytes: 1024,
            allowed_operations: vec!["health_check".to_string()],
            cognitive_enhanced: true,
        },
    );
    let health = homeo.record_observation(0.02, 500.0, 0.8, 8000.0);
    mesh.register_reflex("latency".to_string(), LatencyReflex::new(100.0));
    mesh.ingest(ReflexEvent::new("edge_1", "latency_spike", 200.0));
    let signals = mesh.process_queue();
    let cognitive_signal = cognitive.analyze(&health, &signals);
    assert!(cognitive_signal.is_some());
    let request = ApiFlowRequest::new("ops", "health_check", 100);
    let response = fluid.handle_request(&request, cognitive_signal.as_ref());
    assert!(response.success);
    assert!(response.message.contains("cognitive_hint"));
}
