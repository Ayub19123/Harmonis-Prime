use sovereign_core::brick45::chaos::{ChaosResult, ChaosRunner};

#[test]
fn brick45_chaos_live_fire() {
    println!("\n🧱 BRICK-45 CHAOS ENGINEERING HARNESS 🧱📡🟦🔒");
    println!("==================================================");
    println!("Target: BRICK-42 Quantum-Incorporated Sovereign");
    println!("Mode: DETERMINISTIC FAILURE INJECTION");
    println!("Threshold: 90% PASS RATE MINIMUM");
    println!("==================================================\n");

    let mut runner = ChaosRunner::new("BRICK45_LIVE_FIRE_001");
    let results = runner.run_all_tests();

    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    let rate = (passed as f64 / total as f64) * 100.0;

    println!("\n==================================================");
    println!("CHAOS SUITE COMPLETE");
    println!("==================================================");
    println!("Passed: {}/{} ({:.1}%)", passed, total, rate);

    for (i, result) in results.iter().enumerate() {
        let status = if result.passed {
            "✅ PASS"
        } else {
            "❌ FAIL"
        };
        println!(
            "  [{:02}] {:<20} | {} | detect={:.2}ms | recovery={:.2}ms | {}",
            i + 1,
            format!("{:?}", result.scenario),
            status,
            result.detection_time_ms,
            result.recovery_time_ms,
            result.final_state
        );
        if !result.violations.is_empty() {
            println!("       ⚠️  VIOLATIONS: {:?}", result.violations);
        }
    }

    println!("==================================================");
    assert!(
        rate >= 90.0,
        "ARCHITECTURE FAILED — {} violations",
        total - passed
    );
    println!("🧱 RESULT: SOVEREIGN — ARCHITECTURE SURVIVED CHAOS");
    println!("==================================================");
}
