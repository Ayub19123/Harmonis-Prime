use std::sync::atomic::AtomicU64;
use std::sync::Arc;

/// Physical node abstraction for air-gapped cluster simulation
#[derive(Debug)]
pub struct PhysicalNode {
    pub id: u64,
    pub state: Arc<AtomicU64>,
    network_interface_active: bool,
}

impl PhysicalNode {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            state: Arc::new(AtomicU64::new(0)),
            network_interface_active: false,
        }
    }

    /// Check if network interface is active
    pub fn is_network_active(&self) -> bool {
        self.network_interface_active
    }

    /// Activate network interface (for testing only)
    pub fn activate_network(&mut self) {
        self.network_interface_active = true;
    }

    /// Deactivate network interface (simulates partition)
    pub fn deactivate_network(&mut self) {
        self.network_interface_active = false;
    }
}
