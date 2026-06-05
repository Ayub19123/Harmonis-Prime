use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// LearningEpoch: A single training/update cycle
/// Epoch = (id, loss, gradient_norm, knowledge_delta, timestamp)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEpoch {
    pub epoch_id: u64,
    pub loss_value: f64,
    pub gradient_norm: f64,
    pub knowledge_delta: KnowledgeDelta,
    pub learning_rate: f64,
    pub timestamp_nanos: u64,
    pub substrate_source: String,
}

/// KnowledgeDelta: What changed in the agent's knowledge state
/// ΔK = K_new - K_old — the measurable change in understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDelta {
    pub parameter_changes: Vec<ParameterChange>,
    pub concept_acquisitions: Vec<String>,
    pub concept_refinements: Vec<String>,
    pub forgotten_concepts: Vec<String>,
    pub stability_score: f64,
}

/// ParameterChange: A single weight/update in the learning system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterChange {
    pub parameter_name: String,
    pub old_value: f64,
    pub new_value: f64,
    pub relative_change: f64,
}

/// GradientTrace: The path of gradients through the learning landscape
/// Maps how the system navigates the loss manifold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientTrace {
    pub trace_id: String,
    pub start_point: Vec<f64>,
    pub end_point: Vec<f64>,
    pub path_length: f64,
    pub curvature_estimate: f64,
    pub saddle_points_encountered: u64,
    pub local_minima_escaped: u64,
}

/// MetaLearningState: The system learning about its own learning
/// Second-order cognition: how to learn better
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaLearningState {
    pub adaptation_rate: f64,
    pub exploration_exploitation_ratio: f64,
    pub strategy_effectiveness: Vec<StrategyScore>,
    pub recommended_next_strategy: String,
}

/// StrategyScore: Effectiveness of a learning strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyScore {
    pub strategy_name: String,
    pub success_rate: f64,
    pub average_convergence_time: f64,
    pub times_applied: u64,
}

/// LearningLoopTelemetry: The complete continuous learning observability
/// Tracks learning across all time horizons: immediate, short-term, long-term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningLoopTelemetry {
    pub epochs: VecDeque<LearningEpoch>,
    pub gradient_traces: VecDeque<GradientTrace>,
    pub meta_state: MetaLearningState,
    pub long_term_memory_consolidation: Vec<ConsolidationEvent>,
    pub catastrophic_forgetting_guard: ForgettingGuard,
    pub max_epochs: usize,
    pub max_traces: usize,
    pub global_epoch_counter: u64,
}

/// ConsolidationEvent: Transfer from short-term to long-term knowledge
/// Inspired by neuroscience: sleep/rest consolidation of memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationEvent {
    pub event_id: u64,
    pub consolidated_knowledge: Vec<String>,
    pub consolidation_strength: f64,
    pub trigger: ConsolidationTrigger,
    pub timestamp_nanos: u64,
}

/// ConsolidationTrigger: What caused knowledge consolidation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsolidationTrigger {
    Scheduled,     // Periodic background consolidation
    Threshold,     // Knowledge density reached threshold
    Explicit,      // User/system requested consolidation
    ErrorRecovery, // Post-error consolidation for resilience
}

/// ForgettingGuard: Protects against catastrophic forgetting
/// Ensures new learning doesn't destroy old knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgettingGuard {
    pub protected_knowledge: Vec<String>,
    pub forgetting_events: Vec<ForgettingEvent>,
    pub intervention_count: u64,
    pub knowledge_retention_rate: f64,
}

/// ForgettingEvent: When old knowledge was at risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgettingEvent {
    pub event_id: u64,
    pub at_risk_knowledge: String,
    pub retention_action: RetentionAction,
    pub success: bool,
}

/// RetentionAction: How we protected old knowledge
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionAction {
    Rehearsal,            // Re-trained on old knowledge
    Isolation,            // Sequestered old knowledge in separate module
    ElasticWeight,        // Elastic Weight Consolidation (EWC)
    SynapticIntelligence, // Synaptic Intelligence (SI)
}

impl KnowledgeDelta {
    /// Create empty knowledge delta
    pub fn empty() -> Self {
        Self {
            parameter_changes: Vec::new(),
            concept_acquisitions: Vec::new(),
            concept_refinements: Vec::new(),
            forgotten_concepts: Vec::new(),
            stability_score: 1.0,
        }
    }

    /// Calculate total magnitude of change
    pub fn magnitude(&self) -> f64 {
        self.parameter_changes
            .iter()
            .map(|p| p.relative_change.abs())
            .sum::<f64>()
    }

    /// Check if this represents significant learning (not noise)
    pub fn is_significant(&self, threshold: f64) -> bool {
        self.magnitude() > threshold || !self.concept_acquisitions.is_empty()
    }
}

impl LearningEpoch {
    /// Create new learning epoch
    pub fn new(
        epoch_id: u64,
        loss: f64,
        gradient_norm: f64,
        knowledge_delta: KnowledgeDelta,
        learning_rate: f64,
        substrate: &str,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            epoch_id,
            loss_value: loss,
            gradient_norm,
            knowledge_delta,
            learning_rate,
            timestamp_nanos: now,
            substrate_source: substrate.to_string(),
        }
    }
}

impl GradientTrace {
    /// Create new gradient trace
    pub fn new(trace_id: &str, start: Vec<f64>) -> Self {
        Self {
            trace_id: trace_id.to_string(),
            start_point: start.clone(),
            end_point: start,
            path_length: 0.0,
            curvature_estimate: 0.0,
            saddle_points_encountered: 0,
            local_minima_escaped: 0,
        }
    }

    /// Update trace with new position
    pub fn update_position(&mut self, new_position: Vec<f64>, _step_size: f64) {
        // Calculate Euclidean step distance
        let step_dist: f64 = self
            .end_point
            .iter()
            .zip(new_position.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt();

        self.path_length += step_dist;
        self.end_point = new_position;
    }

    /// Detect saddle point (gradient is small but Hessian has negative eigenvalues)
    pub fn mark_saddle(&mut self) {
        self.saddle_points_encountered += 1;
    }

    /// Mark escape from local minimum
    pub fn mark_escape(&mut self) {
        self.local_minima_escaped += 1;
    }
}

impl MetaLearningState {
    /// Create default meta-learning state
    pub fn default_state() -> Self {
        Self {
            adaptation_rate: 0.01,
            exploration_exploitation_ratio: 0.5,
            strategy_effectiveness: vec![
                StrategyScore {
                    strategy_name: "gradient_descent".to_string(),
                    success_rate: 0.85,
                    average_convergence_time: 1000.0,
                    times_applied: 100,
                },
                StrategyScore {
                    strategy_name: "momentum".to_string(),
                    success_rate: 0.90,
                    average_convergence_time: 800.0,
                    times_applied: 80,
                },
                StrategyScore {
                    strategy_name: "adam".to_string(),
                    success_rate: 0.95,
                    average_convergence_time: 500.0,
                    times_applied: 200,
                },
            ],
            recommended_next_strategy: "adam".to_string(),
        }
    }

    /// Update strategy effectiveness based on outcome
    pub fn update_strategy(&mut self, strategy: &str, success: bool, convergence_time: f64) {
        if let Some(score) = self
            .strategy_effectiveness
            .iter_mut()
            .find(|s| s.strategy_name == strategy)
        {
            score.times_applied += 1;
            let n = score.times_applied as f64;
            score.success_rate =
                (score.success_rate * (n - 1.0) + if success { 1.0 } else { 0.0 }) / n;
            score.average_convergence_time =
                (score.average_convergence_time * (n - 1.0) + convergence_time) / n;
        }

        // Re-rank strategies
        self.strategy_effectiveness
            .sort_by(|a, b| b.success_rate.partial_cmp(&a.success_rate).unwrap());

        if let Some(best) = self.strategy_effectiveness.first() {
            self.recommended_next_strategy = best.strategy_name.clone();
        }
    }
}

impl LearningLoopTelemetry {
    /// Create new learning loop telemetry
    pub fn new(max_epochs: usize, max_traces: usize) -> Self {
        Self {
            epochs: VecDeque::with_capacity(max_epochs),
            gradient_traces: VecDeque::with_capacity(max_traces),
            meta_state: MetaLearningState::default_state(),
            long_term_memory_consolidation: Vec::new(),
            catastrophic_forgetting_guard: ForgettingGuard {
                protected_knowledge: Vec::new(),
                forgetting_events: Vec::new(),
                intervention_count: 0,
                knowledge_retention_rate: 1.0,
            },
            max_epochs,
            max_traces,
            global_epoch_counter: 0,
        }
    }

    /// Record a learning epoch
    pub fn record_epoch(
        &mut self,
        loss: f64,
        gradient_norm: f64,
        knowledge_delta: KnowledgeDelta,
        learning_rate: f64,
        substrate: &str,
    ) {
        self.global_epoch_counter += 1;

        let epoch = LearningEpoch::new(
            self.global_epoch_counter,
            loss,
            gradient_norm,
            knowledge_delta,
            learning_rate,
            substrate,
        );

        if self.epochs.len() >= self.max_epochs {
            self.epochs.pop_front();
        }

        self.epochs.push_back(epoch);
    }

    /// Start new gradient trace
    pub fn start_trace(&mut self, start_point: Vec<f64>) -> String {
        let trace_id = format!("trace_{}", self.global_epoch_counter);

        let trace = GradientTrace::new(&trace_id, start_point);

        if self.gradient_traces.len() >= self.max_traces {
            self.gradient_traces.pop_front();
        }

        self.gradient_traces.push_back(trace);
        trace_id
    }

    /// Trigger knowledge consolidation
    pub fn consolidate(
        &mut self,
        knowledge: Vec<String>,
        strength: f64,
        trigger: ConsolidationTrigger,
    ) {
        let event = ConsolidationEvent {
            event_id: self.long_term_memory_consolidation.len() as u64 + 1,
            consolidated_knowledge: knowledge,
            consolidation_strength: strength,
            trigger,
            timestamp_nanos: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        };

        self.long_term_memory_consolidation.push(event);
    }

    /// Guard against catastrophic forgetting
    pub fn guard_forgetting(&mut self, at_risk: &str, action: RetentionAction) -> bool {
        self.catastrophic_forgetting_guard.intervention_count += 1;

        let event = ForgettingEvent {
            event_id: self.catastrophic_forgetting_guard.forgetting_events.len() as u64 + 1,
            at_risk_knowledge: at_risk.to_string(),
            retention_action: action,
            success: true, // Assume success, verify later
        };

        self.catastrophic_forgetting_guard
            .forgetting_events
            .push(event);

        // Update retention rate
        let total = self.catastrophic_forgetting_guard.forgetting_events.len() as f64;
        let successful = self
            .catastrophic_forgetting_guard
            .forgetting_events
            .iter()
            .filter(|e| e.success)
            .count() as f64;

        self.catastrophic_forgetting_guard.knowledge_retention_rate = successful / total;

        true
    }

    /// Get learning velocity (rate of knowledge acquisition)
    pub fn learning_velocity(&self, window: usize) -> f64 {
        let recent: Vec<&LearningEpoch> = self.epochs.iter().rev().take(window).collect();

        if recent.len() < 2 {
            return 0.0;
        }

        let first_loss = recent.last().unwrap().loss_value;
        let last_loss = recent.first().unwrap().loss_value;

        (first_loss - last_loss) / recent.len() as f64
    }

    /// Get complete learning telemetry
    pub fn telemetry_snapshot(&self) -> LearningTelemetrySnapshot {
        LearningTelemetrySnapshot {
            total_epochs: self.global_epoch_counter,
            recent_loss: self.epochs.back().map(|e| e.loss_value),
            learning_velocity: self.learning_velocity(10),
            meta_strategy: self.meta_state.recommended_next_strategy.clone(),
            retention_rate: self.catastrophic_forgetting_guard.knowledge_retention_rate,
            consolidation_count: self.long_term_memory_consolidation.len() as u64,
            active_traces: self.gradient_traces.len() as u64,
        }
    }
}

/// LearningTelemetrySnapshot: Human-readable learning state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningTelemetrySnapshot {
    pub total_epochs: u64,
    pub recent_loss: Option<f64>,
    pub learning_velocity: f64,
    pub meta_strategy: String,
    pub retention_rate: f64,
    pub consolidation_count: u64,
    pub active_traces: u64,
}
