// BRICK-41 Phase 3: Intelligence Layer
// Decision Guardrails + Safe Execution + Reflection + Cross-Domain Learning

pub mod decision_guardrail;
pub mod execution_safety;
pub mod reflection_loop;
pub mod cross_domain_learning;

pub use decision_guardrail::{
    DecisionGuardrailEngine, DecisionRecord, DecisionOutcome, DecisionRule,
    RuleCondition, RuleAction, LogicOperator, DecisionContext, DecisionVerification,
    DomainDecisionStats, extract_field, hash_input, now_ns,
};
pub use execution_safety::{
    ExecutionSafetyEngine, ExecutionContext, ExecutionStage, ExecutionResult,
    Checkpoint, RollbackRecord, SafetyBoundary, BoundaryType, EnforcementMode,
    hash_state, now_ns as safety_now_ns,
};
pub use reflection_loop::{
    ReflectionLoop, OutcomeRecord, PatternInstance, VerificationTask,
    VerifierType, VerificationStatus, LearningDelta, RuleAdjustment,
    DomainLearningStats, MemoryNodeStub, MemoryEdgeStub,
    calculate_match, hash_pattern, now_ns as reflection_now_ns,
};
pub use cross_domain_learning::{
    CrossDomainLearningBridge, TransferRule, TransferResult, TransferOpportunity,
    PatternType, Transformation, FusedKnowledge, DomainSimilarity,
    default_transfer_rules, apply_transformation, cosine_similarity,
    compute_domain_similarity_cached, hash_domain, now_ns as cross_domain_now_ns,
};
