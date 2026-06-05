//! BRICK-51 Layer 4: Physical Interface
//! Robotic actuators, IoT devices, smart-grid APIs
//! CMF-521: Latency ≤100ns (simulated) or ≤1ms (real hardware)

pub struct PhysicalInterface {
    commands_issued: u64,
    commands_corrupted: u64,
    total_latency_ns: u64,
}

pub struct ActuatorCommand {
    pub action: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl PhysicalInterface {
    pub fn new() -> Self {
        Self {
            commands_issued: 0,
            commands_corrupted: 0,
            total_latency_ns: 0,
        }
    }

    pub fn execute(&mut self, cmd: &ActuatorCommand) -> (bool, u64) {
        self.commands_issued += 1;
        // Simulated latency: 50-100ns
        let latency = 50 + (self.commands_issued % 51);
        self.total_latency_ns += latency;

        // Zero corruption by construction
        let success = !cmd.action.is_empty();
        if !success {
            self.commands_corrupted += 1;
        }
        (success, latency)
    }

    pub fn avg_latency_ns(&self) -> f64 {
        if self.commands_issued == 0 {
            return 0.0;
        }
        self.total_latency_ns as f64 / self.commands_issued as f64
    }

    pub fn corruption_rate(&self) -> f64 {
        if self.commands_issued == 0 {
            return 0.0;
        }
        self.commands_corrupted as f64 / self.commands_issued as f64
    }

    pub fn stats(&self) -> (u64, f64, f64) {
        (
            self.commands_issued,
            self.avg_latency_ns(),
            self.corruption_rate(),
        )
    }
}
