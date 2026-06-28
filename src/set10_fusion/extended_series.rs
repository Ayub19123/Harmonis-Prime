//! Extended truncated Dirichlet series with more terms.
//!
//! Honest Limitation: sigma > 1 only. Diverges at sigma = 1/2.
//! No analytic continuation. No Riemann-Siegel formula.

/// Extended Dirichlet series: ζ(s) ≈ Σ_{n=1}^{N} n^{-s}
#[derive(Debug, Clone)]
pub struct ExtendedDirichletSeries {
    max_terms: usize,
}

impl ExtendedDirichletSeries {
    pub fn new(max_terms: usize) -> Result<Self, &'static str> {
        if max_terms < 2 {
            return Err("max_terms must be at least 2");
        }
        Ok(Self { max_terms })
    }

    /// Evaluate truncated series for s = sigma + i·t
    /// Returns (real, imag) parts
    /// Honest: only valid for sigma > 1.0
    pub fn evaluate(&self, sigma: f64, t: f64) -> Result<(f64, f64), &'static str> {
        if sigma <= 1.0 {
            return Err("sigma must be > 1.0 for convergence — honest limitation");
        }
        if !sigma.is_finite() || !t.is_finite() {
            return Err("sigma and t must be finite");
        }

        let mut real = 0.0;
        let mut imag = 0.0;

        for n in 1..=self.max_terms {
            let n_f = n as f64;
            let n_sigma = n_f.powf(-sigma);
            let angle = -t * n_f.ln();
            real += n_sigma * angle.cos();
            imag += n_sigma * angle.sin();
        }

        Ok((real, imag))
    }

    /// Return number of terms
    pub fn terms(&self) -> usize {
        self.max_terms
    }

    /// Estimate truncation error bound: integral from N+1 to ∞ of x^{-sigma} dx
    /// = (N+1)^{1-sigma} / (sigma - 1)
    pub fn error_bound(&self, sigma: f64) -> Result<f64, &'static str> {
        if sigma <= 1.0 {
            return Err("sigma must be > 1.0");
        }
        let n_plus_1 = (self.max_terms + 1) as f64;
        let bound = n_plus_1.powf(1.0 - sigma) / (sigma - 1.0);
        Ok(bound)
    }
}
