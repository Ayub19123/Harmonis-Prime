use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// SubstrateHealth: Vital signs for a single substrate
/// Vitality = f(heartbeat, error_rate, latency, resource_usage)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstrateHealth {
    pub substrate_name: String,
    pub status: HealthStatus,
    pub heartbeat_nanos: u64,
    pub error_rate: f64,
    pub avg_latency_micros: f64,
    pub memory_bytes: u64,
    pub cpu_percent: f64,
    pub last_error: Option<String>,
    pub consecutive_failures: u64,
}

/// HealthStatus: Discrete vitality states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,    // All vitals within bounds
    Degraded,   // One or more vitals approaching threshold
    Critical,   // Vitals exceeded threshold — intervention required
    Offline,    // Substrate not responding
    Recovering, // Substrate returning from critical/degraded
}

/// VitalityScore: Aggregate health metric [0.0, 1.0]
/// 1.0 = perfect health, 0.0 = dead
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalityScore {
    pub overall: f64,
    pub latency_score: f64,
    pub error_score: f64,
    pub resource_score: f64,
    pub uptime_score: f64,
}

/// HealthSnapshot: Complete organism health at a moment in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    pub timestamp_nanos: u64,
    pub global_status: HealthStatus,
    pub global_vitality: VitalityScore,
    pub substrate_health: Vec<SubstrateHealth>,
    pub active_alerts: Vec<HealthAlert>,
    pub recovery_actions: Vec<RecoveryAction>,
}

/// HealthAlert: A condition requiring attention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    pub alert_id: u64,
    pub substrate: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp_nanos: u64,
    pub acknowledged: bool,
}

/// AlertSeverity: Priority classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// RecoveryAction: Automated or manual intervention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAction {
    pub action_id: u64,
    pub target_substrate: String,
    pub action_type: RecoveryType,
    pub executed: bool,
    pub success: Option<bool>,
    pub timestamp_nanos: u64,
}

/// RecoveryType: Classification of intervention
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryType {
    Restart,
    Throttle,
    Failover,
    Rollback,
    Isolate,
    AlertOperator,
}

/// HealthMonitor: The central nervous system of organism vitality
/// Monitors all substrates, computes vitality scores, triggers recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitor {
    pub substrates: HashMap<String, SubstrateHealth>,
    pub alert_counter: u64,
    pub action_counter: u64,
    pub history: Vec<HealthSnapshot>,
    pub max_history: usize,
    pub latency_threshold_micros: f64,
    pub error_threshold: f64,
    pub cpu_threshold: f64,
    pub memory_threshold_bytes: u64,
}

impl SubstrateHealth {
    /// Create new substrate health tracker
    pub fn new(name: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            substrate_name: name.to_string(),
            status: HealthStatus::Healthy,
            heartbeat_nanos: now,
            error_rate: 0.0,
            avg_latency_micros: 0.0,
            memory_bytes: 0,
            cpu_percent: 0.0,
            last_error: None,
            consecutive_failures: 0,
        }
    }

    /// Update vitals and recompute status
    pub fn update(&mut self, latency: f64, errors: u64, total: u64, memory: u64, cpu: f64) {
        self.heartbeat_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        self.avg_latency_micros = latency;
        self.error_rate = if total > 0 {
            errors as f64 / total as f64
        } else {
            0.0
        };
        self.memory_bytes = memory;
        self.cpu_percent = cpu;

        // Status logic
        self.status = if self.consecutive_failures > 5 {
            HealthStatus::Critical
        } else if self.error_rate > 0.1 || self.avg_latency_micros > 10000.0 {
            HealthStatus::Degraded
        } else if self.consecutive_failures > 0 {
            HealthStatus::Recovering
        } else {
            HealthStatus::Healthy
        };
    }

    /// Record a failure
    pub fn record_failure(&mut self, error: &str) {
        self.consecutive_failures += 1;
        self.last_error = Some(error.to_string());
    }

    /// Record a success (resets failure counter)
    pub fn record_success(&mut self) {
        if self.consecutive_failures > 0 {
            self.consecutive_failures = 0;
            self.status = HealthStatus::Recovering;
        }
    }

    /// Compute vitality score for this substrate [0.0, 1.0]
    pub fn vitality(&self) -> f64 {
        let latency_score = (1.0 / (1.0 + self.avg_latency_micros / 1000.0)).min(1.0);
        let error_score = (1.0 - self.error_rate).max(0.0);
        let resource_score = (1.0 - self.cpu_percent / 100.0).max(0.0);

        (latency_score * 0.3 + error_score * 0.4 + resource_score * 0.3).clamp(0.0, 1.0)
    }
}

impl VitalityScore {
    /// Compute aggregate from substrate vitals
    pub fn from_substrates(substrates: &HashMap<String, SubstrateHealth>) -> Self {
        let count = substrates.len() as f64;
        if count == 0.0 {
            return Self {
                overall: 1.0,
                latency_score: 1.0,
                error_score: 1.0,
                resource_score: 1.0,
                uptime_score: 1.0,
            };
        }

        let avg_latency = substrates
            .values()
            .map(|s| s.avg_latency_micros)
            .sum::<f64>()
            / count;
        let avg_error = substrates.values().map(|s| s.error_rate).sum::<f64>() / count;
        let avg_cpu = substrates.values().map(|s| s.cpu_percent).sum::<f64>() / count;

        let latency_score = (1.0 / (1.0 + avg_latency / 1000.0)).min(1.0);
        let error_score = (1.0 - avg_error).max(0.0);
        let resource_score = (1.0 - avg_cpu / 100.0).max(0.0);
        let uptime_score = substrates
            .values()
            .filter(|s| s.status == HealthStatus::Healthy)
            .count() as f64
            / count;

        let overall =
            (latency_score * 0.2 + error_score * 0.3 + resource_score * 0.2 + uptime_score * 0.3)
                .clamp(0.0, 1.0);

        Self {
            overall,
            latency_score,
            error_score,
            resource_score,
            uptime_score,
        }
    }
}

impl HealthMonitor {
    /// Create new health monitor
    pub fn new() -> Self {
        let mut substrates = HashMap::new();

        // Initialize all sovereign substrates
        for name in &[
            "BRICK-23: Consensus",
            "BRICK-24: POM",
            "BRICK-25: CRE",
            "BRICK-26: RSO",
            "BRICK-27: TSG",
            "BRICK-28.2: GDO",
            "BRICK-29: FLOW",
            "BRICK-30: FV",
            "BRICK-31: FABRIC",
            "BRICK-32: INTEL",
            "BRICK-34: REPLAY",
            "BRICK-36: TELEMETRY",
        ] {
            substrates.insert(name.to_string(), SubstrateHealth::new(name));
        }

        Self {
            substrates,
            alert_counter: 0,
            action_counter: 0,
            history: Vec::with_capacity(1000),
            max_history: 1000,
            latency_threshold_micros: 10000.0,
            error_threshold: 0.05,
            cpu_threshold: 80.0,
            memory_threshold_bytes: 1024 * 1024 * 1024, // 1GB
        }
    }

    /// Update substrate vitals
    pub fn update_substrate(
        &mut self,
        name: &str,
        latency: f64,
        errors: u64,
        total: u64,
        memory: u64,
        cpu: f64,
    ) {
        if let Some(substrate) = self.substrates.get_mut(name) {
            substrate.update(latency, errors, total, memory, cpu);

            // Auto-alert on threshold breach
            if substrate.status == HealthStatus::Critical {
                self.alert_counter += 1;
            }
        }
    }

    /// Record failure on substrate
    pub fn record_failure(&mut self, name: &str, error: &str) {
        if let Some(substrate) = self.substrates.get_mut(name) {
            substrate.record_failure(error);

            if substrate.consecutive_failures >= 3 {
                self.alert_counter += 1;
            }
        }
    }

    /// Record success on substrate
    pub fn record_success(&mut self, name: &str) {
        if let Some(substrate) = self.substrates.get_mut(name) {
            substrate.record_success();
        }
    }

    /// Take health snapshot
    pub fn snapshot(&mut self) -> HealthSnapshot {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let substrate_health: Vec<SubstrateHealth> = self.substrates.values().cloned().collect();
        let global_vitality = VitalityScore::from_substrates(&self.substrates);

        let global_status = if substrate_health
            .iter()
            .any(|s| s.status == HealthStatus::Critical)
        {
            HealthStatus::Critical
        } else if substrate_health
            .iter()
            .any(|s| s.status == HealthStatus::Degraded)
        {
            HealthStatus::Degraded
        } else if substrate_health
            .iter()
            .all(|s| s.status == HealthStatus::Healthy)
        {
            HealthStatus::Healthy
        } else {
            HealthStatus::Recovering
        };

        let snapshot = HealthSnapshot {
            timestamp_nanos: now,
            global_status,
            global_vitality,
            substrate_health,
            active_alerts: Vec::new(),    // Populated by alert system
            recovery_actions: Vec::new(), // Populated by recovery system
        };

        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(snapshot.clone());

        snapshot
    }

    /// Get latest snapshot
    pub fn latest(&self) -> Option<&HealthSnapshot> {
        self.history.last()
    }

    /// Check if any substrate is critical
    pub fn is_critical(&self) -> bool {
        self.substrates
            .values()
            .any(|s| s.status == HealthStatus::Critical)
    }

    /// Get substrates needing attention
    pub fn degraded_substrates(&self) -> Vec<&SubstrateHealth> {
        self.substrates
            .values()
            .filter(|s| s.status == HealthStatus::Degraded || s.status == HealthStatus::Critical)
            .collect()
    }

    /// Health monitor statistics
    pub fn stats(&self) -> MonitorStats {
        MonitorStats {
            substrate_count: self.substrates.len() as u64,
            total_alerts: self.alert_counter,
            total_actions: self.action_counter,
            snapshot_count: self.history.len() as u64,
            global_vitality: self
                .latest()
                .map(|s| s.global_vitality.overall)
                .unwrap_or(1.0),
            critical_count: self
                .substrates
                .values()
                .filter(|s| s.status == HealthStatus::Critical)
                .count() as u64,
        }
    }
}

/// MonitorStats: Observability of the health monitor itself
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorStats {
    pub substrate_count: u64,
    pub total_alerts: u64,
    pub total_actions: u64,
    pub snapshot_count: u64,
    pub global_vitality: f64,
    pub critical_count: u64,
}
