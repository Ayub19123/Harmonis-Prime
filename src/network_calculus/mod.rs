//! SET-6D/7: Network Calculus — Bounded Latency for Sovereign Intelligence
//!
//! Holy Grail Principle:
//!   Less data. Less energy. More precision. The unresolved becomes mere task
//!   when decoded piece by piece with zero emotion, absolute calm, and
//!   mathematical resilience.
//!
//! Mathematical Foundation:
//!   W(t) = sup_{t >= 0} { inf { tau >= 0 : alpha(t) <= beta(t+tau) } }
//!   
//!   Where alpha(t) = arrival curve (traffic injected)
//!         beta(t)  = service curve (capacity offered)
//!         W(t)     = worst-case delay bound
//!
//! Operating Principle:
//!   - Every failure is data, not defeat
//!   - Every boundary condition is a brick
//!   - Every unresolved puzzle becomes executable test
//!   - Fearless. Calm. Clear. Resilient. Zero emotion.

pub mod curves;

pub use curves::{compute_delay_bound, ArrivalCurve, DelayBound, ServiceCurve};
