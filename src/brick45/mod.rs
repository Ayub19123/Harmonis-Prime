//! BRICK-45: Chaos Engineering & Resilience Validation
//! Deterministic failure injection to prove BRICK-42 sovereignty under punishment

pub mod chaos;

pub use chaos::chaos_engine::{ChaosEngine, ChaosResult, ChaosScenario, InjectionEvent};
pub use chaos::chaos_runner::ChaosRunner;
