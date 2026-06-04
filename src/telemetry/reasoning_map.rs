use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// ReasoningStep: A single cognitive step in the agent's thought process
/// Step = (id, premise, operation, conclusion, confidence, dependencies)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub step_id: u64,
    pub premise: String,
    pub operation: ReasoningOperation,
    pub conclusion: String,
    pub confidence: f64,
    pub dependency_steps: Vec<u64>,
    pub timestamp_nanos: u64,
    pub substrate_source: String,
}

/// ReasoningOperation: The type of cognitive transformation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasoningOperation {
    Deduction,      // If P then Q; P; therefore Q
    Induction,      // Observed pattern → general rule
    Abduction,      // Observed effect → best explanation
    Analogy,        // A is like B; B has property; A likely has property
    Counterfactual, // If X had been different, then Y
    Composition,    // Combine multiple premises
    Decomposition,  // Break premise into components
}

/// ThoughtChain: A linear sequence of reasoning steps
/// Mathematically: Chain = [Step₁, Step₂, ..., Stepₙ] where Stepᵢ₊₁ depends on Stepᵢ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtChain {
    pub chain_id: String,
    pub goal: String,
    pub steps: Vec<ReasoningStep>,
    pub final_confidence: f64,
    pub depth: usize,
    pub completed: bool,
}

/// LongTermReasoningTrace: Persistent reasoning across multiple cycles
/// Tracks how conclusions evolve over time as new evidence arrives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTermReasoningTrace {
    pub trace_id: String,
    pub initial_hypothesis: String,
    pub current_belief: String,
    pub belief_revision_history: Vec<BeliefRevision>,
    pub total_evidence_count: u64,
    pub convergence_score: f64,
}

/// BeliefRevision: A single update to the agent's belief state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefRevision {
    pub revision_id: u64,
    pub previous_belief: String,
    pub new_belief: String,
    pub evidence: String,
    pub confidence_delta: f64,
    pub timestamp_nanos: u64,
}

/// ReasoningMap: The complete visualization of the agent's cognitive topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningMap {
    pub active_chains: VecDeque<ThoughtChain>,
    pub long_term_traces: VecDeque<LongTermReasoningTrace>,
    pub global_step_counter: u64,
    pub max_chains: usize,
    pub max_traces: usize,
}

impl ReasoningStep {
    /// Create new reasoning step
    pub fn new(
        step_id: u64,
        premise: &str,
        operation: ReasoningOperation,
        conclusion: &str,
        confidence: f64,
        dependencies: Vec<u64>,
        substrate: &str,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            step_id,
            premise: premise.to_string(),
            operation,
            conclusion: conclusion.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            dependency_steps: dependencies,
            timestamp_nanos: now,
            substrate_source: substrate.to_string(),
        }
    }
}

impl ThoughtChain {
    /// Create new thought chain for a goal
    pub fn new(chain_id: &str, goal: &str) -> Self {
        Self {
            chain_id: chain_id.to_string(),
            goal: goal.to_string(),
            steps: Vec::new(),
            final_confidence: 0.0,
            depth: 0,
            completed: false,
        }
    }

    /// Append step to chain
    pub fn append_step(&mut self, step: ReasoningStep) {
        self.final_confidence = step.confidence;
        self.depth = self.steps.len() + 1;
        self.steps.push(step);
    }

    /// Mark chain as complete
    pub fn complete(&mut self) {
        self.completed = true;
    }

    /// Get step by ID
    pub fn get_step(&self, step_id: u64) -> Option<&ReasoningStep> {
        self.steps.iter().find(|s| s.step_id == step_id)
    }

    /// Chain statistics
    pub fn stats(&self) -> ChainStats {
        let avg_confidence = if !self.steps.is_empty() {
            self.steps.iter().map(|s| s.confidence).sum::<f64>() / self.steps.len() as f64
        } else {
            0.0
        };

        ChainStats {
            chain_id: self.chain_id.clone(),
            step_count: self.steps.len(),
            avg_confidence,
            max_confidence: self
                .steps
                .iter()
                .map(|s| s.confidence)
                .fold(0.0, |a, b| a.max(b)),
            min_confidence: self
                .steps
                .iter()
                .map(|s| s.confidence)
                .fold(1.0, |a, b| a.min(b)),
            completed: self.completed,
        }
    }
}

impl LongTermReasoningTrace {
    /// Create new long-term trace
    pub fn new(trace_id: &str, hypothesis: &str) -> Self {
        Self {
            trace_id: trace_id.to_string(),
            initial_hypothesis: hypothesis.to_string(),
            current_belief: hypothesis.to_string(),
            belief_revision_history: Vec::new(),
            total_evidence_count: 0,
            convergence_score: 0.0,
        }
    }

    /// Revise belief with new evidence
    pub fn revise(&mut self, evidence: &str, new_belief: &str, confidence: f64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let delta = confidence - self.convergence_score;

        let revision = BeliefRevision {
            revision_id: self.belief_revision_history.len() as u64 + 1,
            previous_belief: self.current_belief.clone(),
            new_belief: new_belief.to_string(),
            evidence: evidence.to_string(),
            confidence_delta: delta,
            timestamp_nanos: now,
        };

        self.current_belief = new_belief.to_string();
        self.belief_revision_history.push(revision);
        self.total_evidence_count += 1;
        self.convergence_score = confidence;
    }

    /// Check if belief has converged (stable over N revisions)
    pub fn has_converged(&self, threshold: f64, window: usize) -> bool {
        if self.belief_revision_history.len() < window {
            return false;
        }

        let recent: Vec<&BeliefRevision> = self
            .belief_revision_history
            .iter()
            .rev()
            .take(window)
            .collect();

        recent.iter().all(|r| r.confidence_delta.abs() < threshold)
    }
}

impl ReasoningMap {
    /// Create new reasoning map
    pub fn new(max_chains: usize, max_traces: usize) -> Self {
        Self {
            active_chains: VecDeque::with_capacity(max_chains),
            long_term_traces: VecDeque::with_capacity(max_traces),
            global_step_counter: 0,
            max_chains,
            max_traces,
        }
    }

    /// Start new thought chain
    pub fn start_chain(&mut self, goal: &str) -> String {
        let chain_id = format!("chain_{}", self.global_step_counter);
        self.global_step_counter += 1;

        let chain = ThoughtChain::new(&chain_id, goal);

        if self.active_chains.len() >= self.max_chains {
            self.active_chains.pop_front();
        }

        self.active_chains.push_back(chain);
        chain_id
    }

    /// Add step to existing chain
    pub fn add_step(&mut self, chain_id: &str, step: ReasoningStep) -> Result<(), String> {
        if let Some(chain) = self
            .active_chains
            .iter_mut()
            .find(|c| c.chain_id == chain_id)
        {
            chain.append_step(step);
            Ok(())
        } else {
            Err(format!("Chain {} not found", chain_id))
        }
    }

    /// Start long-term trace
    pub fn start_trace(&mut self, hypothesis: &str) -> String {
        let trace_id = format!("trace_{}", self.global_step_counter);
        self.global_step_counter += 1;

        let trace = LongTermReasoningTrace::new(&trace_id, hypothesis);

        if self.long_term_traces.len() >= self.max_traces {
            self.long_term_traces.pop_front();
        }

        self.long_term_traces.push_back(trace);
        trace_id
    }

    /// Revise trace belief
    pub fn revise_trace(
        &mut self,
        trace_id: &str,
        evidence: &str,
        new_belief: &str,
        confidence: f64,
    ) -> Result<(), String> {
        if let Some(trace) = self
            .long_term_traces
            .iter_mut()
            .find(|t| t.trace_id == trace_id)
        {
            trace.revise(evidence, new_belief, confidence);
            Ok(())
        } else {
            Err(format!("Trace {} not found", trace_id))
        }
    }

    /// Get complete reasoning topology
    pub fn topology(&self) -> ReasoningTopology {
        ReasoningTopology {
            active_chain_count: self.active_chains.len(),
            long_term_trace_count: self.long_term_traces.len(),
            total_steps: self.active_chains.iter().map(|c| c.steps.len()).sum(),
            converged_traces: self
                .long_term_traces
                .iter()
                .filter(|t| t.has_converged(0.01, 5))
                .count(),
            average_chain_depth: if !self.active_chains.is_empty() {
                self.active_chains.iter().map(|c| c.depth).sum::<usize>() as f64
                    / self.active_chains.len() as f64
            } else {
                0.0
            },
        }
    }
}

/// ChainStats: Statistics for a single thought chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStats {
    pub chain_id: String,
    pub step_count: usize,
    pub avg_confidence: f64,
    pub max_confidence: f64,
    pub min_confidence: f64,
    pub completed: bool,
}

/// ReasoningTopology: Global view of all reasoning activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningTopology {
    pub active_chain_count: usize,
    pub long_term_trace_count: usize,
    pub total_steps: usize,
    pub converged_traces: usize,
    pub average_chain_depth: f64,
}
