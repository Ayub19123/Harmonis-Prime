//! BRICK-51.1: Hardware Classification Tag
//! Every latency/performance claim must carry its measurement domain

#[derive(Clone, Debug, PartialEq)]
pub enum HardwareDomain {
    Simulated,        // Software-only timing, no physical hardware
    Emulated,         // Hardware model in simulation (FPGA, QEMU)
    PhysicalMeasured, // Real instrumented measurement
}

impl HardwareDomain {
    pub fn label(&self) -> &'static str {
        match self {
            HardwareDomain::Simulated => "SIMULATED",
            HardwareDomain::Emulated => "EMULATED",
            HardwareDomain::PhysicalMeasured => "PHYSICAL_MEASURED",
        }
    }
}

#[derive(Clone, Debug)]
pub struct TaggedMetric {
    pub value: f64,
    pub unit: String,
    pub domain: HardwareDomain,
    pub runs: u64,
    pub std_dev: f64,
    pub seed: u64,
}

impl TaggedMetric {
    pub fn new(
        value: f64,
        unit: &str,
        domain: HardwareDomain,
        runs: u64,
        std_dev: f64,
        seed: u64,
    ) -> Self {
        Self {
            value,
            unit: unit.to_string(),
            domain,
            runs,
            std_dev,
            seed,
        }
    }

    pub fn report(&self) -> String {
        format!(
            "[{}] {:.2} {} ± {:.2} (n={}, seed={})",
            self.domain.label(),
            self.value,
            self.unit,
            self.std_dev,
            self.runs,
            self.seed
        )
    }
}
