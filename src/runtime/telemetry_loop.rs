use crate::governance::policy::GovernancePolicy;
use crate::hal::atomic_boot::HardwareBindings;
use crate::hal::fingerprint::HardwareFingerprint;
use crate::runtime::flow_runtime::{FlowRuntime, FlowState};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// TelemetryLoop: Real-time neural mapping at 1000 Hz
/// Bridges HAL raw data â†’ predictive GDO allocations
pub struct TelemetryLoop {
    #[allow(dead_code)]
    #[allow(dead_code)]
    fingerprint: HardwareFingerprint,
    #[allow(dead_code)]
    #[allow(dead_code)]
    bindings: HardwareBindings,
    #[allow(dead_code)]
    #[allow(dead_code)]
    policy: GovernancePolicy,

    sample_count: u64,
    last_sample_nanos: u64,
    running: bool,
}

#[derive(Debug, Clone)]
pub struct TelemetryFrame {
    pub timestamp_nanos: u64,
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub thermal_reading: f64,
    pub flow_state: FlowState,
    pub governance_action: String,
}

impl TelemetryLoop {
    pub fn new(fingerprint: HardwareFingerprint, bindings: HardwareBindings) -> Self {
        Self {
            fingerprint,
            bindings,
            #[allow(dead_code)]
            policy: GovernancePolicy::production(),
            sample_count: 0,
            last_sample_nanos: 0,
            running: false,
        }
    }

    /// Engage Telemetry Loop â€” start 1000 Hz diagnostic cycle
    pub fn engage(&mut self, flow: &Arc<Mutex<FlowRuntime>>) -> Vec<TelemetryFrame> {
        println!("TELEMETRY: Engaging 1000 Hz diagnostic cycle...");
        self.running = true;
        let mut frames = Vec::new();
        let start = Instant::now();

        // Execute 10 seconds of telemetry (10,000 samples at 1000 Hz)
        while self.running && start.elapsed() < Duration::from_secs(10) {
            let frame = self.sample(flow);
            frames.push(frame.clone());

            if self.sample_count % 1000 == 0 {
                println!(
                    "TELEMETRY: {} samples | flow: {:?} | cpu: {:.1}% | mem: {:.1}%",
                    self.sample_count,
                    frame.flow_state,
                    frame.cpu_utilization,
                    frame.memory_utilization
                );
            }

            // 1ms sleep = 1000 Hz
            thread::sleep(Duration::from_micros(1000));
        }

        self.running = false;
        println!(
            "TELEMETRY: Loop complete â€” {} samples captured",
            self.sample_count
        );
        frames
    }

    /// Single sample â€” HAL raw â†’ predictive allocation
    fn sample(&mut self, flow: &Arc<Mutex<FlowRuntime>>) -> TelemetryFrame {
        self.sample_count += 1;
        let now = Instant::now().elapsed().as_nanos() as u64;

        let flow_state = flow.lock().unwrap().state();

        // Simulated HAL readings (replace with real WMI/sysfs calls)
        let cpu_util = 15.0 + (self.sample_count as f64 % 30.0);
        let mem_util = 20.0 + (self.sample_count as f64 % 25.0);
        let thermal = 35.0 + (self.sample_count as f64 % 15.0);

        let action = if thermal > 80.0 {
            "THROTTLE: thermal guard".to_string()
        } else if mem_util > 85.0 {
            "THROTTLE: memory guard".to_string()
        } else {
            "PROCEED: all clear".to_string()
        };

        self.last_sample_nanos = now;

        TelemetryFrame {
            timestamp_nanos: now,
            cpu_utilization: cpu_util,
            memory_utilization: mem_util,
            thermal_reading: thermal,
            flow_state,
            governance_action: action,
        }
    }

    pub fn disengage(&mut self) {
        self.running = false;
        println!("TELEMETRY: Disengaged");
    }

    pub fn sample_count(&self) -> u64 {
        self.sample_count
    }
}
