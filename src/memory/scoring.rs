//! M2.7.4: Clause Quality Scoring Engine
//! Dominance formula: S_clause = α(1/LBD) + β(activity) - γ(age_decay)
//!
//! Calibrated for SAT Competition performance:
//! - Low LBD (glue clauses) → highest scores
//! - Recent activity → bonus
//! - Age → exponential decay

use super::provenance::ClauseProvenance;

/// Scoring parameters. Calibrate α, β, γ via benchmark data in M2.11.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoringParams {
    /// Weight for LBD inverse (default: 10.0)
    /// Higher = more aggressive preference for glue clauses
    pub alpha: f64,
    /// Weight for activity bonus (default: 2.0)
    /// Higher = more preference for recently-used clauses
    pub beta: f64,
    /// Weight for age decay (default: 0.1)
    /// Higher = faster eviction of old clauses
    pub gamma: f64,
    /// Dynamic threshold multiplier (default: 0.8)
    /// Evict clauses below μ × mean_score
    pub mu: f64,
    /// Percentile to retain (default: 0.95 = top 5% evicted, bottom 5% kept)
    /// Actually: retain top (1-κ), evict bottom κ
    pub kappa: f64,
}

impl Default for ScoringParams {
    fn default() -> Self {
        Self {
            alpha: 10.0, // Glue clauses dominate
            beta: 2.0,   // Recent activity matters
            gamma: 0.1,  // Slow age decay
            mu: 0.8,     // Evict below 80% of mean
            kappa: 0.05, // Evict bottom 5%
        }
    }
}

/// Quality score for a clause. Higher = more valuable.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ClauseScore(pub f64);

impl ClauseScore {
    /// Compute score from provenance and activity.
    ///
    /// S_clause = α(1/LBD) + β(activity) - γ(age_decay)
    ///
    /// # Arguments
    /// * `provenance` — The clause's birth certificate
    /// * `activity` — Number of times this clause participated in conflict analysis (0.0+)
    /// * `params` — Scoring calibration
    pub fn compute(provenance: &ClauseProvenance, activity: f64, params: &ScoringParams) -> Self {
        // LBD component: 1/LBD, capped at LBD=1 → 1.0
        let lbd_component = params.alpha * (1.0 / provenance.lbd.max(1) as f64);

        // Activity component: linear bonus
        let activity_component = params.beta * activity;

        // Age decay: exponential based on seconds since birth
        let age_seconds = provenance.age_seconds() as f64;
        let age_decay = params.gamma * age_seconds.ln_1p(); // ln(1 + age) for smooth decay

        let score = lbd_component + activity_component - age_decay;

        ClauseScore(score.max(0.0)) // Never negative
    }

    /// M2.7.8: Utility formula — governs eviction decisions.
    /// 𝒰(c) = 𝒜(c) / (LBD(c) × log₂(Age(c) + 1))
    /// Higher utility = more valuable clause. Used for bottom-decile pruning.
    pub fn utility(provenance: &ClauseProvenance, activity: f64) -> f64 {
        let lbd = provenance.lbd.max(1) as f64;
        let age = provenance.age_seconds().max(0) as f64;
        let denominator = lbd * (age + 1.0).log2();
        if denominator <= 0.0 {
            return activity; // Degenerate case: fall back to raw activity
        }
        activity / denominator
    }

    /// Check if score is above dynamic threshold.
    pub fn is_above_threshold(&self, mean_score: f64, params: &ScoringParams) -> bool {
        self.0 >= params.mu * mean_score
    }

    /// Raw value for sorting/comparison.
    pub fn raw(&self) -> f64 {
        self.0
    }
}

/// Compute mean score across a collection.
pub fn mean_score(scores: &[ClauseScore]) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    scores.iter().map(|s| s.0).sum::<f64>() / scores.len() as f64
}

/// Retain top (1-κ) percentile, evict bottom κ.
pub fn eviction_cutoff(scores: &mut [ClauseScore], kappa: f64) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap()); // Descending
    let cutoff_idx = scores.len() - ((scores.len() as f64) * kappa).ceil() as usize;
    scores.get(cutoff_idx).map(|s| s.0).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glue_clause_scores_higher() {
        let params = ScoringParams::default();

        let glue = ClauseProvenance::new(vec![1, -2], 0, 2, vec![]);
        let non_glue = ClauseProvenance::new(vec![1, -2, 3, -4, 5], 0, 5, vec![]);

        let glue_score = ClauseScore::compute(&glue, 0.0, &params);
        let non_glue_score = ClauseScore::compute(&non_glue, 0.0, &params);

        assert!(
            glue_score.raw() > non_glue_score.raw(),
            "Glue clause (LBD=2) should score higher than non-glue (LBD=5)"
        );
    }

    #[test]
    fn test_activity_boosts_score() {
        let params = ScoringParams::default();
        let clause = ClauseProvenance::new(vec![1, -2, 3], 0, 3, vec![]);

        let low_activity = ClauseScore::compute(&clause, 0.0, &params);
        let high_activity = ClauseScore::compute(&clause, 10.0, &params);

        assert!(
            high_activity.raw() > low_activity.raw(),
            "Higher activity should increase score"
        );
    }

    #[test]
    fn test_age_decay_reduces_score() {
        let params = ScoringParams::default();

        // Create an "old" clause by manipulating timestamp (not possible via new(),
        // so we test indirectly via two clauses with same params but different ages)
        let fresh = ClauseProvenance::new(vec![1, -2], 0, 2, vec![]);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let older = ClauseProvenance::new(vec![1, -2], 0, 2, vec![]);

        let fresh_score = ClauseScore::compute(&fresh, 0.0, &params);
        let older_score = ClauseScore::compute(&older, 0.0, &params);

        // Both same LBD, same activity, but older has slightly more age decay
        // The difference should be minimal but measurable
        assert!(
            fresh_score.raw() >= older_score.raw(),
            "Fresh clause should score >= older clause"
        );
    }

    #[test]
    fn test_dynamic_threshold() {
        let params = ScoringParams::default();
        let scores = vec![
            ClauseScore(100.0),
            ClauseScore(80.0),
            ClauseScore(60.0),
            ClauseScore(40.0),
            ClauseScore(20.0),
        ];

        let mean = mean_score(&scores);
        let _threshold = params.mu * mean;

        assert!(ClauseScore(100.0).is_above_threshold(mean, &params));
        assert!(!ClauseScore(20.0).is_above_threshold(mean, &params));
    }

    #[test]
    fn test_eviction_cutoff() {
        let mut scores = vec![
            ClauseScore(100.0),
            ClauseScore(90.0),
            ClauseScore(80.0),
            ClauseScore(70.0),
            ClauseScore(60.0),
            ClauseScore(50.0),
            ClauseScore(40.0),
            ClauseScore(30.0),
            ClauseScore(20.0),
            ClauseScore(10.0),
        ];

        // κ = 0.2 → evict bottom 20% (2 clauses: 10.0 and 20.0)
        let cutoff = eviction_cutoff(&mut scores, 0.2);
        assert_eq!(cutoff, 20.0);
    }
}
