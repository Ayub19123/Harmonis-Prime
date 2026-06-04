//! BRICK-47 Pillar 2: Temporal Memory Engine (TME)
//! Semantic incident store with logarithmic decay and remediation reuse
//! Benchmark: >= 80% reuse of past successful fixes

use crate::brick47::types::IncidentRecord;
use std::collections::HashMap;
use std::time::Instant;

/// TemporalMemoryEngine: Long-term incident memory with decay and reinforcement
pub struct TemporalMemoryEngine {
    incidents: HashMap<String, IncidentRecord>,
    pattern_index: HashMap<String, Vec<String>>,
    max_incidents: usize,
    hit_count: u64,
    miss_count: u64,
}

impl TemporalMemoryEngine {
    pub fn new(max_incidents: usize) -> Self {
        Self {
            incidents: HashMap::with_capacity(max_incidents),
            pattern_index: HashMap::new(),
            max_incidents,
            hit_count: 0,
            miss_count: 0,
        }
    }

    pub fn store(&mut self, incident: IncidentRecord) {
        if self.incidents.len() >= self.max_incidents {
            let now = Instant::now();
            if let Some(evict_id) = self
                .incidents
                .iter()
                .min_by(|a, b| {
                    a.1.decay_score(now)
                        .partial_cmp(&b.1.decay_score(now))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(k, _)| k.clone())
            {
                self.incidents.remove(&evict_id);
                for ids in self.pattern_index.values_mut() {
                    ids.retain(|id| id != &evict_id);
                }
            }
        }

        let id = incident.id.clone();
        let pattern = incident.pattern_signature.clone();
        self.incidents.insert(id.clone(), incident);
        self.pattern_index
            .entry(pattern)
            .or_insert_with(Vec::new)
            .push(id);
    }

    pub fn retrieve_remediation(&mut self, pattern: &str, context: &str) -> Option<(String, f64)> {
        let now = Instant::now();

        let candidates = self.pattern_index.get(pattern)?;
        if candidates.is_empty() {
            self.miss_count += 1;
            return None;
        }

        let mut best_id: Option<String> = None;
        let mut best_score: f64 = 0.0;

        for id in candidates {
            if let Some(incident) = self.incidents.get_mut(id) {
                let score = incident.decay_score(now);
                let context_bonus = if incident.context == context {
                    2.0
                } else {
                    1.0
                };
                let adjusted_score = score * context_bonus;

                if best_id.is_none() || adjusted_score > best_score {
                    best_id = Some(id.clone());
                    best_score = adjusted_score;
                }

                if incident.context == context {
                    incident.access_count += 1;
                }
            }
        }

        if let Some(id) = best_id {
            self.hit_count += 1;
            let remediation = self.incidents.get(&id).unwrap().remediation.clone();
            Some((remediation, best_score))
        } else {
            self.miss_count += 1;
            None
        }
    }

    pub fn stats(&self) -> (usize, u64, u64) {
        (self.incidents.len(), self.hit_count, self.miss_count)
    }

    pub fn incident_count(&self) -> usize {
        self.incidents.len()
    }
}
