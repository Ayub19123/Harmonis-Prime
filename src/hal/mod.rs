// BRICK-39: Hardware Abstraction Layer (HAL)
// Exports the hardware fingerprinting, verification, and atomic boot systems

pub mod atomic_boot;
pub mod fingerprint;

// Re-export core types for easier access
pub use fingerprint::{
    ComplianceResult, ComputeTopology, DriverIntegrity, FirmwareState, GoldenMasterSpec, GpuDevice,
    GpuVendor, HalError, HardwareFingerprint, MemoryArchitecture, NetworkFabric, NpuDevice,
    NumaMemory, PowerDelivery, ThermalBaseline, ThermalEvent,
};

pub use atomic_boot::{
    boot_harmonis, AtomicBootSequence, BootOutcome, GpuBinding, HardwareBindings, TelemetryHandle,
};
