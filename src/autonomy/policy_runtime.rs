use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Policy: An executable safety boundary
/// Mathematically: Policy = State → Bool (predicate over system state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub policy_id: String,
    pub name: String,
    pub predicate: PolicyPredicate,
    pub action_on_violation: ViolationAction,
    pub severity: PolicySeverity,
    pub enabled: bool,
    pub violation_count: u64,
    pub last_violation_nanos: Option<u64>,
}

/// PolicyPredicate: The condition to evaluate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyPredicate {
    MaxComputeUsage(f64),                    // compute_units < threshold
    MaxMemoryUsage(u64),                     // memory_bytes < threshold
    MaxLatencyMicros(f64),                   // latency < threshold
    MaxErrorRate(f64),                       // error_rate < threshold
    MinVitalityScore(f64),                   // vitality > threshold
    ForbiddenAction(String),                 // action not in allowed set
    RequiredContext(Vec<String>),            // context must contain all
    CompositeAnd(Vec<Box<PolicyPredicate>>), // All must pass
    CompositeOr(Vec<Box<PolicyPredicate>>),  // At least one must pass
}

/// ViolationAction: What to do when policy is breached
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ViolationAction {
    Block,    // Prevent the action
    Throttle, // Reduce resource allocation
    Alert,    // Log and notify
    Escalate, // Escalate to higher authority
    LogOnly,  // Record but allow
}

/// PolicySeverity: Classification of policy importance
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PolicySeverity {
    Critical, // System survival depends on this
    High,     // Major impact if violated
    Medium,   // Moderate impact
    Low,      // Minor impact
    Advisory, // Informational only
}

/// Constraint: A bounded condition on execution
/// Constraint = (variable, operator, threshold)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub constraint_id: String,
    pub variable: String,
    pub operator: ConstraintOperator,
    pub threshold: f64,
    pub hard_limit: bool, // true = cannot be violated; false = soft preference
}

/// ConstraintOperator: Comparison operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstraintOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
    InRange(f64, f64), // min, max
}

/// SafetyBoundary: The complete safety envelope
/// All policies and constraints that define the operational perimeter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyBoundary {
    pub boundary_id: String,
    pub name: String,
    pub policies: Vec<Policy>,
    pub constraints: Vec<Constraint>,
    pub emergency_shutdown_threshold: u64, // violations before emergency stop
}

/// PolicyViolation: Record of a policy breach
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub violation_id: u64,
    pub policy_id: String,
    pub timestamp_nanos: u64,
    pub context: String,
    pub attempted_action: String,
    pub actual_value: f64,
    pub threshold_value: f64,
    pub action_taken: ViolationAction,
}

/// PolicyRuntime: The live policy enforcement engine
/// Monad: State → (Action, State') where Action is permitted only if all policies pass
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRuntime {
    pub boundaries: HashMap<String, SafetyBoundary>,
    pub violations: Vec<PolicyViolation>,
    pub global_violation_counter: u64,
    pub emergency_stop_active: bool,
    pub default_boundary: String,
}

/// PolicyEvaluationResult: Outcome of policy check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyEvaluationResult {
    Permitted,
    Denied {
        reason: String,
        action: ViolationAction,
    },
    Throttled {
        new_limits: ResourceLimits,
    },
}

/// ResourceLimits: Throttled resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_compute: f64,
    pub max_memory: u64,
    pub max_latency_micros: f64,
    pub max_concurrent_tasks: usize,
}

impl Policy {
    /// Create new policy
    pub fn new(
        policy_id: &str,
        name: &str,
        predicate: PolicyPredicate,
        action: ViolationAction,
        severity: PolicySeverity,
    ) -> Self {
        Self {
            policy_id: policy_id.to_string(),
            name: name.to_string(),
            predicate,
            action_on_violation: action,
            severity,
            enabled: true,
            violation_count: 0,
            last_violation_nanos: None,
        }
    }

    /// Evaluate policy against current state
    pub fn evaluate(
        &self,
        compute: f64,
        memory: u64,
        latency: f64,
        error_rate: f64,
        vitality: f64,
        action: &str,
        context: &[String],
    ) -> bool {
        if !self.enabled {
            return true; // Disabled policies always pass
        }

        match &self.predicate {
            PolicyPredicate::MaxComputeUsage(threshold) => compute <= *threshold,
            PolicyPredicate::MaxMemoryUsage(threshold) => memory <= *threshold,
            PolicyPredicate::MaxLatencyMicros(threshold) => latency <= *threshold,
            PolicyPredicate::MaxErrorRate(threshold) => error_rate <= *threshold,
            PolicyPredicate::MinVitalityScore(threshold) => vitality >= *threshold,
            PolicyPredicate::ForbiddenAction(forbidden) => action != forbidden,
            PolicyPredicate::RequiredContext(required) => {
                required.iter().all(|r| context.contains(r))
            }
            PolicyPredicate::CompositeAnd(predicates) => predicates.iter().all(|p| {
                // Recursively evaluate composite predicates with same state
                self.evaluate_predicate_box(
                    p, compute, memory, latency, error_rate, vitality, action, context,
                )
            }),
            PolicyPredicate::CompositeOr(predicates) => predicates.iter().any(|p| {
                self.evaluate_predicate_box(
                    p, compute, memory, latency, error_rate, vitality, action, context,
                )
            }),
        }
    }

    /// Helper for boxed predicate evaluation
    fn evaluate_predicate_box(
        &self,
        predicate: &PolicyPredicate,
        compute: f64,
        memory: u64,
        latency: f64,
        error_rate: f64,
        vitality: f64,
        action: &str,
        context: &[String],
    ) -> bool {
        match predicate {
            PolicyPredicate::MaxComputeUsage(threshold) => compute <= *threshold,
            PolicyPredicate::MaxMemoryUsage(threshold) => memory <= *threshold,
            PolicyPredicate::MaxLatencyMicros(threshold) => latency <= *threshold,
            PolicyPredicate::MaxErrorRate(threshold) => error_rate <= *threshold,
            PolicyPredicate::MinVitalityScore(threshold) => vitality >= *threshold,
            PolicyPredicate::ForbiddenAction(forbidden) => action != forbidden,
            PolicyPredicate::RequiredContext(required) => {
                required.iter().all(|r| context.contains(r))
            }
            _ => true, // Nested composites simplified
        }
    }

    /// Record a violation
    pub fn record_violation(&mut self) {
        self.violation_count += 1;
        self.last_violation_nanos = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        );
    }
}

impl Constraint {
    /// Create new constraint
    pub fn new(
        constraint_id: &str,
        variable: &str,
        operator: ConstraintOperator,
        threshold: f64,
        hard: bool,
    ) -> Self {
        Self {
            constraint_id: constraint_id.to_string(),
            variable: variable.to_string(),
            operator,
            threshold,
            hard_limit: hard,
        }
    }

    /// Evaluate constraint against value
    pub fn evaluate(&self, value: f64) -> bool {
        match &self.operator {
            ConstraintOperator::LessThan => value < self.threshold,
            ConstraintOperator::LessThanOrEqual => value <= self.threshold,
            ConstraintOperator::GreaterThan => value > self.threshold,
            ConstraintOperator::GreaterThanOrEqual => value >= self.threshold,
            ConstraintOperator::Equal => (value - self.threshold).abs() < f64::EPSILON,
            ConstraintOperator::NotEqual => (value - self.threshold).abs() >= f64::EPSILON,
            ConstraintOperator::InRange(min, max) => value >= *min && value <= *max,
        }
    }
}

impl SafetyBoundary {
    /// Create new safety boundary
    pub fn new(boundary_id: &str, name: &str) -> Self {
        Self {
            boundary_id: boundary_id.to_string(),
            name: name.to_string(),
            policies: Vec::new(),
            constraints: Vec::new(),
            emergency_shutdown_threshold: 10,
        }
    }

    /// Add policy to boundary
    pub fn add_policy(&mut self, policy: Policy) {
        self.policies.push(policy);
    }

    /// Add constraint to boundary
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Evaluate all policies in this boundary
    pub fn evaluate(
        &self,
        compute: f64,
        memory: u64,
        latency: f64,
        error_rate: f64,
        vitality: f64,
        action: &str,
        context: &[String],
    ) -> Vec<(String, bool, ViolationAction)> {
        self.policies
            .iter()
            .map(|p| {
                let passed = p.evaluate(
                    compute, memory, latency, error_rate, vitality, action, context,
                );
                (p.policy_id.clone(), passed, p.action_on_violation.clone())
            })
            .collect()
    }
}

impl PolicyRuntime {
    /// Create new policy runtime with default TSG/GDO boundaries
    pub fn new() -> Self {
        let mut boundaries = HashMap::new();

        // TSG: Trust & Safety Guard boundary
        let mut tsg = SafetyBoundary::new("tsg", "Trust & Safety Guards");

        tsg.add_policy(Policy::new(
            "tsg_compute_limit",
            "Max Compute Usage",
            PolicyPredicate::MaxComputeUsage(10000.0),
            ViolationAction::Throttle,
            PolicySeverity::High,
        ));

        tsg.add_policy(Policy::new(
            "tsg_memory_limit",
            "Max Memory Usage",
            PolicyPredicate::MaxMemoryUsage(1024 * 1024 * 1024), // 1GB
            ViolationAction::Block,
            PolicySeverity::Critical,
        ));

        tsg.add_policy(Policy::new(
            "tsg_latency_limit",
            "Max Latency",
            PolicyPredicate::MaxLatencyMicros(10000.0),
            ViolationAction::Alert,
            PolicySeverity::Medium,
        ));

        tsg.add_policy(Policy::new(
            "tsg_error_rate",
            "Max Error Rate",
            PolicyPredicate::MaxErrorRate(0.05),
            ViolationAction::Escalate,
            PolicySeverity::High,
        ));

        tsg.add_policy(Policy::new(
            "tsg_vitality_floor",
            "Min Vitality Score",
            PolicyPredicate::MinVitalityScore(0.3),
            ViolationAction::Escalate,
            PolicySeverity::Critical,
        ));

        // GDO: Governance-Driven Optimization boundary
        let mut gdo = SafetyBoundary::new("gdo", "Governance-Driven Optimization");

        gdo.add_policy(Policy::new(
            "gdo_forbidden_actions",
            "Forbidden Actions",
            PolicyPredicate::ForbiddenAction("system_shutdown".to_string()),
            ViolationAction::Block,
            PolicySeverity::Critical,
        ));

        gdo.add_constraint(Constraint::new(
            "gdo_task_concurrency",
            "concurrent_tasks",
            ConstraintOperator::LessThanOrEqual,
            100.0,
            true,
        ));

        boundaries.insert("tsg".to_string(), tsg);
        boundaries.insert("gdo".to_string(), gdo);

        Self {
            boundaries,
            violations: Vec::new(),
            global_violation_counter: 0,
            emergency_stop_active: false,
            default_boundary: "tsg".to_string(),
        }
    }

    /// Evaluate action against all boundaries
    pub fn evaluate_action(
        &mut self,
        action: &str,
        compute: f64,
        memory: u64,
        latency: f64,
        error_rate: f64,
        vitality: f64,
        context: &[String],
    ) -> PolicyEvaluationResult {
        if self.emergency_stop_active {
            return PolicyEvaluationResult::Denied {
                reason: "EMERGENCY STOP ACTIVE".to_string(),
                action: ViolationAction::Block,
            };
        }

        let mut total_violations = 0u64;
        let mut most_severe_action = ViolationAction::LogOnly;
        let mut throttle_limits = ResourceLimits {
            max_compute: f64::INFINITY,
            max_memory: u64::MAX,
            max_latency_micros: f64::INFINITY,
            max_concurrent_tasks: usize::MAX,
        };

        for (boundary_name, boundary) in &self.boundaries {
            let results = boundary.evaluate(
                compute, memory, latency, error_rate, vitality, action, context,
            );

            for (policy_id, passed, violation_action) in results {
                if !passed {
                    total_violations += 1;

                    // Record violation
                    self.global_violation_counter += 1;
                    let violation = PolicyViolation {
                        violation_id: self.global_violation_counter,
                        policy_id: policy_id.clone(),
                        timestamp_nanos: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos() as u64,
                        context: format!("boundary: {}", boundary_name),
                        attempted_action: action.to_string(),
                        actual_value: compute, // Simplified
                        threshold_value: 0.0,  // Would need policy-specific lookup
                        action_taken: violation_action.clone(),
                    };
                    self.violations.push(violation);

                    // Track most severe action
                    if violation_action == ViolationAction::Block {
                        most_severe_action = ViolationAction::Block;
                    } else if violation_action == ViolationAction::Escalate
                        && most_severe_action != ViolationAction::Block
                    {
                        most_severe_action = ViolationAction::Escalate;
                    } else if violation_action == ViolationAction::Throttle
                        && most_severe_action == ViolationAction::LogOnly
                    {
                        most_severe_action = ViolationAction::Throttle;
                        throttle_limits.max_compute =
                            throttle_limits.max_compute.min(compute * 0.5);
                    }
                }
            }
        }

        // Check emergency shutdown threshold
        if total_violations >= self.boundaries[&self.default_boundary].emergency_shutdown_threshold
        {
            self.emergency_stop_active = true;
            return PolicyEvaluationResult::Denied {
                reason: "EMERGENCY SHUTDOWN TRIGGERED".to_string(),
                action: ViolationAction::Block,
            };
        }

        match most_severe_action {
            ViolationAction::Block => PolicyEvaluationResult::Denied {
                reason: format!("{} policy violations detected", total_violations),
                action: ViolationAction::Block,
            },
            ViolationAction::Throttle => PolicyEvaluationResult::Throttled {
                new_limits: throttle_limits,
            },
            _ => PolicyEvaluationResult::Permitted,
        }
    }

    /// Check if action is permitted (read-only)
    pub fn is_permitted(
        &self,
        action: &str,
        compute: f64,
        memory: u64,
        latency: f64,
        error_rate: f64,
        vitality: f64,
        context: &[String],
    ) -> bool {
        if self.emergency_stop_active {
            return false;
        }

        for (_, boundary) in &self.boundaries {
            let results = boundary.evaluate(
                compute, memory, latency, error_rate, vitality, action, context,
            );
            if results.iter().any(|(_, passed, _)| !passed) {
                return false;
            }
        }

        true
    }

    /// Reset emergency stop
    pub fn reset_emergency(&mut self) {
        self.emergency_stop_active = false;
        self.violations.clear();
    }

    /// Policy runtime statistics
    pub fn stats(&self) -> PolicyRuntimeStats {
        let total_policies: usize = self.boundaries.values().map(|b| b.policies.len()).sum();
        let total_constraints: usize = self.boundaries.values().map(|b| b.constraints.len()).sum();

        PolicyRuntimeStats {
            total_boundaries: self.boundaries.len() as u64,
            total_policies: total_policies as u64,
            total_constraints: total_constraints as u64,
            total_violations: self.global_violation_counter,
            emergency_stop_active: self.emergency_stop_active,
            violation_rate: if !self.violations.is_empty() {
                self.global_violation_counter as f64 / self.violations.len().max(1) as f64
            } else {
                0.0
            },
        }
    }
}

/// PolicyRuntimeStats: Observability morphism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRuntimeStats {
    pub total_boundaries: u64,
    pub total_policies: u64,
    pub total_constraints: u64,
    pub total_violations: u64,
    pub emergency_stop_active: bool,
    pub violation_rate: f64,
}
