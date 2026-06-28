//! M2.7: Local Clause Registry with Epistemic Filtering
//! In-memory clause database with LBD-based admission control.

use super::packet::LitPack;
use std::collections::HashMap;

/// Epistemic filter parameters.
#[derive(Debug, Clone, Copy)]
pub struct FilterConfig {
    /// Weight for LBD score (lower is better)
    pub alpha: f64,
    /// Weight for clause size (shorter is better)
    pub beta: f64,
    /// Weight for activity/recency
    pub gamma: f64,
    /// Maximum utility threshold for admission
    pub delta: f64,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            beta: 0.5,
            gamma: 10.0,
            delta: 100.0,
        }
    }
}

/// Local clause registry with semantic indexing.
pub struct ClauseRegistry {
    /// Stored clauses keyed by content hash
    clauses: HashMap<u64, LitPack>,
    /// Epistemic filter configuration
    config: FilterConfig,
    /// Total clauses admitted
    admitted: usize,
    /// Total clauses rejected
    rejected: usize,
}

impl ClauseRegistry {
    /// Create new registry with default filter.
    pub fn new() -> Self {
        Self::with_config(FilterConfig::default())
    }

    /// Create new registry with custom filter.
    pub fn with_config(config: FilterConfig) -> Self {
        Self {
            clauses: HashMap::new(),
            config,
            admitted: 0,
            rejected: 0,
        }
    }

    /// Attempt to register a clause. Returns true if admitted.
    pub fn register(&mut self, pack: LitPack) -> bool {
        let utility = pack.utility(self.config.alpha, self.config.beta, self.config.gamma);

        if utility > self.config.delta {
            self.rejected += 1;
            return false;
        }

        let hash = Self::hash_clause(&pack.literals);
        self.clauses.insert(hash, pack);
        self.admitted += 1;
        true
    }

    /// Retrieve clause by content hash.
    pub fn get(&self, hash: u64) -> Option<&LitPack> {
        self.clauses.get(&hash)
    }

    /// Query clauses with LBD score below threshold.
    pub fn query_by_lbd(&self, max_lbd: u8) -> Vec<&LitPack> {
        self.clauses
            .values()
            .filter(|p| p.lbd_score <= max_lbd)
            .collect()
    }

    /// Simple hash for clause content.
    fn hash_clause(literals: &[i32]) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        literals.hash(&mut hasher);
        hasher.finish()
    }

    /// Registry statistics.
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            stored: self.clauses.len(),
            admitted: self.admitted,
            rejected: self.rejected,
        }
    }
}

#[derive(Debug)]
pub struct RegistryStats {
    pub stored: usize,
    pub admitted: usize,
    pub rejected: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admission_control() {
        let mut reg = ClauseRegistry::new();

        let good = LitPack::new(1, 2, vec![1, -2]);
        assert!(reg.register(good));

        let bad = LitPack::new(1, 200, vec![1; 1000]);
        assert!(!reg.register(bad));

        let stats = reg.stats();
        assert_eq!(stats.admitted, 1);
        assert_eq!(stats.rejected, 1);
    }
}
