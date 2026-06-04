pub mod explainability;
pub mod health_monitor;
pub mod learning_loop;
pub mod reasoning_map;
pub mod telemetry_core;

pub use explainability::{
    CausalAttribution, DecisionJustification, ExplainabilityEngine, IntentExtraction,
};
pub use health_monitor::{HealthMonitor, HealthSnapshot, SubstrateHealth, VitalityScore};
pub use learning_loop::{GradientTrace, KnowledgeDelta, LearningEpoch, LearningLoopTelemetry};
pub use reasoning_map::{LongTermReasoningTrace, ReasoningMap, ReasoningStep, ThoughtChain};
pub use telemetry_core::{AggregationWindow, Event, Metric, TelemetryConfig, TelemetryStream};
