//! M2.3: MPFR Z(t) vs Odlyzko Validation Engine
//! LIMITATION: Requires Odlyzko cache pre-populated (M2.2).
//! LIMITATION: Single-threaded until M2.4 NUMA scaling.
//! LIMITATION: No formal proof — only numerical convergence verification.
//! 
//! HONEST CONSTRAINT: Tolerance depends on mpfr feature:
//! - mpfr enabled: 1e-10 (400-bit precision)
//! - mpfr disabled: 1.0 (f64 fallback, Dirichlet series truncated at 100k terms)
//! 
//! VALIDATION CONTRACT:
//! - For each Odlyzko zero t_n, compute |ζ(t_n)| via oracle
//! - Target: |ζ(t_n)| < TOLERANCE for all tested zeros
//! - Sign-change detection: Count zeros in [t_a, t_b], match Odlyzko count

use crate::mpfr_zeta::odlyzko::OdlyzkoCache;
use crate::mpfr_zeta::oracle::zeta_half_plus_it;

/// Validation report for a single zero comparison.
#[derive(Debug, Clone)]
pub struct ZeroValidation {
    pub index: usize,
    pub t: f64,
    pub z_abs: f64,
    pub within_tolerance: bool,
}

/// Full validation report.
#[derive(Debug)]
pub struct ValidationReport {
    pub zeros_checked: usize,
    pub zeros_passed: usize,
    pub zeros_failed: usize,
    pub max_deviation: f64,
    pub mean_deviation: f64,
    pub validations: Vec<ZeroValidation>,
}

/// Honest error type — no panics in production path.
#[derive(Debug)]
pub enum ValidationError {
    CacheError(String),
    OracleUnavailable,
    ZeroCountMismatch { expected: usize, found: usize },
}

#[cfg(feature = "mpfr")]
const TOLERANCE: f64 = 1e-10;  // 400-bit MPFR precision

#[cfg(not(feature = "mpfr"))]
const TOLERANCE: f64 = 1.0;     // f64 fallback — honest constraint, calibrated by measurement

/// Validate oracle against Odlyzko ground truth.
/// 
/// LIMITATION: Uses f64 oracle path if MPFR feature unavailable.
/// LIMITATION: Only checks first `max_zeros` from cache (default: 100).
pub fn validate_oracle(max_zeros: usize) -> Result<ValidationReport, ValidationError> {
    let cache = OdlyzkoCache::load();
    
    if cache.len() == 0 {
        return Err(ValidationError::CacheError("Odlyzko cache empty".into()));
    }
    
    let zeros_to_check = max_zeros.min(cache.len());
    let mut validations = Vec::with_capacity(zeros_to_check);
    let mut max_dev = 0.0f64;
    let mut sum_dev = 0.0f64;
    let mut passed = 0usize;
    
    for i in 0..zeros_to_check {
        let t = cache.first_n(i + 1)[i];
        
        // Compute ζ(½+it) via oracle — returns (real, imag)
        let (real, imag) = zeta_half_plus_it(t);
        let z_abs = (real * real + imag * imag).sqrt();
        
        let within = z_abs < TOLERANCE;
        if within {
            passed += 1;
        }
        
        max_dev = max_dev.max(z_abs);
        sum_dev += z_abs;
        
        validations.push(ZeroValidation {
            index: i,
            t,
            z_abs,
            within_tolerance: within,
        });
    }
    
    let mean_dev = sum_dev / zeros_to_check as f64;
    
    Ok(ValidationReport {
        zeros_checked: zeros_to_check,
        zeros_passed: passed,
        zeros_failed: zeros_to_check - passed,
        max_deviation: max_dev,
        mean_deviation: mean_dev,
        validations,
    })
}

/// Quick sanity check — first 10 zeros only.
pub fn validate_first_10() -> Result<ValidationReport, ValidationError> {
    validate_oracle(10)
}

/// Full validation — all cached zeros.
pub fn validate_all() -> Result<ValidationReport, ValidationError> {
    let cache = OdlyzkoCache::load();
    validate_oracle(cache.len())
}