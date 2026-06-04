use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// TelemetryEvent: A single observation from the organism
/// Event = (timestamp, substrate, metric_type, value, context)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub timestamp_nanos: u64,
    pub substrate: String,
    pub metric_type: MetricType,
    pub value: f64,
    pub context: String,
}

/// MetricType: The class of observable
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    LatencyMicros,
    ThroughputPerSec,
    ErrorRate,
    MemoryBytes,
    CpuPercent,
    LearningRate,
    ConfidenceScore,
    DecisionEntropy,
    ReasoningDepth,
    KnowledgeGain,
}

/// Metric: Aggregated statistics over a time window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub metric_type: MetricType,
    pub window_start_nanos: u64,
    pub window_end_nanos: u64,
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub variance: f64,
}

/// AggregationWindow: Temporal bucketing for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationWindow {
    pub duration_nanos: u64,
    pub events: Vec<Event>,
}

/// TelemetryConfig: Configuration for the observability system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub max_events: usize,
    pub aggregation_window_nanos: u64,
    pub enable_reasoning_trace: bool,
    pub enable_learning_trace: bool,
    pub enable_health_monitoring: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            max_events: 100_000,
            aggregation_window_nanos: 1_000_000_000, // 1 second
            enable_reasoning_trace: true,
            enable_learning_trace: true,
            enable_health_monitoring: true,
        }
    }
}

/// TelemetryStream: The live, append-only event stream
/// Mathematical invariant: events are strictly monotonic in timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryStream {
    pub events: VecDeque<Event>,
    pub config: TelemetryConfig,
    pub total_events: u64,
    pub substrate_counters: std::collections::HashMap<String, u64>,
}

impl TelemetryStream {
    /// Create new telemetry stream
    pub fn new(config: TelemetryConfig) -> Self {
        Self {
            events: VecDeque::with_capacity(config.max_events),
            config,
            total_events: 0,
            substrate_counters: std::collections::HashMap::new(),
        }
    }

    /// Emit event — O(1) amortized
    pub fn emit(&mut self, substrate: &str, metric_type: MetricType, value: f64, context: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let event = Event {
            timestamp_nanos: now,
            substrate: substrate.to_string(),
            metric_type,
            value,
            context: context.to_string(),
        };

        // Ring buffer eviction
        if self.events.len() >= self.config.max_events {
            self.events.pop_front();
        }

        self.events.push_back(event);
        self.total_events += 1;

        *self
            .substrate_counters
            .entry(substrate.to_string())
            .or_insert(0) += 1;
    }

    /// Aggregate metrics over time window
    pub fn aggregate(&self, metric_type: MetricType, window_nanos: u64) -> Option<Metric> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let window_start = now.saturating_sub(window_nanos);

        let filtered: Vec<&Event> = self
            .events
            .iter()
            .filter(|e| e.metric_type == metric_type && e.timestamp_nanos >= window_start)
            .collect();

        if filtered.is_empty() {
            return None;
        }

        let count = filtered.len() as u64;
        let sum: f64 = filtered.iter().map(|e| e.value).sum();
        let min = filtered
            .iter()
            .map(|e| e.value)
            .fold(f64::INFINITY, |a, b| a.min(b));
        let max = filtered
            .iter()
            .map(|e| e.value)
            .fold(f64::NEG_INFINITY, |a, b| a.max(b));
        let mean = sum / count as f64;

        let variance = if count > 1 {
            filtered
                .iter()
                .map(|e| (e.value - mean).powi(2))
                .sum::<f64>()
                / (count - 1) as f64
        } else {
            0.0
        };

        Some(Metric {
            metric_type,
            window_start_nanos: window_start,
            window_end_nanos: now,
            count,
            sum,
            min,
            max,
            mean,
            variance,
        })
    }

    /// Get events by substrate
    pub fn by_substrate(&self, substrate: &str) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|e| e.substrate == substrate)
            .collect()
    }

    /// Get latest event
    pub fn latest(&self) -> Option<&Event> {
        self.events.back()
    }

    /// Stream statistics
    pub fn stats(&self) -> StreamStats {
        StreamStats {
            total_events: self.total_events,
            buffered_events: self.events.len() as u64,
            substrate_count: self.substrate_counters.len() as u64,
            substrates: self.substrate_counters.keys().cloned().collect(),
        }
    }
}

/// StreamStats: Observability of the observability system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStats {
    pub total_events: u64,
    pub buffered_events: u64,
    pub substrate_count: u64,
    pub substrates: Vec<String>,
}
