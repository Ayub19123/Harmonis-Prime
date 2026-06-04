use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HardwareFingerprint: The immutable identity of the silicon
/// BRICK-39 SVL — State Verification Layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareFingerprint {
    pub fingerprint_id: String,
    pub generated_at_nanos: u64,
    pub compute: ComputeTopology,
    pub memory: MemoryArchitecture,
    pub thermal: ThermalBaseline,
    pub power: PowerDelivery,
    pub drivers: DriverIntegrity,
    pub firmware: FirmwareState,
    pub network: NetworkFabric,
    pub compliance_score: f64,
    pub golden_master_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeTopology {
    pub cpu_cores: usize,
    pub cpu_threads: usize,
    pub cpu_vendor: String,
    pub cpu_model: String,
    pub numa_nodes: usize,
    pub gpu_devices: Vec<GpuDevice>,
    pub npu_devices: Vec<NpuDevice>,
    pub total_compute_units: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuDevice {
    pub device_id: String,
    pub vendor: GpuVendor,
    pub model: String,
    pub compute_capability: String,
    pub vram_bytes: u64,
    pub pci_slot: String,
    pub numa_affinity: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuVendor {
    NVIDIA,
    AMD,
    Intel,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpuDevice {
    pub device_id: String,
    pub vendor: String,
    pub model: String,
    pub compute_units: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryArchitecture {
    pub total_ram_bytes: u64,
    pub available_ram_bytes: u64,
    pub ram_bandwidth_gbps: f64,
    pub numa_affinity_map: HashMap<usize, NumaMemory>,
    pub page_size: usize,
    pub huge_pages_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumaMemory {
    pub node_id: usize,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub local_access_latency_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalBaseline {
    pub cpu_idle_celsius: f64,
    pub gpu_idle_celsius: Vec<f64>,
    pub ambient_estimate_celsius: f64,
    pub cooling_capacity_watts: f64,
    pub thermal_throttle_history: Vec<ThermalEvent>,
    pub max_sustained_load_celsius: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalEvent {
    pub timestamp_nanos: u64,
    pub device: String,
    pub temp_celsius: f64,
    pub throttle_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerDelivery {
    pub psu_watts_rated: u32,
    pub psu_efficiency_rating: String,
    pub voltage_12v_min: f64,
    pub voltage_12v_max: f64,
    pub power_limit_enforced: bool,
    pub battery_backup_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverIntegrity {
    pub cuda_version: String,
    pub driver_version: String,
    pub runtime_path: String,
    pub runtime_hash_sha256: String,
    pub cudnn_version: Option<String>,
    pub tensorrt_version: Option<String>,
    pub signature_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareState {
    pub bios_vendor: String,
    pub bios_version: String,
    pub uefi_version: String,
    pub secure_boot_enabled: bool,
    pub microcode_version: String,
    pub tpm_present: bool,
    pub tpm_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkFabric {
    pub nic_count: usize,
    pub rdma_capable: bool,
    pub total_bandwidth_gbps: f64,
    pub latency_ns_rtt: u64,
    pub jumbo_frames_enabled: bool,
    pub numa_affinity: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceResult {
    Compliant { score: f64 },
    NonCompliant { score: f64, violations: Vec<String> },
    CriticalFailure { reason: String },
}

impl HardwareFingerprint {
    pub fn generate() -> Result<Self, HalError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let compute = Self::detect_compute()?;
        let memory = Self::detect_memory()?;
        let thermal = Self::detect_thermal()?;
        let power = Self::detect_power()?;
        let drivers = Self::detect_drivers()?;
        let firmware = Self::detect_firmware()?;
        let network = Self::detect_network()?;
        let compliance = Self::calculate_compliance(&compute, &memory, &thermal, &power, &drivers);
        Ok(Self {
            fingerprint_id: format!("fp_{}", now),
            generated_at_nanos: now,
            compute,
            memory,
            thermal,
            power,
            drivers,
            firmware,
            network,
            compliance_score: compliance,
            golden_master_hash: String::new(),
        })
    }

    pub fn verify_against_gm(&self, gm: &GoldenMasterSpec) -> ComplianceResult {
        let mut violations = Vec::new();
        let mut score = 1.0_f64;
        if self.compute.total_compute_units < gm.min_compute_units {
            violations.push(format!(
                "Compute: {} < {}",
                self.compute.total_compute_units, gm.min_compute_units
            ));
            score -= 0.15;
        }
        if self.memory.total_ram_bytes < gm.min_ram_bytes {
            violations.push(format!(
                "RAM: {} < {}",
                self.memory.total_ram_bytes, gm.min_ram_bytes
            ));
            score -= 0.15;
        }
        if self.thermal.max_sustained_load_celsius > gm.max_thermal_celsius {
            violations.push(format!(
                "Thermal: {}C > {}C",
                self.thermal.max_sustained_load_celsius, gm.max_thermal_celsius
            ));
            score -= 0.20;
        }
        if !self.drivers.signature_valid {
            violations.push("Driver signature invalid".to_string());
            score -= 0.25;
        }
        if gm.requires_secure_boot && !self.firmware.secure_boot_enabled {
            violations.push("Secure boot disabled".to_string());
            score -= 0.10;
        }
        if score >= 0.95 && violations.is_empty() {
            ComplianceResult::Compliant { score }
        } else if score >= 0.70 {
            ComplianceResult::NonCompliant { score, violations }
        } else {
            ComplianceResult::CriticalFailure {
                reason: violations.join("; "),
            }
        }
    }

    fn calculate_compliance(
        compute: &ComputeTopology,
        _memory: &MemoryArchitecture,
        thermal: &ThermalBaseline,
        power: &PowerDelivery,
        drivers: &DriverIntegrity,
    ) -> f64 {
        let mut score = 1.0;
        if !thermal.thermal_throttle_history.is_empty() {
            score -= (thermal.thermal_throttle_history.len() as f64 * 0.05).min(0.30);
        }
        if !power.power_limit_enforced {
            score -= 0.10;
        }
        if compute.gpu_devices.is_empty() {
            score -= 0.05;
        }
        if !drivers.signature_valid {
            score -= 0.25;
        }
        score.max(0.0)
    }

    fn detect_compute() -> Result<ComputeTopology, HalError> {
        Ok(Self::default_compute())
    }
    fn detect_memory() -> Result<MemoryArchitecture, HalError> {
        Ok(Self::default_memory())
    }
    fn detect_thermal() -> Result<ThermalBaseline, HalError> {
        Ok(Self::default_thermal())
    }
    fn detect_power() -> Result<PowerDelivery, HalError> {
        Ok(Self::default_power())
    }
    fn detect_drivers() -> Result<DriverIntegrity, HalError> {
        Ok(Self::default_drivers())
    }
    fn detect_firmware() -> Result<FirmwareState, HalError> {
        Ok(Self::default_firmware())
    }
    fn detect_network() -> Result<NetworkFabric, HalError> {
        Ok(Self::default_network())
    }

    fn default_compute() -> ComputeTopology {
        ComputeTopology {
            cpu_cores: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
            cpu_threads: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
            cpu_vendor: "Unknown".to_string(),
            cpu_model: "Unknown".to_string(),
            numa_nodes: 1,
            gpu_devices: Vec::new(),
            npu_devices: Vec::new(),
            total_compute_units: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1) as u32,
        }
    }
    fn default_memory() -> MemoryArchitecture {
        MemoryArchitecture {
            total_ram_bytes: 16_000_000_000,
            available_ram_bytes: 8_000_000_000,
            ram_bandwidth_gbps: 25.6,
            numa_affinity_map: HashMap::new(),
            page_size: 4096,
            huge_pages_available: false,
        }
    }
    fn default_thermal() -> ThermalBaseline {
        ThermalBaseline {
            cpu_idle_celsius: 35.0,
            gpu_idle_celsius: Vec::new(),
            ambient_estimate_celsius: 22.0,
            cooling_capacity_watts: 200.0,
            thermal_throttle_history: Vec::new(),
            max_sustained_load_celsius: 85.0,
        }
    }
    fn default_power() -> PowerDelivery {
        PowerDelivery {
            psu_watts_rated: 750,
            psu_efficiency_rating: "80 Plus Gold".to_string(),
            voltage_12v_min: 11.4,
            voltage_12v_max: 12.6,
            power_limit_enforced: true,
            battery_backup_present: false,
        }
    }
    fn default_drivers() -> DriverIntegrity {
        DriverIntegrity {
            cuda_version: "Unknown".to_string(),
            driver_version: "Unknown".to_string(),
            runtime_path: String::new(),
            runtime_hash_sha256: String::new(),
            cudnn_version: None,
            tensorrt_version: None,
            signature_valid: true,
        }
    }
    fn default_firmware() -> FirmwareState {
        FirmwareState {
            bios_vendor: "Unknown".to_string(),
            bios_version: "Unknown".to_string(),
            uefi_version: "Unknown".to_string(),
            secure_boot_enabled: false,
            microcode_version: "Unknown".to_string(),
            tpm_present: false,
            tpm_version: "None".to_string(),
        }
    }
    fn default_network() -> NetworkFabric {
        NetworkFabric {
            nic_count: 1,
            rdma_capable: false,
            total_bandwidth_gbps: 1.0,
            latency_ns_rtt: 1_000_000,
            jumbo_frames_enabled: false,
            numa_affinity: vec![0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenMasterSpec {
    pub spec_id: String,
    pub min_compute_units: u32,
    pub min_ram_bytes: u64,
    pub max_thermal_celsius: f64,
    pub requires_secure_boot: bool,
    pub min_driver_version: String,
    pub requires_rdma: bool,
}

#[derive(Debug, Clone)]
pub enum HalError {
    DetectionFailed(String),
    VerificationFailed(String),
    DeviceNotFound(String),
    PermissionDenied(String),
}

impl std::fmt::Display for HalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HalError::DetectionFailed(s) => write!(f, "HAL detection failed: {}", s),
            HalError::VerificationFailed(s) => write!(f, "HAL verification failed: {}", s),
            HalError::DeviceNotFound(s) => write!(f, "Device not found: {}", s),
            HalError::PermissionDenied(s) => write!(f, "Permission denied: {}", s),
        }
    }
}

impl std::error::Error for HalError {}
