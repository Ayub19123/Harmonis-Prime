// BRICK-41: The Holy Grail — Global Beta Network
// Phase 1: Foundation Layer — Trust + Memory + Ledger + Security

pub mod foundation;

pub use foundation::{
    AuditEntry, CommitStatus, ConsensusProposal, Ledger, LedgerValue, MemoryEdge, MemoryNode,
    MemoryStore, RetrievalResult, SecurityBaseline, SecurityContext, SecurityDecision,
    SecurityLevel, TrustLayer, TrustVerification,
};

pub mod orchestration;

// pub mod intelligence; // DISABLED - Phase 3 under revision
