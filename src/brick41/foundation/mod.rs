// BRICK-41 Phase 1: Foundation Layer
// Trust + Memory + Ledger + Security = Immutable Bedrock

pub mod ledger;
pub mod memory_store;
pub mod security;
pub mod trust_layer;

pub use ledger::{CommitStatus, ConsensusProposal, Ledger, LedgerValue};
pub use memory_store::{MemoryEdge, MemoryNode, MemoryStore, RetrievalResult};
pub use security::{SecurityBaseline, SecurityContext, SecurityDecision, SecurityLevel};
pub use trust_layer::{AuditEntry, TrustLayer, TrustVerification};
