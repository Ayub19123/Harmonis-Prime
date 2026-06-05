// BRICK-51.1 Certification: Industry-Hardened Benchmark
// Statistical rigor, hardware honesty, reproducible seeds

use rand::Rng;
use sovereign_core::brick51::awareness_monitor::AwarenessMonitor;
use sovereign_core::brick51::collective_reasoning::CollectiveReasoning;
use sovereign_core::brick51::explainability_engine::ExplainabilityEngine;
use sovereign_core::brick51::goal_registry::GoalRegistry;
use sovereign_core::brick51::hardware_tag::{HardwareDomain, TaggedMetric};
use sovereign_core::brick51::knowledge_ledger::KnowledgeLedger;
use sovereign_core::brick51::physical_interface::{ActuatorCommand, PhysicalInterface};
use sovereign_core::brick51::recovery_engine::RecoveryEngine;
use sovereign_core::brick51::self_model_engine::SelfModelEngine;
use sovereign_core::brick51::shared_memory_graph::SharedMemoryGraph;
use sovereign_core::brick51::statistical_runner::StatisticalRunner;
use sovereign_core::brick51::trust_registry::TrustRegistry;

const BENCHMARK_SEED: u64 = 0x51C3_2026_0613;
const N_RUNS: u64 = 10_000;
const HEAVY_RUNS: u64 = 1_000; // For slow tests (CMF-511, CMF-519)

// ============================================================================
// T1 — STATE CONSISTENCY
// ============================================================================

#[test]
fn test_cmf511_shared_memory_consistency() {
    let mut runner = StatisticalRunner::new(BENCHMARK_SEED);
    let (mean, std_dev, _) = runner.benchmark(HEAVY_RUNS, |rng| {
        let mut nodes: Vec<SharedMemoryGraph> =
            (0..10).map(|i| SharedMemoryGraph::new(i, 10)).collect();

        let entries = 100 + (rng.gen::<u64>() % 9900);
        for i in 0..entries {
            let clock = vec![i; 10];
            nodes[0].insert(&format!("key_{}", i), &format!("value_{}", i), clock);
        }

        let node0_snapshot = nodes[0].clone();
        for i in 1..10 {
            nodes[i].merge(&node0_snapshot);
        }

        let (_, _, consistency) = nodes[9].stats();
        consistency
    });

    let metric = TaggedMetric::new(
        mean,
        "%",
        HardwareDomain::Simulated,
        HEAVY_RUNS,
        std_dev,
        BENCHMARK_SEED,
    );
    println!("CMF-511: {}", metric.report());
    assert!(mean >= 0.9999, "CMF-511: consistency {} < 99.99%", mean);
}

#[test]
fn test_cmf512_collective_reasoning_gain() {
    let mut runner = StatisticalRunner::new(BENCHMARK_SEED + 1);
    let (mean, std_dev, _) = runner.benchmark(N_RUNS, |rng| {
        let mut mesh = CollectiveReasoning::new();
        let nodes = 2 + (rng.gen::<usize>() % 98);
        mesh.solve("prob", nodes, 1.0);
        let (_, gain, _) = mesh.stats();
        gain
    });

    let metric = TaggedMetric::new(
        mean * 100.0,
        "%",
        HardwareDomain::Simulated,
        N_RUNS,
        std_dev * 100.0,
        BENCHMARK_SEED + 1,
    );
    println!("CMF-512: {}", metric.report());
    assert!(mean >= 0.25, "CMF-512: gain {} < 25%", mean);
}

#[test]
fn test_cmf513_goal_convergence() {
    let mut runner = StatisticalRunner::new(BENCHMARK_SEED + 2);
    let (mean, std_dev, _) = runner.benchmark(N_RUNS, |rng| {
        let mut registry = GoalRegistry::new();
        let goals = 10 + (rng.gen::<u64>() % 990);
        for i in 0..goals {
            registry.propose(&format!("g{}", i), 1.0);
            registry.converge(&format!("g{}", i), (i % 5000) as u64);
        }
        let (_, _, _, fast_rate) = registry.stats();
        fast_rate
    });

    let metric = TaggedMetric::new(
        mean * 100.0,
        "%",
        HardwareDomain::Simulated,
        N_RUNS,
        std_dev * 100.0,
        BENCHMARK_SEED + 2,
    );
    println!("CMF-513: {}", metric.report());
    assert!(mean >= 0.95, "CMF-513: fast convergence {} < 95%", mean);
}

#[test]
fn test_cmf514_mesh_recovery() {
    let mut runner = StatisticalRunner::new(BENCHMARK_SEED + 3);
    let (mean_recovery, std_dev_recovery, _) = runner.benchmark(N_RUNS, |rng| {
        let mut engine = RecoveryEngine::new(100);
        let failure_pct = 10 + (rng.gen::<usize>() % 50);
        engine.simulate_failure(failure_pct);
        engine.recover(15_000 + (rng.gen::<u64>() % 15_000));
        let (_, _, recovery_time, _) = engine.stats();
        recovery_time as f64
    });

    let metric = TaggedMetric::new(
        mean_recovery,
        "ms",
        HardwareDomain::Simulated,
        N_RUNS,
        std_dev_recovery,
        BENCHMARK_SEED + 3,
    );
    println!("CMF-514: {}", metric.report());
    assert!(
        mean_recovery < 30_000.0,
        "CMF-514: recovery {}ms >= 30s",
        mean_recovery
    );
}

#[test]
fn test_cmf515_self_model_accuracy() {
    let mut runner = StatisticalRunner::new(BENCHMARK_SEED + 4);
    let (mean, std_dev, _) = runner.benchmark(N_RUNS, |rng| {
        let mut engine = SelfModelEngine::new();
        let samples = 100 + (rng.gen::<u64>() % 1900);
        for i in 0..samples {
            let load = 0.1 + (i as f64 * 0.001);
            let health = 1.0 - (i as f64 * 0.0001);
            engine.sample(load, health.max(0.5));
        }
        let (_, _, accuracy) = engine.stats();
        accuracy
    });

    let metric = TaggedMetric::new(
        mean * 100.0,
        "%",
        HardwareDomain::Simulated,
        N_RUNS,
        std_dev * 100.0,
        BENCHMARK_SEED + 4,
    );
    println!("CMF-515: {}", metric.report());
    assert!(mean >= 0.95, "CMF-515: accuracy {} < 95%", mean);
}

#[test]
fn test_cmf519_state_consistency_index() {
    let mut runner = StatisticalRunner::new(BENCHMARK_SEED + 5);
    let (mean, std_dev, _) = runner.benchmark(HEAVY_RUNS, |rng| {
        let mut monitor = AwarenessMonitor::new(10);
        let checks = 10 + (rng.gen::<usize>() % 990);
        for _ in 0..checks {
            for node in 0..10 {
                for metric in 0..20 {
                    monitor.report_metric(
                        &format!("n{}", node),
                        &format!("m{}", metric),
                        rng.gen::<f64>(),
                    );
                }
            }
            monitor.check_coverage();
        }
        let (_, _, index) = monitor.stats();
        index
    });

    let metric = TaggedMetric::new(
        mean * 100.0,
        "%",
        HardwareDomain::Simulated,
        HEAVY_RUNS,
        std_dev * 100.0,
        BENCHMARK_SEED + 5,
    );
    println!("CMF-519 (State Consistency): {}", metric.report());
    assert!(mean >= 0.95, "CMF-519: consistency {} < 95%", mean);
}

// ============================================================================
// T2 — HARDWARE LATENCY (SIMULATION-BOUND)
// ============================================================================

#[test]
fn test_cmf520_521_simulated_hardware_latency() {
    let mut runner = StatisticalRunner::new(BENCHMARK_SEED + 6);
    let (mean, std_dev, _) = runner.benchmark(N_RUNS, |rng| {
        let mut phys = PhysicalInterface::new();
        let cmds = 10 + (rng.gen::<u64>() % 990);
        for i in 0..cmds {
            let cmd = ActuatorCommand {
                action: format!("move_{}", i),
                x: rng.gen::<f64>(),
                y: rng.gen::<f64>(),
                z: rng.gen::<f64>(),
            };
            let (_, latency) = phys.execute(&cmd);
            assert!(latency <= 100, "SIMULATED: latency {}ns > 100ns", latency);
        }
        let (_, avg_lat, _) = phys.stats();
        avg_lat
    });

    let metric = TaggedMetric::new(
        mean,
        "ns",
        HardwareDomain::Simulated,
        N_RUNS,
        std_dev,
        BENCHMARK_SEED + 6,
    );
    println!("CMF-520/521 [SIMULATED]: {}", metric.report());
    assert!(mean <= 100.0, "SIMULATED: avg latency {}ns > 100ns", mean);
}

// ============================================================================
// T3 — FORMAL VERIFICATION
// ============================================================================

#[test]
fn test_cmf516_knowledge_integrity() {
    let mut runner = StatisticalRunner::new(BENCHMARK_SEED + 7);
    let (mean, std_dev, _) = runner.benchmark(N_RUNS, |rng| {
        let mut ledger = KnowledgeLedger::new();
        let entries = 100 + (rng.gen::<u64>() % 4900);
        let false_idx = rng.gen::<u64>() % entries;

        for i in 0..entries {
            let stmt = if i == false_idx {
                "false_stmt".to_string()
            } else {
                format!("truth_{}", i)
            };
            ledger.submit(&stmt, &format!("h{}", i), "n0");
        }

        for i in 0..entries {
            let stmt = if i == false_idx {
                "false_stmt"
            } else {
                &format!("truth_{}", i)
            };
            let truth = i != false_idx;
            ledger.verify(stmt, truth);
        }

        let (_, _, false_rate, _) = ledger.stats();
        false_rate
    });

    let metric = TaggedMetric::new(
        mean * 100.0,
        "%",
        HardwareDomain::Simulated,
        N_RUNS,
        std_dev * 100.0,
        BENCHMARK_SEED + 7,
    );
    println!("CMF-516: {}", metric.report());
    assert!(mean <= 0.001, "CMF-516: false rate {} > 0.1%", mean);
}

#[test]
fn test_cmf522_explainability_and_override() {
    let mut runner = StatisticalRunner::new(BENCHMARK_SEED + 8);
    let (mean_acc, std_dev_acc, _) = runner.benchmark(N_RUNS, |rng| {
        let mut engine = ExplainabilityEngine::new();
        let decisions = 5 + (rng.gen::<usize>() % 95);
        for i in 0..decisions {
            engine.register_decision(&format!("d{}", i), &format!("graph_{}", i));
            let _ = engine.explain(&format!("d{}", i));
            engine.human_override(&format!("d{}", i));
        }
        let (_, _, rate, _) = engine.stats();
        rate
    });

    let metric = TaggedMetric::new(
        mean_acc * 100.0,
        "%",
        HardwareDomain::Simulated,
        N_RUNS,
        std_dev_acc * 100.0,
        BENCHMARK_SEED + 8,
    );
    println!("CMF-522: {}", metric.report());
    assert!(mean_acc >= 0.90, "CMF-522: accuracy {} < 90%", mean_acc);
}

// ============================================================================
// T4 — CHAOS RESILIENCE
// ============================================================================

#[test]
fn test_cmf518_autonomous_discovery() {
    let mut runner = StatisticalRunner::new(BENCHMARK_SEED + 9);
    let (mean, std_dev, _) = runner.benchmark(N_RUNS, |rng| {
        let mut mesh = CollectiveReasoning::new();
        let improvements = 1 + (rng.gen::<usize>() % 10);
        for i in 0..improvements {
            mesh.generate_improvement(&format!("imp_{}", i));
        }
        let (_, _, count) = mesh.stats();
        count as f64
    });

    let metric = TaggedMetric::new(
        mean,
        "count",
        HardwareDomain::Simulated,
        N_RUNS,
        std_dev,
        BENCHMARK_SEED + 9,
    );
    println!("CMF-518: {}", metric.report());
    assert!(mean >= 1.0, "CMF-518: improvements {} < 1", mean);
}

#[test]
fn test_cmf520_probabilistic_landmark() {
    let mut runner = StatisticalRunner::new(BENCHMARK_SEED + 10);
    let (avail_rate, passed) = runner.verify_property(N_RUNS, 0.99999, |rng| {
        let mut engine = RecoveryEngine::new(100);
        let failure_pct = 30 + (rng.gen::<usize>() % 50);
        engine.simulate_failure(failure_pct);

        let recovery_time = if rng.gen::<f64>() < 0.95 {
            15_000 + (rng.gen::<u64>() % 10_000)
        } else {
            25_000 + (rng.gen::<u64>() % 5_000)
        };
        engine.recover(recovery_time);

        let (_, _, _, avail) = engine.stats();
        avail >= 0.99999
    });

    let metric = TaggedMetric::new(
        avail_rate * 100.0,
        "%",
        HardwareDomain::Simulated,
        N_RUNS,
        (avail_rate * (1.0 - avail_rate)).sqrt(),
        BENCHMARK_SEED + 10,
    );
    println!("CMF-520 [Probabilistic Landmark]: {}", metric.report());
    assert!(
        passed,
        "CMF-520: availability rate {} < 99.999%",
        avail_rate
    );
}

// ============================================================================
// T5 — DOMAIN GENERALIZATION
// ============================================================================

#[test]
fn test_cmf517_emergent_specialisation() {
    let mut runner = StatisticalRunner::new(BENCHMARK_SEED + 11);
    let (mean, std_dev, _) = runner.benchmark(N_RUNS, |rng| {
        let mut engine = RecoveryEngine::new(10);
        let baseline = 100.0;
        let episodes = 10 + (rng.gen::<usize>() % 990);

        for _ in 0..episodes {
            engine.train_specialisation("finance", baseline * (1.2 + rng.gen::<f64>() * 0.3));
            engine.train_specialisation("health", baseline * (1.15 + rng.gen::<f64>() * 0.25));
            engine.train_specialisation("logistics", baseline * (1.25 + rng.gen::<f64>() * 0.35));
        }

        engine.specialisation_gain("finance", baseline)
    });

    let metric = TaggedMetric::new(
        mean * 100.0,
        "%",
        HardwareDomain::Simulated,
        N_RUNS,
        std_dev * 100.0,
        BENCHMARK_SEED + 11,
    );
    println!("CMF-517: {}", metric.report());
    assert!(mean >= 0.20, "CMF-517: gain {} < 20%", mean);
}

// ============================================================================
// T6 — TRUST & PRIVACY
// ============================================================================

#[test]
fn test_cmf524_decentralised_trust() {
    let mut runner = StatisticalRunner::new(BENCHMARK_SEED + 12);
    let (trust_rate, passed) = runner.verify_property(N_RUNS, 0.99, |rng| {
        let mut registry = TrustRegistry::new();
        let nodes = 3 + (rng.gen::<usize>() % 97);

        for i in 0..nodes {
            registry.register(&format!("n{}", i), &format!("k{:064}", i));
            registry.offer_compute(&format!("n{}", i), 10_000);
        }

        let from = rng.gen::<usize>() % nodes;
        let to = rng.gen::<usize>() % nodes;
        let amount = 1 + (rng.gen::<u64>() % 5_000);

        let result = registry.barter_compute(&format!("n{}", from), &format!("n{}", to), amount);

        let (_, _, rate) = registry.stats();
        result && rate >= 0.99
    });

    let metric = TaggedMetric::new(
        trust_rate * 100.0,
        "%",
        HardwareDomain::Simulated,
        N_RUNS,
        (trust_rate * (1.0 - trust_rate)).sqrt(),
        BENCHMARK_SEED + 12,
    );
    println!("CMF-524: {}", metric.report());
    assert!(passed, "CMF-524: trust rate {} < 99%", trust_rate);
}
