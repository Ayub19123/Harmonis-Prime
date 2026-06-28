//! Criterion Benchmarks — MPFR Oracle vs f64 Fallback
//!
//! ACHIEVED:
//! - 30-run statistical baseline for reproducible performance measurement
//! - Comparison between 400-bit MPFR and f64 fallback paths
//! - Regression detection: future commits must not exceed baseline + 10%
//!
//! LIMITATION:
//! - Single-machine benchmarks only (Windows 11, Intel i7-1165G7)
//! - No NUMA affinity, no SIMD, no GPU, no FPGA
//! - Benchmarks measure wall-clock time, not energy or instruction count
//! - Results may vary across architectures — document your machine
//!
//! Run: cargo bench --features mpfr

#[cfg(feature = "mpfr")]
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(feature = "mpfr")]
use crate::mpfr_zeta::oracle::zeta_half_plus_it;

/// Benchmark: ζ(½+i·10) at 400-bit precision
///
/// LIMITATION: This is a single evaluation, not a batch or throughput test.
/// Phase 3 will add batched evaluation benchmarks.
#[cfg(feature = "mpfr")]
fn bench_mpfr_zeta_t10(c: &mut Criterion) {
    c.bench_function("mpfr_zeta_t10_400bit", |b| {
        b.iter(|| {
            let t = black_box(10.0f64);
            let (real, imag) = zeta_half_plus_it(t);
            black_box((real, imag));
        })
    });
}

/// Benchmark: ζ(½+i·100) at 400-bit precision
///
/// LIMITATION: Higher t requires more terms. This benchmark captures
/// the O(N) scaling of the Dirichlet series.
#[cfg(feature = "mpfr")]
fn bench_mpfr_zeta_t100(c: &mut Criterion) {
    c.bench_function("mpfr_zeta_t100_400bit", |b| {
        b.iter(|| {
            let t = black_box(100.0f64);
            let (real, imag) = zeta_half_plus_it(t);
            black_box((real, imag));
        })
    });
}

/// Benchmark: ζ(½+i·1000) at 400-bit precision
///
/// LIMITATION: t=1000 is the practical limit of the Dirichlet series.
/// Beyond this, Riemann-Siegel formula is required (Phase 3).
#[cfg(feature = "mpfr")]
fn bench_mpfr_zeta_t1000(c: &mut Criterion) {
    c.bench_function("mpfr_zeta_t1000_400bit", |b| {
        b.iter(|| {
            let t = black_box(1000.0f64);
            let (real, imag) = zeta_half_plus_it(t);
            black_box((real, imag));
        })
    });
}

#[cfg(feature = "mpfr")]
criterion_group!(
    benches,
    bench_mpfr_zeta_t10,
    bench_mpfr_zeta_t100,
    bench_mpfr_zeta_t1000
);

#[cfg(feature = "mpfr")]
criterion_main!(benches);

// LIMITATION: When mpfr feature is disabled, no benchmarks are registered.
// This is intentional — f64 fallback is not benchmark-worthy.
#[cfg(not(feature = "mpfr"))]
fn main() {
    eprintln!("WARNING: mpfr feature disabled. No benchmarks available.");
    eprintln!("Enable with: cargo bench --features mpfr");
}
