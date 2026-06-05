use crate::brick41::foundation::TrustLayer;
use std::collections::HashMap;

/// MemoryNode: A node in the Graph-RAG memory store
#[derive(Debug, Clone)]
pub struct MemoryNode {
    pub id: String,
    pub domain: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub confidence: f64,
    pub timestamp_ns: u128,
}

/// MemoryEdge: A relationship between two memory nodes
#[derive(Debug, Clone)]
pub struct MemoryEdge {
    pub source_id: String,
    pub target_id: String,
    pub relation: String,
    pub weight: f64,
    pub bidirectional: bool,
    pub timestamp_ns: u128,
}

/// RetrievalResult: A matched node with similarity score
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    pub node: MemoryNode,
    pub similarity: f32,
    pub confidence: f64,
    pub path: Vec<String>,
}

/// MemoryStore: Graph-RAG distributed state store with cross-domain learning
#[derive(Debug, Clone)]
pub struct MemoryStore {
    pub nodes: HashMap<String, MemoryNode>,
    pub edges: HashMap<String, Vec<MemoryEdge>>,
    pub domain_index: HashMap<String, Vec<String>>,
    pub trust: TrustLayer,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            domain_index: HashMap::new(),
            trust: TrustLayer::new(),
        }
    }

    pub fn store(
        &mut self,
        id: &str,
        domain: &str,
        content: &str,
        embedding: Vec<f32>,
        confidence: f64,
    ) {
        let timestamp_ns = now_ns();
        let node = MemoryNode {
            id: id.to_string(),
            domain: domain.to_string(),
            content: content.to_string(),
            embedding,
            confidence,
            timestamp_ns,
        };

        self.nodes.insert(id.to_string(), node.clone());
        self.domain_index
            .entry(domain.to_string())
            .or_insert_with(Vec::new)
            .push(id.to_string());
        self.trust.append("memory_store", "store", id);
    }

    pub fn link(
        &mut self,
        source_id: &str,
        target_id: &str,
        relation: &str,
        weight: f64,
        bidirectional: bool,
    ) {
        let timestamp_ns = now_ns();
        let edge = MemoryEdge {
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            relation: relation.to_string(),
            weight,
            bidirectional,
            timestamp_ns,
        };

        self.edges
            .entry(source_id.to_string())
            .or_insert_with(Vec::new)
            .push(edge.clone());

        if bidirectional {
            let reverse = MemoryEdge {
                source_id: target_id.to_string(),
                target_id: source_id.to_string(),
                relation: format!("reverse_{}", relation),
                weight,
                bidirectional: true,
                timestamp_ns,
            };
            self.edges
                .entry(target_id.to_string())
                .or_insert_with(Vec::new)
                .push(reverse);
        }

        self.trust.append(
            "memory_store",
            "link",
            &format!("{}->{}", source_id, target_id),
        );
    }

    pub fn retrieve(
        &self,
        query_embedding: &[f32],
        domain_filter: Option<&str>,
        top_k: usize,
    ) -> Vec<RetrievalResult> {
        let mut results: Vec<RetrievalResult> = Vec::new();

        for (id, node) in &self.nodes {
            if let Some(domain) = domain_filter {
                if node.domain != domain {
                    continue;
                }
            }

            let similarity = cosine_similarity(&node.embedding, query_embedding);
            results.push(RetrievalResult {
                node: node.clone(),
                similarity,
                confidence: node.confidence,
                path: vec![id.clone()],
            });
        }

        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        results.truncate(top_k);
        results
    }

    pub fn traverse(&self, start_id: &str, depth: usize) -> Vec<MemoryNode> {
        let mut visited: HashMap<String, bool> = HashMap::new();
        let mut queue: Vec<(String, usize)> = vec![(start_id.to_string(), 0)];
        let mut result: Vec<MemoryNode> = Vec::new();

        while let Some((current_id, current_depth)) = queue.pop() {
            if current_depth > depth {
                continue;
            }

            if visited.contains_key(&current_id) {
                continue;
            }
            visited.insert(current_id.clone(), true);

            if let Some(node) = self.nodes.get(&current_id) {
                result.push(node.clone());
            }

            if let Some(edges) = self.edges.get(&current_id) {
                for edge in edges {
                    if !visited.contains_key(&edge.target_id) {
                        queue.push((edge.target_id.clone(), current_depth + 1));
                    }
                }
            }
        }

        result
    }

    pub fn get_domain_nodes(&self, domain: &str) -> Vec<MemoryNode> {
        self.domain_index
            .get(domain)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.nodes.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.values().map(|v| v.len()).sum()
    }

    pub fn cross_domain_transfer(
        &mut self,
        source_domain: &str,
        target_domain: &str,
        pattern: &str,
    ) -> Vec<String> {
        let source_nodes = self.get_domain_nodes(source_domain);
        let mut transferred = Vec::new();

        for node in source_nodes {
            if node.content.contains(pattern) {
                let new_id = format!("transferred_{}_{}", target_domain, node.id);
                self.store(
                    &new_id,
                    target_domain,
                    &node.content,
                    node.embedding.clone(),
                    node.confidence * 0.95,
                );
                transferred.push(new_id);
            }
        }

        self.trust.append(
            "memory_store",
            "cross_domain_transfer",
            &format!("{}->{}", source_domain, target_domain),
        );
        transferred
    }
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

pub fn now_ns() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
