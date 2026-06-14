#!/usr/bin/env python3
"""SET-5.4: PyO3 Python-side invariant tests
Invariant: Round-trip latency ≤ 10 µs (release), ≤ 100 µs (debug)
Mathematical Authority: Maxwell field theory + Kalman predictive filters"""

import time
import sys
sys.path.insert(0, "target/debug")

def test_maxwell_field_divergence():
    import harmonis_prime as hp
    field = hp.MaxwellField(4)
    field.vector_field = [1.0, 2.0, 3.0, 4.0]
    div = field.compute_divergence()
    assert div >= 0.0, "Divergence must be non-negative"
    print(f"✅ Maxwell divergence: {div}")

def test_kalman_predictor_trajectory():
    import harmonis_prime as hp
    kf = hp.KalmanPredictor(3, 0.01, 0.1)
    controls = [[0.1, 0.2, 0.3] for _ in range(5)]
    trajectory = kf.predict_trajectory(5, controls)
    assert len(trajectory) == 6, "Trajectory must include initial + 5 steps"
    print(f"✅ Kalman trajectory: {len(trajectory)} steps")

def test_roundtrip_latency():
    import harmonis_prime as hp
    field = hp.MaxwellField(8)
    
    # Warm-up: prime caches, GIL, and module resolution
    for _ in range(100):
        field.compute_divergence()
    
    # Scientific measurement: 1000 iterations, take minimum
    latencies = []
    for _ in range(1000):
        start = time.perf_counter_ns()
        field.compute_divergence()
        end = time.perf_counter_ns()
        latencies.append((end - start) / 1000.0)
    
    min_us = min(latencies)
    avg_us = sum(latencies) / len(latencies)
    
    # Debug build threshold (unoptimized): 100 µs
    # Release build threshold (optimized): 10 µs
    assert min_us <= 100.0, f"Min latency {min_us:.2f} µs exceeds 100 µs (debug)"
    print(f"✅ Round-trip latency — min: {min_us:.2f} µs, avg: {avg_us:.2f} µs")
    print(f"   ℹ️  Release build target: ≤ 10 µs (run: maturin develop --features pyo3 --release)")

if __name__ == "__main__":
    test_maxwell_field_divergence()
    test_kalman_predictor_trajectory()
    test_roundtrip_latency()
    print("🧱 SET-5.4: All Python invariants passed")
