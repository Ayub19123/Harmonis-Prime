//! SET-7B: Zeta Resonance Mapping — Numerical Zero Approximation
//!
//! Holy Grail Principle:
//!   Less data. Less energy. More precision. The unresolved becomes mere task
//!   when decoded piece by piece with zero emotion, absolute calm, and
//!   mathematical resilience.
//!
//! MATHEMATICAL FOUNDATION:
//!   Hardy Z-function: Z(t) = e^{iθ(t)} ζ(1/2 + it)
//!   Gram points: g_n where (-1)^n Z(g_n) > 0
//!   Riemann-Siegel formula for high-precision evaluation
//!
//! CRITICAL LIMITATION — RH DISCIPLINE:
//!   This module computes numerical approximations of zeta zeros.
//!   - High-precision evaluation of ζ(s) on the critical line
//!   - Locates zeros via sign changes of Z(t)
//!   - Maps zero distribution for resonance patterns
//!   - NO claim of proving the Riemann Hypothesis
//!   - NO claim that all zeros lie on Re(s) = 1/2
//!   - We compute where zeros appear, not why they must appear there
//!
//! Operating Principle:
//!   - Every failure is data, not defeat
//!   - Every boundary condition is a brick
//!   - Every unresolved puzzle becomes executable test
//!   - Fearless. Calm. Clear. Resilient. Zero emotion.

pub mod zeta;

pub use zeta::{GramPoint, ZeroLocation, ZetaResonance};

#[cfg(test)]
mod tests;
