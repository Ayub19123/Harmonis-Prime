//! M2.7: Distributed Memory Layer
//! Clause Registry and DHT mesh for epistemic knowledge sharing.

pub mod dht;
pub mod packet;
pub mod proof;
pub mod provenance; // M2.7.3: Clause birth certificates with BLAKE3
pub mod registry;
pub mod scoring; // M2.7.4: Quality scoring engine

pub use packet::LitPack;
pub use proof::{EpistemicMeta, EpistemicProofTrace, ProofEntry};
pub use provenance::ClauseProvenance;
pub use registry::{ClauseRegistry, RegistryStats, ScoredClause};
pub use scoring::{eviction_cutoff, mean_score, ClauseScore, ScoringParams};
