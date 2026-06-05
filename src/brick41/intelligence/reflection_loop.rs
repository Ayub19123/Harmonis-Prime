use crate::brick41::foundation::{TrustLayer, MemoryStore};
use crate::brick41::intelligence::{DecisionRecord, DecisionOutcome, ExecutionResult, ExecutionStage};
use std::collections::HashMap;

/// ReflectionLoop: Outcome verification, cross-domain intelligence transfer
/// BRICK-41 Phase 3: Intelligence — Continuous Feedback & Learning
#[derive(Debug, Clone)]
pub struct ReflectionLoop {
    pub trust: TrustLayer,
    pub memory: MemoryStore,
    pub outcome_log: Vec<OutcomeRecord>,
    pub pattern_index: HashMap<String, Vec<PatternInstance>,
    pub learning_rate: f64,
    pub verification_queue: Vec<VerificationTask>,
}

#[derive(Debug, Clone)]
pub struct OutcomeRecord {
    pub outcome_id: String,
    pub timestamp_ns: u128,
    pub domain: String,
    pub decision_id: String,
    pub execution_id: String,
    pub expected_outcome: String,
    pub actual_outcome: String,
    pub match_score: f64,
    pub verified: bool,
    pub lessons: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PatternInstance {
    pub pattern_id: String,
    pub domain: String,
    pub trigger_conditions: Vec<String>,
    pub success_rate: f64,
    pub occurrence_count: u32,
    pub last_seen_ns: u128,
}

#[derive(Debug, Clone)]
pub struct VerificationTask {
    pub task_id: String,
    pub outcome_id: String,
    pub verifier_type: VerifierType,
    pub status: VerificationStatus,
    pub scheduled_ns: u128,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerifierType {
    Automated,
    HumanReview,
    CrossDomainAgent,
    ExternalOracle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerificationStatus {
    Pending,
    InProgress,
    Approved,
    Rejected,
    Escalated,
}

#[derive(Debug, Clone)]
pub struct LearningDelta {
    pub domain: String,
    pub rule_adjustments: Vec<RuleAdjustment>,
    pub memory_additions: Vec<MemoryNodeStub>,
    pub memory_links: Vec<MemoryEdgeStub>,
    pub confidence_delta: f64,
}

#[derive(Debug, Clone)]
pub struct MemoryNodeStub {
    pub id: String,
    pub domain: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub confidence: f64,
    pub timestamp_ns: u128,
}

#[derive(Debug, Clone)]
pub struct MemoryEdgeStub {
    pub source_id: String,
    pub target_id: String,
    pub relation: String,
    pub weight: f64,
    pub bidirectional: bool,
}

#[derive(Debug, Clone)]
pub struct RuleAdjustment {
    pub rule_id: String,
    pub field: String,
    pub old_value: String,
    pub new_value: String,
    pub reason: String,
}

impl ReflectionLoop {
    pub fn new(learning_rate: f64) -> Self {
        Self {
            trust: TrustLayer::new(),
            memory: MemoryStore::new(),
            outcome_log: Vec::new(),
            pattern_index: HashMap::new(),
            learning_rate,
            verification_queue: Vec::new(),
        }
    }

    pub fn record_outcome(&mut self, domain: &str, decision_id: &str, execution_id: &str, expected: &str, actual: &str) -> OutcomeRecord {
        let timestamp_ns = now_ns();
        let match_score = calculate_match(expected, actual);
        let verified = match_score > 0.95;

        let record = OutcomeRecord {
            outcome_id: format!("out_{}", timestamp_ns),
            timestamp_ns,
            domain: domain.to_string(),
            decision_id: decision_id.to_string(),
            execution_id: execution_id.to_string(),
            expected_outcome: expected.to_string(),
            actual_outcome: actual.to_string(),
            match_score,
            verified,
            lessons: Vec::new(),
        };

        self.outcome_log.push(record.clone());
        self.trust.append("reflection_loop", "outcome_recorded", &record.outcome_id);

        if verified {
            self.trust.append("reflection_loop", "auto_verified", &record.outcome_id);
        } else {
            self.verification_queue.push(VerificationTask {
                task_id: format!("ver_{}", timestamp_ns),
                outcome_id: record.outcome_id.clone(),
                verifier_type: VerifierType::Automated,
                status: VerificationStatus::Pending,
                scheduled_ns: timestamp_ns,
            });
        }

        record
    }

    pub fn verify_outcome(&mut self, outcome_id: &str, verifier: VerifierType, approved: bool) -> Option<VerificationTask> {
        let task = self.verification_queue.iter_mut().find(|t| t.outcome_id == outcome_id)?;
        task.verifier_type = verifier;
        task.status = if approved { VerificationStatus::Approved } else { VerificationStatus::Rejected };
        
        if let Some(record) = self.outcome_log.iter_mut().find(|r| r.outcome_id == outcome_id) {
            record.verified = approved;
            if !approved {
                record.lessons.push(format!("rejected_by_{:?}", verifier));
            }
        }

        self.trust.append("reflection_loop", &format!("verified_{}", if approved { "approved" } else { "rejected" }), outcome_id);
        Some(task.clone())
    }

    pub fn extract_lessons(&mut self, outcome_id: &str) -> Vec<String> {
        let record = match self.outcome_log.iter().find(|r| r.outcome_id == outcome_id) {
            Some(r) => r.clone(),
            None => return vec![],
        };

        let mut lessons = Vec::new();

        if record.match_score < 0.5 {
            lessons.push(format!("critical_mismatch: expected '{}' got '{}'", record.expected_outcome, record.actual_outcome));
        } else if record.match_score < 0.95 {
            lessons.push(format!("partial_mismatch: score {:.2}", record.match_score));
        }

        if !record.verified {
            lessons.push("verification_required: outcome not automatically trusted".to_string());
        }

        let pattern_key = format!("{}_{}", record.domain, hash_pattern(&record.expected_outcome));
        let pattern = self.pattern_index.entry(pattern_key.clone()).or_insert_with(Vec::new);
        
        let existing = pattern.iter_mut().find(|p| p.pattern_id == pattern_key);
        if let Some(p) = existing {
            p.occurrence_count += 1;
            p.last_seen_ns = record.timestamp_ns;
            p.success_rate = (p.success_rate * (p.occurrence_count as f64 - 1.0) + record.match_score) / p.occurrence_count as f64;
        } else {
            pattern.push(PatternInstance {
                pattern_id: pattern_key.clone(),
                domain: record.domain.clone(),
                trigger_conditions: vec![record.expected_outcome.clone()],
                success_rate: record.match_score,
                occurrence_count: 1,
                last_seen_ns: record.timestamp_ns,
            });
        }

        self.memory.store(
            &record.outcome_id,
            &record.domain,
            &format!("outcome: expected={}, actual={}, score={:.2}", record.expected_outcome, record.actual_outcome, record.match_score),
            vec![record.match_score as f32, 1.0 - record.match_score as f32],
            record.match_score,
        );

        self.trust.append("reflection_loop", "lessons_extracted", outcome_id);
        lessons
    }

    pub fn generate_learning_delta(&mut self, domain: &str) -> LearningDelta {
        let domain_outcomes: Vec<&OutcomeRecord> = self.outcome_log.iter()
            .filter(|r| r.domain == domain)
            .collect();

        let total = domain_outcomes.len();
        let avg_score = if total > 0 {
            domain_outcomes.iter().map(|r| r.match_score).sum::<f64>() / total as f64
        } else { 1.0 };

        let mut adjustments = Vec::new();
        let mut confidence_delta = 0.0;

        if avg_score < 0.8 {
            confidence_delta = -0.05;
            adjustments.push(RuleAdjustment {
                rule_id: "confidence_threshold".to_string(),
                field: "threshold".to_string(),
                old_value: "0.85".to_string(),
                new_value: "0.90".to_string(),
                reason: format!("low_avg_score: {:.2}", avg_score),
            });
        } else if avg_score > 0.98 {
            confidence_delta = 0.02;
            adjustments.push(RuleAdjustment {
                rule_id: "confidence_threshold".to_string(),
                field: "threshold".to_string(),
                old_value: "0.85".to_string(),
                new_value: "0.80".to_string(),
                reason: format!("high_avg_score: {:.2}", avg_score),
            });
        }

        let cross_domain_patterns = self.detect_cross_domain_patterns(domain);
        
        let mut memory_additions = Vec::new();
        let mut memory_links = Vec::new();

        for pattern in &cross_domain_patterns {
            let node = MemoryNodeStub {
                id: format!("pattern_{}", now_ns()),
                domain: domain.to_string(),
                content: pattern.clone(),
                embedding: vec![avg_score as f32, 1.0],
                confidence: avg_score,
                timestamp_ns: now_ns(),
            };
            memory_additions.push(node);
        }

        self.trust.append("reflection_loop", "learning_delta_generated", domain);

        LearningDelta {
            domain: domain.to_string(),
            rule_adjustments: adjustments,
            memory_additions,
            memory_links,
            confidence_delta,
        }
    }

    pub fn apply_learning_delta(&mut self, delta: &LearningDelta) {
        for adjustment in &delta.rule_adjustments {
            self.trust.append(
                "reflection_loop",
                &format!("rule_adjust_{}", adjustment.rule_id),
                &format!("{}->{}", adjustment.old_value, adjustment.new_value)
            );
        }

        for node in &delta.memory_additions {
            self.memory.store(&node.id, &node.domain, &node.content, node.embedding.clone(), node.confidence);
        }

        self.trust.append("reflection_loop", "learning_delta_applied", &delta.domain);
    }

    pub fn get_domain_learning_stats(&self, domain: &str) -> DomainLearningStats {
        let outcomes: Vec<&OutcomeRecord> = self.outcome_log.iter().filter(|r| r.domain == domain).collect();
        let patterns: Vec<&PatternInstance> = self.pattern_index.values()
            .flatten()
            .filter(|p| p.domain == domain)
            .collect();

        let total = outcomes.len();
        let verified = outcomes.iter().filter(|r| r.verified).count();
        let avg_score = if total > 0 {
            outcomes.iter().map(|r| r.match_score).sum::<f64>() / total as f64
        } else { 0.0 };

        DomainLearningStats {
            domain: domain.to_string(),
            total_outcomes: total,
            verified_outcomes: verified,
            avg_match_score: avg_score,
            pattern_count: patterns.len(),
            last_update_ns: outcomes.last().map(|r| r.timestamp_ns).unwrap_or(0),
        }
    }

    fn detect_cross_domain_patterns(&self, domain: &str) -> Vec<String> {
        let mut patterns = Vec::new();
        
        for (key, instances) in &self.pattern_index {
            for instance in instances {
                if instance.domain != domain && instance.success_rate > 0.9 {
                    patterns.push(format!(
                        "cross_domain_learn: {} -> {} (success: {:.2})",
                        instance.domain, domain, instance.success_rate
                    ));
                }
            }
        }
        
        patterns
    }
}

pub fn calculate_match(expected: &str, actual: &str) -> f64 {
    if expected == actual { 1.0 } else {
        let common: String = expected.chars().zip(actual.chars())
            .filter(|(a, b)| a == b)
            .map(|(a, _)| a)
            .collect();
        common.len() as f64 / expected.len().max(actual.len()) as f64
    }
}

pub fn hash_pattern(pattern: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    pattern.hash(&mut hasher);
    format!("{:08x}", hasher.finish())
}

pub fn now_ns() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
}

#[derive(Debug, Clone)]
pub struct DomainLearningStats {
    pub domain: String,
    pub total_outcomes: usize,
    pub verified_outcomes: usize,
    pub avg_match_score: f64,
    pub pattern_count: usize,
    pub last_update_ns: u128,
}
