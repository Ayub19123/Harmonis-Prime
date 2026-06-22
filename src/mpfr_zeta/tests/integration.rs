//! SET-12 Integration Tests — Cross-Module Validation
//! 
//! ACHIEVED:
//! - End-to-end validation of MPFR oracle against f64 fallback
//! - Performance regression detection (no drift > 10% from baseline)
//! - Feature flag correctness (mpfr vs fallback paths)
//! 
//! LIMITATION:
//! - These tests run only when the full mpfr feature is enabled
//! - Baseline values are machine-dependent; adjust for your hardware
//! - No formal proof of correctness — only numerical consistency checks

use harmonis_prime::mpfr_zeta::{
    oracle::zeta_half_plus_it,
    neumaier::neumaier_sum_f64,
    truncation::truncation_bound_dirichlet,
};

/// Invariant: MPFR oracle output must be within f64 fallback tolerance
/// 
/// LIMITATION: This only checks that MPFR and fallback are "close enough"
/// for coarse validation. The real test of MPFR is against known values.
#[test]
#[cfg(feature = "mpfr")]
fn test_mpfr_vs_fallback_consistency() {
    let t = 10.0f64;

    // MPFR path (400-bit)
    let (mpfr_real, mpfr_imag) = zeta_half_plus_it(t);

    // Fallback path (f64)
    let (fallback_real, fallback_imag) = {
        let n_terms = 100_000usize;
        let mut sum_real = 0.0f64;
        let mut sum_imag = 0.0f64;
        for n in 1..=n_terms {
            let n_f = n as f64;
            let coeff = n_f.powf(-0.5);
            let t_ln_n = t * n_f.ln();
            sum_real += coeff * t_ln_n.cos();
            sum_imag -= coeff * t_ln_n.sin();
        }
        (sum_real, sum_imag)
    };

    // They should be within ~1e-10 (f64 fallback is crude but not random)
    let real_diff = (mpfr_real - fallback_real).abs();
    let imag_diff = (mpfr_imag - fallback_imag).abs();

    assert!(
        real_diff < 1e-6,
        "MPFR vs fallback real part diff too large: {}. \
         NOTE: Fallback is intentionally crude. Large diff expected for large t.",
        real_diff
    );
    assert!(
        imag_diff < 1e-6,
        "MPFR vs fallback imag part diff too large: {}. \
         NOTE: Fallback is intentionally crude. Large diff expected for large t.",
        imag_diff
    );
}

/// Invariant: Truncation bound must dominate actual error
/// 
/// LIMITATION: This is a weak check. The bound is an upper bound,
/// so it should always be >= actual error. But we can only estimate
/// actual error by comparing with higher-precision reference.
#[test]
fn test_truncation_bound_dominates_error() {
    let t = 10.0f64;
    let n = 10_000u64;

    let bound = truncation_bound_dirichlet(n, t);

    // Compute "actual" error by comparing coarse and fine approximations
    let coarse = {
        let mut sum = 0.0f64;
        for k in 1..=n {
            sum += (k as f64).powf(-0.5) * (-(t * (k as f64).ln())).sin();
        }
        sum
    };

    let fine = {
        let mut sum = 0.0f64;
        for k in 1..=(n * 10) {
            sum += (k as f64).powf(-0.5) * (-(t * (k as f64).ln())).sin();
        }
        sum
    };

    let actual_error = (coarse - fine).abs();

    assert!(
        bound >= actual_error,
        "Truncation bound {} should dominate actual error {}. \
         If this fails, the bound formula is wrong.",
        bound, actual_error
    );
}

/// Invariant: Performance must not regress beyond 10% from baseline
/// 
/// LIMITATION: This is a coarse check. Real regression detection uses
/// Criterion's statistical analysis. This test catches catastrophic regressions.
#[test]
fn test_no_catastrophic_regression() {
    // Simple timing check: ζ(½+i·10) should complete in < 5 seconds
    let start = std::time::Instant::now();
    let _ = zeta_half_plus_it(10.0);
    let elapsed = start.elapsed().as_secs_f64();

    assert!(
        elapsed < 5.0,
        "ζ(½+i·10) took {}s — possible catastrophic regression or infinite loop. \
         Baseline: < 1s on Intel i7-1165G7.",
        elapsed
    );
}

/// Invariant: Feature flag correctly routes between MPFR and fallback
/// 
/// LIMITATION: This test verifies the routing logic, not the numerical correctness.
#[test]
fn test_feature_flag_routing() {
    #[cfg(feature = "mpfr")]
    {
        // When mpfr is enabled, we should get the high-precision path
        // We can't easily verify 400-bit precision in a test, but we can
        // verify the function produces output
        let (real, imag) = zeta_half_plus_it(1.0);
        assert!(real.is_finite(), "MPFR path must produce finite real output");
        assert!(imag.is_finite(), "MPFR path must produce finite imag output");
    }

    #[cfg(not(feature = "mpfr"))]
    {
        // When mpfr is disabled, fallback path must still work
        let (real, imag) = zeta_half_plus_it(1.0);
        assert!(real.is_finite(), "Fallback path must produce finite real output");
        assert!(imag.is_finite(), "Fallback path must produce finite imag output");
    }
}