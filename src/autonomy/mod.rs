pub mod goal_engine;
pub mod policy_runtime;
pub mod resource_allocator;
pub mod self_correction;

pub use goal_engine::{AchievementStatus, ActionPlan, Goal, GoalEngine, GoalPriority};
pub use policy_runtime::{
    Policy, PolicyEvaluationResult, PolicyPredicate, PolicyRuntime, PolicySeverity,
    PolicyViolation, ResourceLimits, SafetyBoundary, ViolationAction,
};
pub use resource_allocator::{
    AllocationResult, ComputeAllocation, ResourceAllocator, ResourceBudget, ResourceStats,
};
pub use self_correction::{
    Correction, ErrorRecovery, Feedback, FeedbackLoop, FeedbackStats, FeedbackType,
};
