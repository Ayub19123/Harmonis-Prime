//! Criterion Benchmark -- 30-run scalar zeta baseline.
//!
//! HONEST SCOPE (M1.4):
//! - Single-machine benchmark, no statistical CI from multiple machines
//! - 30 runs for mean/median/std reporting
//! - No RAPL energy measurement (software-only)
//! - No Level A/B/C build separation
//!
//! Honest limitation: This measures wall-clock latency on one Windows laptop.
//! NOT a production benchmark. NOT hardware-independent.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_theta_approximation(c: &mut Criterion) {
    use sovereign_core::set10_fusion::ThetaApproximation;
    
    let theta = ThetaApproximation::new();
    let t = 1000.0;
    
    c.bench_function("theta_approx_t1000", |b| {
        b.iter(|| {
            let _ = black_box(theta.evaluate(black_box(t)));
        })
    });
}

fn bench_extended_series(c: &mut Criterion) {
    use sovereign_core::set10_fusion::ExtendedDirichletSeries;
    
    let series = ExtendedDirichletSeries::new(100).unwrap();
    
    c.bench_function("dirichlet_series_100terms_sigma2", |b| {
        b.iter(|| {
            let _ = black_box(series.evaluate(black_box(2.0), black_box(0.0)));
        })
    });
}

fn bench_mpfr_oracle(c: &mut Criterion) {
    use sovereign_core::mpfr_oracle::theta_mpfr;
    
    let t = 1000.0;
    
    c.bench_function("mpfr_oracle_theta_t1000", |b| {
        b.iter(|| {
            let _ = black_box(theta_mpfr(black_box(t)));
        })
    });
}

criterion_group!(benches, bench_theta_approximation, bench_extended_series, bench_mpfr_oracle);
criterion_main!(benches);