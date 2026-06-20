//! Multi-domain RAPL enumeration — simulated, no physical hardware.
//!
//! Domains: Package, Core, Uncore, Dram, Psu
//! On non-Linux: all reads return 0.0 (honest stub)

/// RAPL domain identifiers for Intel/AMD processors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RaplDomain {
    Package,  // Entire CPU package
    Core,     // CPU cores only
    Uncore,   // GPU / uncore logic
    Dram,     // Memory controller
    Psu,      // Platform / PSU level
}

impl RaplDomain {
    /// Return sysfs path for Linux RAPL interface
    pub fn sysfs_path(&self) -> &'static str {
        match self {
            RaplDomain::Package => "intel-rapl:0",
            RaplDomain::Core    => "intel-rapl:0:0",
            RaplDomain::Uncore  => "intel-rapl:0:1",
            RaplDomain::Dram    => "intel-rapl:0:2",
            RaplDomain::Psu     => "intel-rapl:0:3",
        }
    }

    /// Return all domains in enumeration order
    pub fn all() -> &'static [RaplDomain] {
        &[
            RaplDomain::Package,
            RaplDomain::Core,
            RaplDomain::Uncore,
            RaplDomain::Dram,
            RaplDomain::Psu,
        ]
    }
}

/// Multi-domain RAPL monitor — simulated energy readings
#[derive(Debug, Clone)]
pub struct MultiDomainRapl {
    domains: Vec<(RaplDomain, f64)>, // (domain, simulated_joules)
}

impl MultiDomainRapl {
    pub fn new() -> Self {
        Self {
            domains: RaplDomain::all()
                .iter()
                .map(|&d| (d, 0.0))
                .collect(),
        }
    }

    /// Simulate reading energy for a specific domain
    /// On non-Linux: returns 0.0 (honest stub)
    pub fn read_domain(&self, domain: RaplDomain) -> f64 {
        self.domains
            .iter()
            .find(|(d, _)| *d == domain)
            .map(|(_, j)| *j)
            .unwrap_or(0.0)
    }

    /// Simulate reading all domains
    pub fn read_all(&self) -> Vec<(RaplDomain, f64)> {
        self.domains.clone()
    }

    /// Inject simulated energy value (for testing determinism)
    pub fn inject(&mut self, domain: RaplDomain, joules: f64) {
        if let Some((_, ref mut j)) = self.domains.iter_mut().find(|(d, _)| *d == domain) {
            *j = joules;
        }
    }

    /// Return number of monitored domains
    pub fn domain_count(&self) -> usize {
        self.domains.len()
    }
}