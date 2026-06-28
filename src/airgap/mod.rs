//! SET-6A: Software-Defined Air-Gap Cluster
//! Invariant: Sovereignty index = 1.0 (zero external API calls)
//! Invariant: Partition halts safely (Raft leader steps down)
//! Invariant: Reconnect converges (log consistency restored)

pub mod entropy;
pub mod firewall;
pub mod mesh;
pub mod node;

#[cfg(test)]
mod tests;
