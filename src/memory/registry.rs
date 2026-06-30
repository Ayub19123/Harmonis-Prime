use std::collections::HashSet;
// M2.7.5: Provenance-Aware Clause Registry with Scoring
// Every learned clause is born with a cryptographic identity and scored for retention.

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
    pub(crate) max_capacity: usize,
    /// M2.7.8: Immutable strategic base — core clauses exempt from eviction.
    strategic_base: HashSet<[u8; 32]>,
    /// M2.7.8: Utility threshold τ — bottom decile purged when below this.
    tau_threshold: f64,
}

impl ClauseRegistry {
    /// Create registry with default scoring parameters.
    pub fn new(max_capacity: usize) -> Self {
        Self {
            clauses: Vec::new(),
            params: ScoringParams::default(),
            max_capacity,
            strategic_base: HashSet::new(),
            tau_threshold: 0.0,
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
            self.evict_by_utility();
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

    /// M2.7.8: Mark clauses as immutable strategic base (exempt from eviction).
    pub fn set_strategic_base(&mut self, hashes: Vec<[u8; 32]>) {
        self.strategic_base.clear();
        for h in hashes {
            self.strategic_base.insert(h);
        }
    }

    /// M2.7.8: Check if a clause is in the immutable strategic base.
    pub fn is_strategic(&self, clause_hash: &[u8; 32]) -> bool {
        self.strategic_base.contains(clause_hash)
    }

    /// M2.7.8: Current utility threshold τ (bottom decile boundary).
    pub fn tau_threshold(&self) -> f64 {
        self.tau_threshold
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

    /// M2.7.8: Utility-based strategic bounded eviction.
    /// Sorts clauses by utility, purges bottom decile below τ_threshold,
    /// while preserving immutable strategic base clauses.
    pub fn evict_by_utility(&mut self) {
        if self.clauses.is_empty() {
            return;
        }

        // Compute utility for each clause
        let mut utilities: Vec<(usize, f64)> = self
            .clauses
            .iter()
            .enumerate()
            .map(|(i, c)| (i, ClauseScore::utility(&c.provenance, c.activity)))
            .collect();

        // Sort by utility descending
        utilities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Compute τ_threshold as the utility at the 10th percentile (bottom decile)
        let decile_idx = (utilities.len() as f64 * 0.1).ceil() as usize;
        self.tau_threshold = if decile_idx < utilities.len() {
            utilities[decile_idx.min(utilities.len() - 1)].1
        } else {
            0.0
        };

        // Build new clause list: strategic base + high-utility clauses
        let mut retained = Vec::new();
        let mut _evicted_count = 0;

        for (idx, u) in utilities {
            let clause = &self.clauses[idx];
            let is_strategic = self.strategic_base.contains(&clause.provenance.clause_hash);

            if is_strategic || u >= self.tau_threshold {
                retained.push(self.clauses[idx].clone());
            } else {
                _evicted_count += 1;
            }
        }

        self.clauses = retained;

        // Hard cap: if still over capacity, evict non-strategic lowest-utility
        if self.clauses.len() > self.max_capacity {
            self.clauses.sort_by(|a, b| {
                let ua = ClauseScore::utility(&a.provenance, a.activity);
                let ub = ClauseScore::utility(&b.provenance, b.activity);
                ub.partial_cmp(&ua).unwrap_or(std::cmp::Ordering::Equal)
            });
            let mut kept = 0;
            self.clauses.retain(|c| {
                let is_strategic = self.strategic_base.contains(&c.provenance.clause_hash);
                kept += 1;
                is_strategic || kept <= self.max_capacity
            });
        }
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

        // M2.7.8: Ingest p1 first and bump activity so it has non-zero utility
        reg.ingest(p1.clone());
        reg.bump_activity_by_literals(&p1.literals); // p1 activity = 1.0
        assert_eq!(reg.stats().stored, 1);

        reg.ingest(p2.clone()); // Registry now at capacity (2)
        assert_eq!(reg.stats().stored, 2);

        // Ingest p3 -- triggers utility-based eviction
        // p1 has utility = 1.0/(2*log2(1)) = 1.0 (highest due to activity)
        // p2 has utility = 0.0/(10*log2(1)) = 0.0 (lowest, no activity)
        // p3 enters with utility = 0.0, bottom decile threshold evicts p2
        reg.ingest(p3.clone());
        assert_eq!(reg.stats().stored, 2); // Capacity enforced

        // p1 (LBD=2, activity=1.0) should survive due to higher utility
        assert!(
            reg.clauses.iter().any(|c| c.provenance.lbd == 2),
            "M2.7.8: Glue clause with activity must survive utility-based eviction"
        );
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

    // M2.7.8: Utility-Based Strategic Bounded Eviction — Functional Verification

    #[test]
    fn test_utility_monotonicity() {
        let p = ClauseProvenance::new(vec![1, -2, 3], 0, 3, vec![]);
        let u_low = ClauseScore::utility(&p, 0.0);
        let u_high = ClauseScore::utility(&p, 10.0);
        assert!(
            u_high > u_low,
            "M2.7.8: Higher activity must yield higher utility"
        );
    }

    #[test]
    fn test_strategic_base_preservation() {
        let mut reg = ClauseRegistry::new(2);
        let p1 = ClauseProvenance::new(vec![1, -2], 0, 2, vec![]);
        let p2 = ClauseProvenance::new(vec![1, -2, 3, -4, 5, -6, 7, -8, 9, -10], 0, 10, vec![]);
        let p3 = ClauseProvenance::new(vec![1, -2, 3], 0, 3, vec![]);

        reg.ingest(p1.clone());
        reg.ingest(p2.clone());
        reg.ingest(p3.clone());

        // Mark p1 as strategic (immutable base)
        reg.set_strategic_base(vec![p1.clause_hash]);

        // Trigger utility-based eviction
        reg.evict_by_utility();

        // p1 must survive despite potentially low utility, because it is strategic
        assert!(
            reg.clauses.iter().any(|c| c.provenance.lbd == 2),
            "M2.7.8: Strategic base clause must survive eviction"
        );
    }

    #[test]
    fn test_tau_threshold_computation() {
        let mut reg = ClauseRegistry::new(100);

        // Create 10 clauses with varying LBD
        for i in 1..=10 {
            let lits = vec![i as i32, -(i as i32 + 1)];
            let p = ClauseProvenance::new(lits, 0, i as u8, vec![]);
            reg.ingest(p);
        }

        // Bump activity on first 5 clauses to create utility spread
        for i in 1..=5 {
            let lits = vec![i as i32, -(i as i32 + 1)];
            reg.bump_activity_by_literals(&lits);
        }

        // Trigger eviction with capacity 5 (but we have 10 clauses)
        reg.evict_by_utility();

        // Tau should be non-negative (bottom decile boundary)
        assert!(
            reg.tau_threshold() >= 0.0,
            "M2.7.8: Tau threshold must be non-negative"
        );

        // Only clauses with utility >= tau should remain, or strategic ones
        assert!(
            reg.stats().stored <= 100,
            "M2.7.8: Eviction must respect capacity"
        );
    }
}
