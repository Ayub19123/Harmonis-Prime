use crate::brick41::foundation::{MemoryStore, RetrievalResult};
use crate::brick41::intelligence::{PatternInstance, LearningDelta};
use std::collections::{HashMap, HashSet};

/// CrossDomainLearningBridge: Domain-to-domain pattern transfer, knowledge fusion
/// BRICK-41 Phase 3: Intelligence — Cross-Domain Learning
#[derive(Debug, Clone)]
pub struct CrossDomainLearningBridge {
    pub memory: MemoryStore,
    pub domain_embeddings: HashMap<String, Vec<f32>,
    pub transfer_rules: Vec<TransferRule>,
    pub fusion_index: HashMap<String, FusedKnowledge>,
    pub similarity_threshold: f32,
}

#[derive(Debug, Clone)]
pub struct TransferRule {
    pub rule_id: String,
    pub source_domain: String,
    pub target_domain: String,
    pub pattern_type: PatternType,
    pub confidence_min: f64,
    pub transformation: Transformation,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternType {
    RiskModel,
    Optimization,
    ComplianceCheck,
    AnomalyDetection,
    ResourceAllocation,
}

#[derive(Debug, Clone)]
pub enum Transformation {
    DirectCopy,
    Scale { factor: f64 },
    Normalize { min: f64, max: f64 },
    DomainMap { field_mappings: Vec<(String, String)> },
}

#[derive(Debug, Clone)]
pub struct FusedKnowledge {
    pub fusion_id: String,
    pub source_domains: Vec<String>,
    pub fused_pattern: String,
    pub confidence: f64,
    pub applications: Vec<String>,
    pub timestamp_ns: u128,
}

#[derive(Debug, Clone)]
pub struct DomainSimilarity {
    pub domain_a: String,
    pub domain_b: String,
    pub similarity_score: f32,
    pub shared_patterns: Vec<String>,
}

impl CrossDomainLearningBridge {
    pub fn new(similarity_threshold: f32) -> Self {
        Self {
            memory: MemoryStore::new(),
            domain_embeddings: HashMap::new(),
            transfer_rules: default_transfer_rules(),
            fusion_index: HashMap::new(),
            similarity_threshold,
        }
    }

    pub fn compute_domain_similarity(&mut self, domain_a: &str, domain_b: &str) -> DomainSimilarity {
        let nodes_a = self.memory.retrieve(&[1.0, 0.0, 0.0], Some(domain_a), 100);
        let nodes_b = self.memory.retrieve(&[0.0, 1.0, 0.0], Some(domain_b), 100);

        let patterns_a: HashSet<String> = nodes_a.iter().map(|r| r.node.content.clone()).collect();
        let patterns_b: HashSet<String> = nodes_b.iter().map(|r| r.node.content.clone()).collect();

        let intersection: HashSet<&String> = patterns_a.intersection(&patterns_b).collect();
        let union: HashSet<&String> = patterns_a.union(&patterns_b).collect();

        let jaccard = if union.is_empty() { 0.0 } else { intersection.len() as f32 / union.len() as f32 };
        let cosine = cosine_similarity(
            self.domain_embedding(domain_a),
            self.domain_embedding(domain_b),
        );

        let similarity = (jaccard + cosine) / 2.0;

        let shared: Vec<String> = intersection.iter().map(|&s| s.clone()).collect();

        DomainSimilarity {
            domain_a: domain_a.to_string(),
            domain_b: domain_b.to_string(),
            similarity_score: similarity,
            shared_patterns: shared,
        }
    }

    pub fn transfer_knowledge(&mut self, rule_id: &str) -> Option<TransferResult> {
        let rule = self.transfer_rules.iter().find(|r| r.rule_id == rule_id && r.enabled)?;
        
        let source_patterns = self.memory.retrieve(&[1.0, 1.0, 1.0], Some(&rule.source_domain), 50);
        let high_confidence: Vec<&RetrievalResult> = source_patterns.iter()
            .filter(|r| r.confidence >= rule.confidence_min)
            .collect();

        if high_confidence.is_empty() {
            return None;
        }

        let mut transferred = Vec::new();
        for pattern in &high_confidence {
            let transformed = apply_transformation(&pattern.node.content, &rule.transformation);
            let target_node_id = format!("transferred_{}_{}", rule.target_domain, now_ns());
            
            self.memory.store(&target_node_id, &rule.target_domain, &transformed, pattern.node.embedding.clone(), pattern.confidence * 0.95);
            transferred.push(target_node_id);
        }

        Some(TransferResult {
            rule_id: rule_id.to_string(),
            source_domain: rule.source_domain.clone(),
            target_domain: rule.target_domain.clone(),
            transferred_patterns: transferred,
            transfer_count: high_confidence.len(),
            timestamp_ns: now_ns(),
        })
    }

    pub fn fuse_knowledge(&mut self, domains: &[String]) -> FusedKnowledge {
        let mut all_patterns = Vec::new();
        let mut source_domains = Vec::new();

        for domain in domains {
            let patterns = self.memory.retrieve(&[1.0, 1.0, 1.0], Some(domain), 50);
            for p in &patterns {
                all_patterns.push(p.node.content.clone());
            }
            source_domains.push(domain.clone());
        }

        let fused_pattern = all_patterns.join(" | ");
        let avg_confidence = if all_patterns.is_empty() { 0.0 } else { 0.9 };

        let fusion = FusedKnowledge {
            fusion_id: format!("fusion_{}", now_ns()),
            source_domains,
            fused_pattern,
            confidence: avg_confidence,
            applications: domains.to_vec(),
            timestamp_ns: now_ns(),
        };

        self.fusion_index.insert(fusion.fusion_id.clone(), fusion.clone());
        fusion
    }

    pub fn get_transfer_opportunities(&self) -> Vec<TransferOpportunity> {
        let mut opportunities = Vec::new();

        for rule in &self.transfer_rules {
            if !rule.enabled { continue; }

            let similarity = compute_domain_similarity_cached(&rule.source_domain, &rule.target_domain);
            
            if similarity.similarity_score >= self.similarity_threshold {
                let source_count = self.memory.retrieve(&[1.0, 1.0, 1.0], Some(&rule.source_domain), 1).len();
                
                opportunities.push(TransferOpportunity {
                    rule_id: rule.rule_id.clone(),
                    source_domain: rule.source_domain.clone(),
                    target_domain: rule.target_domain.clone(),
                    similarity_score: similarity.similarity_score,
                    available_patterns: source_count,
                    estimated_impact: similarity.similarity_score * source_count as f32,
                });
            }
        }

        opportunities
    }

    fn domain_embedding(&mut self, domain: &str) -> Vec<f32> {
        self.domain_embeddings.entry(domain.to_string()).or_insert_with(|| {
            let hash = hash_domain(domain);
            vec![
                ((hash >> 24) & 0xFF) as f32 / 255.0,
                ((hash >> 16) & 0xFF) as f32 / 255.0,
                ((hash >> 8) & 0xFF) as f32 / 255.0,
            ]
        }).clone()
    }
}

pub fn default_transfer_rules() -> Vec<TransferRule> {
    vec![
        TransferRule {
            rule_id: "finance_to_logistics_risk".to_string(),
            source_domain: "finance".to_string(),
            target_domain: "logistics".to_string(),
            pattern_type: PatternType::RiskModel,
            confidence_min: 0.9,
            transformation: Transformation::DomainMap {
                field_mappings: vec![
                    ("credit_score".to_string(), "route_reliability".to_string()),
                    ("default_probability".to_string(), "delay_probability".to_string()),
                ],
            },
            enabled: true,
        },
        TransferRule {
            rule_id: "healthcare_to_finance_compliance".to_string(),
            source_domain: "healthcare".to_string(),
            target_domain: "finance".to_string(),
            pattern_type: PatternType::ComplianceCheck,
            confidence_min: 0.95,
            transformation: Transformation::DirectCopy,
            enabled: true,
        },
        TransferRule {
            rule_id: "logistics_to_healthcare_opt".to_string(),
            source_domain: "logistics".to_string(),
            target_domain: "healthcare".to_string(),
            pattern_type: PatternType::Optimization,
            confidence_min: 0.85,
            transformation: Transformation::Scale { factor: 1.5 },
            enabled: true,
        },
    ]
}

pub fn apply_transformation(content: &str, transformation: &Transformation) -> String {
    match transformation {
        Transformation::DirectCopy => content.to_string(),
        Transformation::Scale { factor } => format!("scaled({:.2}): {}", factor, content),
        Transformation::Normalize { min, max } => format!("normalized({:.2}-{:.2}): {}", min, max, content),
        Transformation::DomainMap { field_mappings } => {
            let mut result = content.to_string();
            for (from, to) in field_mappings {
                result = result.replace(from, to);
            }
            result
        }
    }
}

pub fn cosine_similarity(a: Vec<f32>, b: Vec<f32>) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot / (norm_a * norm_b) }
}

pub fn compute_domain_similarity_cached(domain_a: &str, domain_b: &str) -> DomainSimilarity {
    DomainSimilarity {
        domain_a: domain_a.to_string(),
        domain_b: domain_b.to_string(),
        similarity_score: 0.5,
        shared_patterns: vec![],
    }
}

pub fn hash_domain(domain: &str) -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    domain.hash(&mut hasher);
    (hasher.finish() & 0xFFFFFFFF) as u32
}

pub fn now_ns() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
}

#[derive(Debug, Clone)]
pub struct TransferResult {
    pub rule_id: String,
    pub source_domain: String,
    pub target_domain: String,
    pub transferred_patterns: Vec<String>,
    pub transfer_count: usize,
    pub timestamp_ns: u128,
}

#[derive(Debug, Clone)]
pub struct TransferOpportunity {
    pub rule_id: String,
    pub source_domain: String,
    pub target_domain: String,
    pub similarity_score: f32,
    pub available_patterns: usize,
    pub estimated_impact: f32,
}
