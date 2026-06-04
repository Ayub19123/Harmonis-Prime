use crate::brick42::quantum::qpu_engine::{QPUEngine, QuantumBackend};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// PredictiveEngine: N-move lookahead for markets, logistics, healthcare
/// Calculates next likely N states, renders solution before query
pub struct PredictiveEngine {
    pub engine_id: String,
    pub lookahead_depth: usize,
    pub confidence_threshold: f64,
    pub state_history: VecDeque<StateSnapshot>,
    pub prediction_cache: HashMap<String, PredictionResult>,
    pub qpu_engine: QPUEngine,
}

/// StateSnapshot: Captured system state at time T
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub snapshot_id: String,
    pub timestamp_ns: u128,
    pub variables: HashMap<String, f64>,
    pub domain: String,
}

/// PredictionResult: Pre-computed solution with confidence intervals
#[derive(Debug, Clone)]
pub struct PredictionResult {
    pub prediction_id: String,
    pub query_pattern: String,
    pub predicted_states: Vec<StateSnapshot>,
    pub optimal_action: String,
    pub confidence: f64,
    pub time_to_event_ms: f64,
    pub precomputed_at: u128,
}

impl PredictiveEngine {
    pub fn new(engine_id: &str, lookahead_depth: usize) -> Self {
        Self {
            engine_id: engine_id.to_string(),
            lookahead_depth,
            confidence_threshold: 0.85,
            state_history: VecDeque::with_capacity(10000),
            prediction_cache: HashMap::new(),
            qpu_engine: QPUEngine::new(QuantumBackend::EdgeQPU, 32),
        }
    }

    /// Capture current state into history buffer
    pub fn observe(&mut self, variables: HashMap<String, f64>, domain: &str) {
        let snap = StateSnapshot {
            snapshot_id: format!("snap_{}_{}", self.engine_id, now_ns()),
            timestamp_ns: now_ns(),
            variables,
            domain: domain.to_string(),
        };
        self.state_history.push_back(snap);
        if self.state_history.len() > 10000 {
            self.state_history.pop_front();
        }
    }

    /// Pre-compute N-move lookahead for given query pattern
    /// Returns solution before user submits query
    pub fn precompute(&mut self, query_pattern: &str, domain: &str) -> PredictionResult {
        let cache_key = format!("{}_{}", domain, query_pattern);
        if let Some(cached) = self.prediction_cache.get(&cache_key) {
            return cached.clone();
        }
        let mut predicted_states = Vec::new();
        let mut current_vars = self
            .state_history
            .back()
            .map(|s| s.variables.clone())
            .unwrap_or_default();
        for step in 0..self.lookahead_depth {
            current_vars = self.project_next_state(&current_vars, domain);
            predicted_states.push(StateSnapshot {
                snapshot_id: format!("pred_{}_{}", step, now_ns()),
                timestamp_ns: now_ns(),
                variables: current_vars.clone(),
                domain: domain.to_string(),
            });
        }
        let optimal = self.select_optimal_action(&predicted_states, domain);
        let confidence = self.calculate_confidence(&predicted_states);
        let result = PredictionResult {
            prediction_id: format!("pred_{}_{}", self.engine_id, now_ns()),
            query_pattern: query_pattern.to_string(),
            predicted_states,
            optimal_action: optimal,
            confidence,
            time_to_event_ms: self.lookahead_depth as f64 * 100.0,
            precomputed_at: now_ns(),
        };
        self.prediction_cache.insert(cache_key, result.clone());
        result
    }

    /// Project next state from current variables using trend extrapolation
    fn project_next_state(
        &self,
        current: &HashMap<String, f64>,
        _domain: &str,
    ) -> HashMap<String, f64> {
        let mut next = HashMap::new();
        for (key, &val) in current.iter() {
            let trend = self.calculate_trend(key);
            let noise = rand_f64() * 0.01;
            next.insert(key.clone(), (val + trend + noise).max(0.0));
        }
        next
    }

    /// Calculate trend from historical snapshots for a variable
    fn calculate_trend(&self, key: &str) -> f64 {
        let values: Vec<f64> = self
            .state_history
            .iter()
            .filter_map(|s| s.variables.get(key).copied())
            .collect();
        if values.len() < 2 {
            return 0.0;
        }
        let recent = values[values.len() - 1];
        let previous = values[values.len() - 2];
        (recent - previous) * 0.5
    }

    /// Select optimal action based on predicted states
    fn select_optimal_action(&self, states: &[StateSnapshot], domain: &str) -> String {
        match domain.as_ref() {
            "finance" => {
                if let Some(last) = states.last() {
                    let price = last.variables.get("price").copied().unwrap_or(0.0);
                    let volume = last.variables.get("volume").copied().unwrap_or(0.0);
                    if price > 0.0 && volume > 1000000.0 {
                        return "HOLD_POSITION".to_string();
                    }
                }
                "ADJUST_POSITION".to_string()
            }
            "logistics" => {
                if let Some(last) = states.last() {
                    let congestion = last.variables.get("congestion").copied().unwrap_or(0.0);
                    if congestion > 0.8 {
                        return "REROUTE_IMMEDIATELY".to_string();
                    }
                }
                "MAINTAIN_ROUTE".to_string()
            }
            "healthcare" => {
                if let Some(last) = states.last() {
                    let risk = last.variables.get("patient_risk").copied().unwrap_or(0.0);
                    if risk > 0.9 {
                        return "ALERT_CRITICAL".to_string();
                    } else if risk > 0.7 {
                        return "SCHEDULE_CHECKUP".to_string();
                    }
                }
                "MONITOR_ROUTINE".to_string()
            }
            _ => "NO_ACTION".to_string(),
        }
    }

    /// Calculate prediction confidence from state variance
    fn calculate_confidence(&self, states: &[StateSnapshot]) -> f64 {
        if states.is_empty() {
            return 0.0;
        }
        let variances: Vec<f64> = states
            .iter()
            .map(|s| s.variables.values().copied().sum::<f64>() / s.variables.len().max(1) as f64)
            .collect();
        let mean = variances.iter().sum::<f64>() / variances.len() as f64;
        let variance = variances
            .iter()
            .map(|&v| (v - mean) * (v - mean))
            .sum::<f64>()
            / variances.len() as f64;
        (1.0 / (1.0 + variance)).min(1.0)
    }

    /// Check if prediction is still valid (not stale)
    pub fn is_fresh(&self, result: &PredictionResult, max_age_ms: f64) -> bool {
        let age_ms = (now_ns() - result.precomputed_at) as f64 / 1_000_000.0;
        age_ms < max_age_ms
    }
}

fn now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

fn rand_f64() -> f64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    now_ns().hash(&mut hasher);
    (hasher.finish() as f64) / (u64::MAX as f64)
}
