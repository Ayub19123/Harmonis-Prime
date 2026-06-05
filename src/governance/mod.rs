// BRICK-39B: Governance Protocol -- TSG/GDO hardware enforcement
pub mod gdo;
pub mod policy;
pub mod tsg;

pub use tsg::{SafetyAction, SafetyCheck, TrustSafetyGuard};

pub use gdo::{DirectiveOutcome, GlobalDirectiveOptimizer, ResourceAllocation, TaskPriority};

pub use policy::{GovernancePolicy, PolicyEnforcementResult};
