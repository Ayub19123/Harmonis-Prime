//! M2.7: Distributed Memory Layer
//! Clause Registry and DHT mesh for epistemic knowledge sharing.

pub mod dht;
pub mod packet;
pub mod registry;

pub use packet::LitPack;
pub use registry::{ClauseRegistry, FilterConfig, RegistryStats};