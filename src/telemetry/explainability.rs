use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// DecisionJustification: Why a decision was made
/// Justification = (decision, reasoning_chain, confidence, alternative_considered)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionJustification {
    pub decision_id: String,
    pub decision: String,
    pub reasoning_chain: Vec<ReasoningLink>,
    pub confidence: f64,
    pub alternatives_considered: Vec<Alternative>,
    pub timestamp_nanos: u64,
    pub substrate_source: String,
}

/// ReasoningLink: A single step in the justification chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningLink {
    pub step_number: usize,
    pub premise: String,
    pub inference_rule: String,
    pub conclusion: String,
    pub confidence: f64,
}

/// Alternative: A path not taken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub alternative_id: String,
    pub description: String,
    pub expected_outcome: String,
    pub why_rejected: String,
    pub rejection_confidence: f64,
}

/// IntentExtraction: What the system intended to achieve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentExtraction {
    pub intent_id: String,
    pub stated_goal: String,
    pub inferred_goal: String,
    pub goal_alignment_score: f64,
    pub sub_intents: Vec<String>,
    pub completion_criteria: Vec<String>,
}

/// CausalAttribution: What caused what
/// Maps cause → effect with confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalAttribution {
    pub attribution_id: String,
    pub cause: String,
    pub effect: String,
    pub causal_strength: f64,
    pub evidence: Vec<String>,
    pub counterfactual: String,
}

/// ExplainabilityEngine: The transparency core
/// Makes every decision explainable, every intent visible, every cause traceable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainabilityEngine {
    pub justifications: VecDeque<DecisionJustification>,
    pub intents: VecDeque<IntentExtraction>,
    pub attributions: VecDeque<CausalAttribution>,
    pub max_justifications: usize,
    pub max_intents: usize,
    pub max_attributions: usize,
    pub global_justification_counter: u64,
}

/// ExplanationRequest: What a human operator wants to know
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationRequest {
    pub request_id: String,
    pub query_type: ExplanationQuery,
    pub target_decision: Option<String>,
    pub time_range_start: Option<u64>,
    pub time_range_end: Option<u64>,
}

/// ExplanationQuery: Types of explainability questions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExplanationQuery {
    WhyDecision,     // Why was this decision made?
    WhatIntent,      // What was the system's intent?
    WhatAlternative, // What alternatives were considered?
    WhatCause,       // What caused this outcome?
    HowConfident,    // How confident was the system?
    FullTrace,       // Give me the complete reasoning trace
}

/// ExplanationResponse: Human-readable explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationResponse {
    pub request_id: String,
    pub query_type: ExplanationQuery,
    pub summary: String,
    pub detailed_explanation: String,
    pub confidence: f64,
    pub supporting_evidence: Vec<String>,
    pub related_decisions: Vec<String>,
}

impl ReasoningLink {
    /// Create new reasoning link
    pub fn new(step: usize, premise: &str, rule: &str, conclusion: &str, confidence: f64) -> Self {
        Self {
            step_number: step,
            premise: premise.to_string(),
            inference_rule: rule.to_string(),
            conclusion: conclusion.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

impl Alternative {
    /// Create new alternative
    pub fn new(
        id: &str,
        description: &str,
        outcome: &str,
        rejected: &str,
        confidence: f64,
    ) -> Self {
        Self {
            alternative_id: id.to_string(),
            description: description.to_string(),
            expected_outcome: outcome.to_string(),
            why_rejected: rejected.to_string(),
            rejection_confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

impl DecisionJustification {
    /// Create new justification
    pub fn new(decision_id: &str, decision: &str, substrate: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            decision_id: decision_id.to_string(),
            decision: decision.to_string(),
            reasoning_chain: Vec::new(),
            confidence: 0.0,
            alternatives_considered: Vec::new(),
            timestamp_nanos: now,
            substrate_source: substrate.to_string(),
        }
    }

    /// Add reasoning step
    pub fn add_step(&mut self, premise: &str, rule: &str, conclusion: &str, confidence: f64) {
        let step = ReasoningLink::new(
            self.reasoning_chain.len() + 1,
            premise,
            rule,
            conclusion,
            confidence,
        );
        self.reasoning_chain.push(step);
        self.confidence = self.reasoning_chain.iter().map(|r| r.confidence).product();
    }

    /// Add alternative
    pub fn add_alternative(
        &mut self,
        id: &str,
        description: &str,
        outcome: &str,
        rejected: &str,
        confidence: f64,
    ) {
        self.alternatives_considered.push(Alternative::new(
            id,
            description,
            outcome,
            rejected,
            confidence,
        ));
    }

    /// Human-readable summary
    pub fn summarize(&self) -> String {
        format!(
            "Decision '{}' (confidence: {:.2}%): {} reasoning steps, {} alternatives considered",
            self.decision_id,
            self.confidence * 100.0,
            self.reasoning_chain.len(),
            self.alternatives_considered.len()
        )
    }
}

impl IntentExtraction {
    /// Create new intent
    pub fn new(intent_id: &str, stated: &str, inferred: &str) -> Self {
        Self {
            intent_id: intent_id.to_string(),
            stated_goal: stated.to_string(),
            inferred_goal: inferred.to_string(),
            goal_alignment_score: 0.0,
            sub_intents: Vec::new(),
            completion_criteria: Vec::new(),
        }
    }

    /// Check if intent is aligned (stated matches inferred)
    pub fn is_aligned(&self, threshold: f64) -> bool {
        self.goal_alignment_score >= threshold
    }

    /// Human-readable intent summary
    pub fn summarize(&self) -> String {
        format!(
            "Intent '{}' | Stated: '{}' | Inferred: '{}' | Alignment: {:.2}% | Sub-intents: {}",
            self.intent_id,
            self.stated_goal,
            self.inferred_goal,
            self.goal_alignment_score * 100.0,
            self.sub_intents.len()
        )
    }
}

impl CausalAttribution {
    /// Create new causal attribution
    pub fn new(id: &str, cause: &str, effect: &str, strength: f64) -> Self {
        Self {
            attribution_id: id.to_string(),
            cause: cause.to_string(),
            effect: effect.to_string(),
            causal_strength: strength.clamp(0.0, 1.0),
            evidence: Vec::new(),
            counterfactual: String::new(),
        }
    }

    /// Human-readable causal summary
    pub fn summarize(&self) -> String {
        format!(
            "Cause '{}' → Effect '{}' (strength: {:.2}%) | Evidence: {} items",
            self.cause,
            self.effect,
            self.causal_strength * 100.0,
            self.evidence.len()
        )
    }
}

impl ExplainabilityEngine {
    /// Create new explainability engine
    pub fn new(max_justifications: usize, max_intents: usize, max_attributions: usize) -> Self {
        Self {
            justifications: VecDeque::with_capacity(max_justifications),
            intents: VecDeque::with_capacity(max_intents),
            attributions: VecDeque::with_capacity(max_attributions),
            max_justifications,
            max_intents,
            max_attributions,
            global_justification_counter: 0,
        }
    }

    /// Record a decision justification
    pub fn justify(&mut self, justification: DecisionJustification) {
        self.global_justification_counter += 1;

        if self.justifications.len() >= self.max_justifications {
            self.justifications.pop_front();
        }

        self.justifications.push_back(justification);
    }

    /// Record an intent
    pub fn record_intent(&mut self, intent: IntentExtraction) {
        if self.intents.len() >= self.max_intents {
            self.intents.pop_front();
        }

        self.intents.push_back(intent);
    }

    /// Record a causal attribution
    pub fn attribute(&mut self, attribution: CausalAttribution) {
        if self.attributions.len() >= self.max_attributions {
            self.attributions.pop_front();
        }

        self.attributions.push_back(attribution);
    }

    /// Explain a specific decision
    pub fn explain_decision(&self, decision_id: &str) -> Option<ExplanationResponse> {
        let justification = self
            .justifications
            .iter()
            .find(|j| j.decision_id == decision_id)?;

        let mut explanation = format!("Decision: {}\n\n", justification.decision);
        explanation.push_str("Reasoning Chain:\n");

        for link in &justification.reasoning_chain {
            explanation.push_str(&format!(
                "  Step {}: {} [{}] → {} (confidence: {:.2}%)\n",
                link.step_number,
                link.premise,
                link.inference_rule,
                link.conclusion,
                link.confidence * 100.0
            ));
        }

        if !justification.alternatives_considered.is_empty() {
            explanation.push_str("\nAlternatives Considered:\n");
            for alt in &justification.alternatives_considered {
                explanation.push_str(&format!(
                    "  - {}: {} (rejected because: {})\n",
                    alt.alternative_id, alt.description, alt.why_rejected
                ));
            }
        }

        Some(ExplanationResponse {
            request_id: format!("explain_{}", decision_id),
            query_type: ExplanationQuery::WhyDecision,
            summary: justification.summarize(),
            detailed_explanation: explanation,
            confidence: justification.confidence,
            supporting_evidence: justification
                .reasoning_chain
                .iter()
                .map(|r| r.conclusion.clone())
                .collect(),
            related_decisions: Vec::new(),
        })
    }

    /// Explain system intent
    pub fn explain_intent(&self, intent_id: &str) -> Option<ExplanationResponse> {
        let intent = self.intents.iter().find(|i| i.intent_id == intent_id)?;

        Some(ExplanationResponse {
            request_id: format!("intent_{}", intent_id),
            query_type: ExplanationQuery::WhatIntent,
            summary: intent.summarize(),
            detailed_explanation: format!(
                "Stated Goal: {}\nInferred Goal: {}\nAlignment: {:.2}%\nSub-intents: {}",
                intent.stated_goal,
                intent.inferred_goal,
                intent.goal_alignment_score * 100.0,
                intent.sub_intents.join(", ")
            ),
            confidence: intent.goal_alignment_score,
            supporting_evidence: intent.completion_criteria.clone(),
            related_decisions: Vec::new(),
        })
    }

    /// Explain causal chain
    pub fn explain_cause(&self, attribution_id: &str) -> Option<ExplanationResponse> {
        let attr = self
            .attributions
            .iter()
            .find(|a| a.attribution_id == attribution_id)?;

        Some(ExplanationResponse {
            request_id: format!("cause_{}", attribution_id),
            query_type: ExplanationQuery::WhatCause,
            summary: attr.summarize(),
            detailed_explanation: format!(
                "Cause: {}\nEffect: {}\nCausal Strength: {:.2}%\nCounterfactual: {}\nEvidence: {}",
                attr.cause,
                attr.effect,
                attr.causal_strength * 100.0,
                attr.counterfactual,
                attr.evidence.join(", ")
            ),
            confidence: attr.causal_strength,
            supporting_evidence: attr.evidence.clone(),
            related_decisions: Vec::new(),
        })
    }

    /// Full explainability snapshot
    pub fn full_explanation(&self) -> FullExplanation {
        FullExplanation {
            total_justifications: self.justifications.len() as u64,
            total_intents: self.intents.len() as u64,
            total_attributions: self.attributions.len() as u64,
            recent_justifications: self.justifications.iter().rev().take(5).cloned().collect(),
            recent_intents: self.intents.iter().rev().take(5).cloned().collect(),
            recent_attributions: self.attributions.iter().rev().take(5).cloned().collect(),
            explainability_score: self.compute_explainability_score(),
        }
    }

    /// Compute how explainable the system currently is
    fn compute_explainability_score(&self) -> f64 {
        let j_score = if !self.justifications.is_empty() {
            self.justifications
                .iter()
                .map(|j| j.confidence)
                .sum::<f64>()
                / self.justifications.len() as f64
        } else {
            0.0
        };

        let i_score = if !self.intents.is_empty() {
            self.intents
                .iter()
                .map(|i| i.goal_alignment_score)
                .sum::<f64>()
                / self.intents.len() as f64
        } else {
            0.0
        };

        let a_score = if !self.attributions.is_empty() {
            self.attributions
                .iter()
                .map(|a| a.causal_strength)
                .sum::<f64>()
                / self.attributions.len() as f64
        } else {
            0.0
        };

        (j_score * 0.4 + i_score * 0.3 + a_score * 0.3).clamp(0.0, 1.0)
    }

    /// Get explainability statistics
    pub fn stats(&self) -> ExplainabilityStats {
        ExplainabilityStats {
            total_justifications: self.global_justification_counter,
            buffered_justifications: self.justifications.len() as u64,
            buffered_intents: self.intents.len() as u64,
            buffered_attributions: self.attributions.len() as u64,
            explainability_score: self.compute_explainability_score(),
        }
    }
}

/// FullExplanation: Complete explainability state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullExplanation {
    pub total_justifications: u64,
    pub total_intents: u64,
    pub total_attributions: u64,
    pub recent_justifications: Vec<DecisionJustification>,
    pub recent_intents: Vec<IntentExtraction>,
    pub recent_attributions: Vec<CausalAttribution>,
    pub explainability_score: f64,
}

/// ExplainabilityStats: Observability morphism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainabilityStats {
    pub total_justifications: u64,
    pub buffered_justifications: u64,
    pub buffered_intents: u64,
    pub buffered_attributions: u64,
    pub explainability_score: f64,
}
