//! SET-5.3: Telemetry Stream

use std::time::Instant;

#[derive(Debug, Clone)]
pub struct TelemetryRecord {
    pub timestamp: Instant,
    pub operation_id: u64,
    pub latency_micros: u64,
    pub energy_joules: f64,
    pub success: bool,
}

#[derive(Debug)]
pub struct TelemetryStream {
    records: Vec<TelemetryRecord>,
    max_records: usize,
    total_operations: u64,
    total_latency_micros: u64,
    total_energy_joules: f64,
}

impl TelemetryStream {
    pub fn with_capacity(max_records: usize) -> Self {
        Self {
            records: Vec::with_capacity(max_records),
            max_records,
            total_operations: 0,
            total_latency_micros: 0,
            total_energy_joules: 0.0,
        }
    }

    pub fn record(&mut self, operation_id: u64, latency_micros: u64, energy_joules: f64, success: bool) {
        let record = TelemetryRecord {
            timestamp: Instant::now(),
            operation_id,
            latency_micros,
            energy_joules,
            success,
        };
        if self.records.len() >= self.max_records {
            self.records.remove(0);
        }
        self.records.push(record);
        self.total_operations += 1;
        self.total_latency_micros += latency_micros;
        self.total_energy_joules += energy_joules;
    }

    pub fn avg_latency_micros(&self) -> f64 {
        if self.total_operations == 0 { 0.0 } else { self.total_latency_micros as f64 / self.total_operations as f64 }
    }

    pub fn success_rate(&self) -> f64 {
        if self.records.is_empty() { 1.0 } else { self.records.iter().filter(|r| r.success).count() as f64 / self.records.len() as f64 }
    }
}
