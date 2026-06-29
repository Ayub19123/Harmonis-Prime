//! M2.7.5: Provenance-Aware Clause Registry with Scoring
//! Every learned clause is born with a cryptographic identity and scored for retention.

use super::provenance::ClauseProvenance;
use super::scoring::{ClauseScore, ScoringParams};

/// Scored clause entry: provenance + computed score + activity counter.
#[derive(Debug, Clone)]
pub struct ScoredClause {
    pub provenance: ClauseProvenance,
    pub score: ClauseScore,
    pub activity: f64, // Times this clause participated in conflict analysis
}

/// Provenance-aware clause registry with dynamic eviction.
#[derive(Default)]
pub struct ClauseRegistry {
    clauses: Vec<ScoredClause>,
    params: ScoringParams,
    max_capacity: usize,
}

impl ClauseRegistry {
    /// Create registry with default scoring parameters.
    pub fn new(max_capacity: usize) -> Self {
        Self {
            clauses: Vec::new(),
            params: ScoringParams::default(),
            max_capacity,
        }
    }

    /// Ingest a learned clause with provenance. Returns true if retained.
    /// Computes score immediately and evicts bottom-κ if over capacity.
    pub fn ingest(&mut self, provenance: ClauseProvenance) -> bool {
        let score = ClauseScore::compute(&provenance, 0.0, &self.params);
        let entry = ScoredClause {
            provenance,
            score,
            activity: 0.0,
        };

        // Check for exact duplicates by clause_hash
        if self
            .clauses
            .iter()
            .any(|c| c.provenance.clause_hash == entry.provenance.clause_hash)
        {
            return false; // Duplicate suppressed
        }

        self.clauses.push(entry);

        // Enforce capacity with dynamic eviction
        if self.clauses.len() > self.max_capacity {
            self.evict_bottom_k();
        }

        true
    }

    /// Increment activity for a clause (called when it participates in conflict analysis).
    pub fn bump_activity(&mut self, clause_hash: &[u8; 32]) {
        if let Some(entry) = self
            .clauses
            .iter_mut()
            .find(|c| c.provenance.clause_hash == *clause_hash)
        {
            entry.activity += 1.0;
            // Recompute score with updated activity
            entry.score = ClauseScore::compute(&entry.provenance, entry.activity, &self.params);
        }
    }

    /// Bump activity for a clause identified by its literals (not hash).
    /// Computes the hash internally and delegates to bump_activity.
    pub fn bump_activity_by_literals(&mut self, literals: &[i32]) {
        // Match by sorted literal content (deterministic, timestamp-independent)
        let mut target = literals.to_vec();
        target.sort_unstable();
        if let Some(entry) = self.clauses.iter_mut().find(|c| {
            let mut stored = c.provenance.literals.clone();
            stored.sort_unstable();
            stored == target
        }) {
            entry.activity += 1.0;
            // Recompute score with updated activity
            entry.score = ClauseScore::compute(&entry.provenance, entry.activity, &self.params);
        }
    }
    /// Retrieve top-scored clauses by minimum LBD threshold.
    pub fn query_by_lbd(&self, max_lbd: u8) -> Vec<&ScoredClause> {
        self.clauses
            .iter()
            .filter(|c| c.provenance.lbd <= max_lbd)
            .collect()
    }

    /// Export top-k clauses for DHT gossip (future M2.7.6).
    pub fn export_top_k(&self, k: usize) -> Vec<&ScoredClause> {
        let mut sorted: Vec<_> = self.clauses.iter().collect();
        sorted.sort_by(|a, b| b.score.raw().partial_cmp(&a.score.raw()).unwrap());
        sorted.into_iter().take(k).collect()
    }

    /// Registry statistics.
    pub fn stats(&self) -> RegistryStats {
        let mean = if self.clauses.is_empty() {
            0.0
        } else {
            self.clauses.iter().map(|c| c.score.raw()).sum::<f64>() / self.clauses.len() as f64
        };

        RegistryStats {
            stored: self.clauses.len(),
            mean_score: mean,
            glue_clauses: self
                .clauses
                .iter()
                .filter(|c| c.provenance.is_glue())
                .count(),
        }
    }

    /// Dynamic eviction: retain only top max_capacity clauses by score.
    fn evict_bottom_k(&mut self) {
        // Sort by score descending, keep top max_capacity
        self.clauses
            .sort_by(|a, b| b.score.raw().partial_cmp(&a.score.raw()).unwrap());
        self.clauses.truncate(self.max_capacity);
    }
}

#[derive(Debug)]
pub struct RegistryStats {
    pub stored: usize,
    pub mean_score: f64,
    pub glue_clauses: usize,
}

#[cfg(test)]
mod tests {
    use super::super::provenance::ClauseProvenance;
    use super::*;

    #[test]
    fn test_ingest_and_retrieve() {
        let mut reg = ClauseRegistry::new(100);
        let p = ClauseProvenance::new(vec![1, -2], 0, 2, vec![]);
        assert!(reg.ingest(p.clone()));
        assert_eq!(reg.stats().stored, 1);
        assert_eq!(reg.stats().glue_clauses, 1);
    }

    #[test]
    fn test_duplicate_suppression() {
        let mut reg = ClauseRegistry::new(100);
        let p = ClauseProvenance::new(vec![1, -2, 3], 0, 3, vec![]);
        assert!(reg.ingest(p.clone()));
        assert!(!reg.ingest(p)); // Same hash = duplicate
        assert_eq!(reg.stats().stored, 1);
    }

    #[test]
    fn test_eviction_at_capacity() {
        let mut reg = ClauseRegistry::new(2); // Tiny capacity to force eviction
        let p1 = ClauseProvenance::new(vec![1, -2], 0, 2, vec![]); // Glue = high score
        let p2 = ClauseProvenance::new(vec![1, -2, 3, -4, 5, -6, 7, -8, 9, -10], 0, 10, vec![]); // Low score
        let p3 = ClauseProvenance::new(vec![1, -2, 3], 0, 3, vec![]); // Medium score

        reg.ingest(p2.clone()); // Low score first
        reg.ingest(p3.clone()); // Medium score
        assert_eq!(reg.stats().stored, 2);

        reg.ingest(p1.clone()); // High score (glue) — should evict low score
        assert_eq!(reg.stats().stored, 2); // Capacity enforced
        assert!(reg.clauses.iter().any(|c| c.provenance.lbd == 2)); // Glue retained
    }

    #[test]
    fn test_activity_bump_updates_score() {
        let mut reg = ClauseRegistry::new(100);
        let p = ClauseProvenance::new(vec![1, -2, 3], 0, 3, vec![]);
        let hash = p.clause_hash;
        reg.ingest(p);

        let before = reg.clauses[0].score.raw();
        reg.bump_activity(&hash);
        let after = reg.clauses[0].score.raw();

        assert!(after > before, "Activity bump should increase score");
    }

    #[test]
    fn test_export_top_k() {
        let mut reg = ClauseRegistry::new(100);
        let p1 = ClauseProvenance::new(vec![1, -2], 0, 2, vec![]); // High score (glue)
        let p2 = ClauseProvenance::new(vec![1, -2, 3, -4, 5], 0, 5, vec![]); // Lower score

        reg.ingest(p2);
        reg.ingest(p1.clone());

        let top = reg.export_top_k(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].provenance.lbd, 2); // Glue clause is top
    }
}
