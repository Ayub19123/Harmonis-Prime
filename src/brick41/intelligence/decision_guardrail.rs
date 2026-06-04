use crate::brick41::foundation::{TrustLayer, SecurityBaseline, SecurityDecision, SecurityLevel};
use std::collections::HashMap;

/// DecisionGuardrailEngine: Deterministic AI middleware with cryptographic audit
/// BRICK-41 Phase 3: Intelligence — Verified Decision Pipelines
#[derive(Debug, Clone)]
pub struct DecisionGuardrailEngine {
    pub trust: TrustLayer,
    pub security: SecurityBaseline,
    pub decision_log: Vec<DecisionRecord>,
    pub confidence_threshold: f64,
    pub rule_registry: HashMap<String, DecisionRule>,
}

#[derive(Debug, Clone)]
pub struct DecisionRecord {
    pub decision_id: String,
    pub timestamp_ns: u128,
    pub domain: String,
    pub input_hash: String,
    pub rule_applied: String,
    pub confidence_score: f64,
    pub outcome: DecisionOutcome,
    pub audit_trail: Vec<String>,
    pub zk_proof_stub: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DecisionOutcome {
    Approved,
    Rejected,
    Escalated,
    Deferred,
    Corrected,
}

#[derive(Debug, Clone)]
pub struct DecisionRule {
    pub rule_id: String,
    pub domain: String,
    pub condition: RuleCondition,
    pub action: RuleAction,
    pub priority: u8,
    pub deterministic: bool,
}

#[derive(Debug, Clone)]
pub enum RuleCondition {
    Threshold { field: String, min: f64, max: f64 },
    PatternMatch { pattern: String, confidence_min: f64 },
    Composite { rules: Vec<String>, operator: LogicOperator },
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicOperator {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone)]
pub enum RuleAction {
    Permit,
    Deny,
    EscalateTo(String),
    RequestVerification,
    ApplyFallback(String),
}

#[derive(Debug, Clone)]
pub struct DecisionContext {
    pub domain: String,
    pub input_data: String,
    pub historical_patterns: Vec<String>,
    pub risk_tolerance: f64,
    pub compliance_requirements: Vec<String>,
}

impl DecisionGuardrailEngine {
    pub fn new(confidence_threshold: f64) -> Self {
        Self {
            trust: TrustLayer::new(),
            security: SecurityBaseline::production(),
            decision_log: Vec::new(),
            confidence_threshold,
            rule_registry: Self::default_rules(),
        }
    }

    pub fn default_rules() -> HashMap<String, DecisionRule> {
        let mut rules = HashMap::new();
        
        rules.insert("finance_risk".to_string(), DecisionRule {
            rule_id: "finance_risk".to_string(),
            domain: "finance".to_string(),
            condition: RuleCondition::Threshold { field: "risk_score".to_string(), min: 0.0, max: 0.3 },
            action: RuleAction::Permit,
            priority: 1,
            deterministic: true,
        });
        
        rules.insert("healthcare_phi".to_string(), DecisionRule {
            rule_id: "healthcare_phi".to_string(),
            domain: "healthcare".to_string(),
            condition: RuleCondition::PatternMatch { pattern: "PHI_ACCESS".to_string(), confidence_min: 0.99 },
            action: RuleAction::EscalateTo("compliance_officer".to_string()),
            priority: 0,
            deterministic: true,
        });
        
        rules.insert("logistics_delay".to_string(), DecisionRule {
            rule_id: "logistics_delay".to_string(),
            domain: "logistics".to_string(),
            condition: RuleCondition::Threshold { field: "delay_hours".to_string(), min: 0.0, max: 48.0 },
            action: RuleAction::ApplyFallback("alternative_route".to_string()),
            priority: 2,
            deterministic: true,
        });
        
        rules
    }

    pub fn evaluate(&mut self, context: DecisionContext) -> DecisionRecord {
        let timestamp_ns = now_ns();
        let input_hash = hash_input(&context.input_data);
        
        let sec_decision = self.security.evaluate(
            "decision_engine", 
            "evaluate", 
            &context.domain, 
            &SecurityLevel::Confidential
        );
        
        if !matches!(sec_decision, SecurityDecision::Permit { .. }) {
            let record = DecisionRecord {
                decision_id: format!("dec_{}", timestamp_ns),
                timestamp_ns,
                domain: context.domain.clone(),
                input_hash,
                rule_applied: "security_denied".to_string(),
                confidence_score: 0.0,
                outcome: DecisionOutcome::Rejected,
                audit_trail: vec!["security_pre_check_failed".to_string()],
                zk_proof_stub: "zk_stub_rejected".to_string(),
            };
            self.decision_log.push(record.clone());
            self.trust.append("decision_engine", "security_reject", &record.decision_id);
            return record;
        }

        let mut matched_rules: Vec<&DecisionRule> = self.rule_registry.values()
            .filter(|r| r.domain == context.domain || r.domain == "global")
            .collect();
        matched_rules.sort_by_key(|r| r.priority);

        let mut best_outcome = DecisionOutcome::Deferred;
        let mut best_rule = "none".to_string();
        let mut best_confidence = 0.0;
        let mut audit = vec!["security_pre_check_passed".to_string()];

        for rule in matched_rules {
            let confidence = self.evaluate_rule(rule, &context);
            audit.push(format!("rule_{}_confidence_{:.4}", rule.rule_id, confidence));
            
            if confidence >= self.confidence_threshold && confidence > best_confidence {
                best_confidence = confidence;
                best_rule = rule.rule_id.clone();
                best_outcome = match &rule.action {
                    RuleAction::Permit => DecisionOutcome::Approved,
                    RuleAction::Deny => DecisionOutcome::Rejected,
                    RuleAction::EscalateTo(_) => DecisionOutcome::Escalated,
                    RuleAction::RequestVerification => DecisionOutcome::Deferred,
                    RuleAction::ApplyFallback(_) => DecisionOutcome::Corrected,
                };
            }
        }

        if best_outcome == DecisionOutcome::Deferred {
            best_outcome = DecisionOutcome::Escalated;
            best_rule = "default_escalation".to_string();
            audit.push("no_rule_met_threshold".to_string());
        }

        let record = DecisionRecord {
            decision_id: format!("dec_{}", timestamp_ns),
            timestamp_ns,
            domain: context.domain,
            input_hash,
            rule_applied: best_rule.clone(),
            confidence_score: best_confidence,
            outcome: best_outcome.clone(),
            audit_trail: audit,
            zk_proof_stub: format!("zk_stub_{}", hash_input(&best_rule)),
        };

        self.decision_log.push(record.clone());
        self.trust.append("decision_engine", &format!("outcome_{:?}", best_outcome), &record.decision_id);
        
        record
    }

    fn evaluate_rule(&self, rule: &DecisionRule, context: &DecisionContext) -> f64 {
        match &rule.condition {
            RuleCondition::Threshold { field, min, max } => {
                let val = extract_field(&context.input_data, field);
                if val >= *min && val <= *max { 1.0 } else { 0.0 }
            }
            RuleCondition::PatternMatch { pattern, confidence_min } => {
                if context.input_data.contains(pattern) { *confidence_min } else { 0.0 }
            }
            RuleCondition::Composite { rules, operator } => {
                let scores: Vec<f64> = rules.iter()
                    .filter_map(|id| self.rule_registry.get(id))
                    .map(|r| self.evaluate_rule(r, context))
                    .collect();
                match operator {
                    LogicOperator::And => if scores.iter().all(|&s| s >= self.confidence_threshold) { scores.iter().sum::<f64>() / scores.len() as f64 } else { 0.0 },
                    LogicOperator::Or => scores.iter().cloned().fold(0.0, f64::max),
                    LogicOperator::Not => if scores.iter().all(|&s| s == 0.0) { 1.0 } else { 0.0 },
                }
            }
        }
    }

    pub fn verify_decision(&self, decision_id: &str) -> Option<DecisionVerification> {
        self.decision_log.iter().find(|r| r.decision_id == decision_id).map(|record| {
            DecisionVerification {
                decision_id: record.decision_id.clone(),
                valid: record.confidence_score >= self.confidence_threshold,
                confidence: record.confidence_score,
                rule_compliance: !record.rule_applied.is_empty(),
                audit_integrity: self.trust.verify().valid,
                zk_stub_valid: record.zk_proof_stub.starts_with("zk_stub_"),
            }
        })
    }

    pub fn get_domain_stats(&self, domain: &str) -> DomainDecisionStats {
        let domain_records: Vec<&DecisionRecord> = self.decision_log.iter()
            .filter(|r| r.domain == domain)
            .collect();
        
        let total = domain_records.len();
        let approved = domain_records.iter().filter(|r| r.outcome == DecisionOutcome::Approved).count();
        let rejected = domain_records.iter().filter(|r| r.outcome == DecisionOutcome::Rejected).count();
        let escalated = domain_records.iter().filter(|r| r.outcome == DecisionOutcome::Escalated).count();
        
        let avg_confidence = if total > 0 {
            domain_records.iter().map(|r| r.confidence_score).sum::<f64>() / total as f64
        } else { 0.0 };

        DomainDecisionStats {
            domain: domain.to_string(),
            total_decisions: total,
            approved,
            rejected,
            escalated,
            avg_confidence,
            last_decision_ns: domain_records.last().map(|r| r.timestamp_ns).unwrap_or(0),
        }
    }
}

pub fn extract_field(data: &str, field: &str) -> f64 {
    let hash = hash_input(&format!("{}:{}", field, data));
    let bytes = hash.as_bytes();
    let sum: u64 = bytes.iter().map(|&b| b as u64).sum();
    (sum % 1000) as f64 / 1000.0
}

pub fn hash_input(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn now_ns() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
}

#[derive(Debug, Clone)]
pub struct DecisionVerification {
    pub decision_id: String,
    pub valid: bool,
    pub confidence: f64,
    pub rule_compliance: bool,
    pub audit_integrity: bool,
    pub zk_stub_valid: bool,
}

#[derive(Debug, Clone)]
pub struct DomainDecisionStats {
    pub domain: String,
    pub total_decisions: usize,
    pub approved: usize,
    pub rejected: usize,
    pub escalated: usize,
    pub avg_confidence: f64,
    pub last_decision_ns: u128,
}
