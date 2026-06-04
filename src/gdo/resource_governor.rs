use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ResourceMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub latency_ms: u64,
    pub throughput_ops: u64,
}

#[derive(Debug, Clone)]
pub struct ThrottleDecision {
    pub should_throttle: bool,
    pub new_rate_limit: u64,
    pub reason: String,
}

pub struct ResourceGovernor {
    pub cpu_limit: f64,
    pub memory_limit_mb: u64,
    pub latency_limit_ms: u64,
    pub instruction_cache: HashMap<String, u64>,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl ResourceGovernor {
    pub fn new(cpu_limit: f64, memory_limit_mb: u64, latency_limit_ms: u64) -> Self {
        Self {
            cpu_limit: cpu_limit.clamp(0.0, 100.0),
            memory_limit_mb,
            latency_limit_ms,
            instruction_cache: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    pub fn evaluate_resources(&self, metrics: &ResourceMetrics) -> ThrottleDecision {
        if metrics.cpu_usage_percent > self.cpu_limit {
            return ThrottleDecision {
                should_throttle: true,
                new_rate_limit: (metrics.throughput_ops / 2).max(1),
                reason: format!(
                    "CPU {:.1}% exceeds limit {:.1}%",
                    metrics.cpu_usage_percent, self.cpu_limit
                ),
            };
        }

        if metrics.memory_usage_mb > self.memory_limit_mb {
            return ThrottleDecision {
                should_throttle: true,
                new_rate_limit: (metrics.throughput_ops / 2).max(1),
                reason: format!(
                    "Memory {}MB exceeds limit {}MB",
                    metrics.memory_usage_mb, self.memory_limit_mb
                ),
            };
        }

        if metrics.latency_ms > self.latency_limit_ms {
            return ThrottleDecision {
                should_throttle: true,
                new_rate_limit: (metrics.throughput_ops / 2).max(1),
                reason: format!(
                    "Latency {}ms exceeds limit {}ms",
                    metrics.latency_ms, self.latency_limit_ms
                ),
            };
        }

        ThrottleDecision {
            should_throttle: false,
            new_rate_limit: metrics.throughput_ops,
            reason: String::from("Within bounds"),
        }
    }

    pub fn cache_instruction(&mut self, instruction_fingerprint: String) -> bool {
        if let Some(count) = self.instruction_cache.get(&instruction_fingerprint) {
            self.instruction_cache
                .insert(instruction_fingerprint, count + 1);
            self.cache_hits += 1;
            true
        } else {
            self.instruction_cache.insert(instruction_fingerprint, 1);
            self.cache_misses += 1;
            false
        }
    }

    pub fn get_cache_efficiency(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total as f64).clamp(0.0, 1.0)
        }
    }

    pub fn prune_cache(&mut self, max_size: usize) {
        if self.instruction_cache.len() > max_size {
            let mut entries: Vec<_> = self.instruction_cache.iter().collect();
            entries.sort_by(|a, b| b.1.cmp(a.1));
            let to_keep = entries
                .into_iter()
                .take(max_size)
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            self.instruction_cache = to_keep;
        }
    }
}
