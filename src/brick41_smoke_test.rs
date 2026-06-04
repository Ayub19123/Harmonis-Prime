
// BRICK-41 Phase 1: Smoke Test — Foundation Behavior Verification
// This test proves TrustLayer, MemoryStore, Ledger, and SecurityBaseline
// are not just compiled — they execute correctly on silicon.

use sovereign_core::brick41::foundation::{
    TrustLayer, AuditEntry, TrustVerification,
    MemoryStore, MemoryNode, MemoryEdge, RetrievalResult,
    Ledger, LedgerValue, ConsensusProposal, CommitStatus,
    SecurityBaseline, SecurityContext, SecurityDecision, SecurityLevel,
};

#[test]
fn test_trust_layer_append_and_verify() {
    let mut trust = TrustLayer::new();
    
    // Append 5 audit entries
    for i in 0..5 {
        trust.append("pilot_ayub", &format!("action_{}", i), &format!("context_{}", i));
    }
    
    assert_eq!(trust.len(), 6); // genesis + 5 entries
    
    let verification = trust.verify();
    assert!(verification.valid, "Trust chain should be valid");
    assert_eq!(verification.chain_length, 6);
    assert_eq!(verification.integrity_score, 1.0);
    assert!(verification.tampered_indices.is_empty());
    
    println!("✅ TrustLayer: {} entries, integrity {:.2}%", verification.chain_length, verification.integrity_score * 100.0);
}

#[test]
fn test_memory_store_crud() {
    let mut store = MemoryStore::new();
    
    // Store nodes across domains
    store.store("node_1", "finance", "Q4 revenue projection", vec![0.1, 0.2, 0.3], 0.95);
    store.store("node_2", "healthcare", "patient diagnosis protocol", vec![0.4, 0.5, 0.6], 0.88);
    store.store("node_3", "finance", "risk assessment model", vec![0.7, 0.8, 0.9], 0.92);
    
    // Link nodes
    store.link("node_1", "node_3", "depends_on", 0.85, true);
    
    // Retrieve by similarity
    let results = store.retrieve(&[0.15, 0.25, 0.35], Some("finance"), 2);
    assert!(!results.is_empty(), "Should retrieve finance nodes");
    assert_eq!(results[0].node.domain, "finance");
    
    // Traverse graph
    let traversed = store.traverse("node_1", 2);
    assert!(traversed.len() >= 2, "Should traverse at least 2 nodes");
    
    println!("✅ MemoryStore: {} nodes stored, {} traversed", 3, traversed.len());
}

#[test]
fn test_ledger_consensus() {
    let mut ledger = Ledger::new("node_alpha", 2);
    ledger.add_replica("node_beta");
    ledger.add_replica("node_gamma");
    
    // Propose a value
    let proposal = ledger.propose("config_key", "sovereign_config_v1");
    assert_eq!(proposal.sequence, 1);
    assert_eq!(proposal.value.status, CommitStatus::Proposed);
    
    // Simulate prepare with replica signatures
    let mut prepared = proposal.clone();
    let ready = ledger.prepare(&mut prepared, vec!["node_beta".to_string()]);
    assert!(ready, "Should reach quorum with 2 signatures");
    
    // Commit
    let committed = ledger.commit(prepared);
    assert!(committed, "Commit should succeed");
    
    // Verify retrieval
    let value = ledger.get("config_key");
    assert!(value.is_some());
    assert_eq!(value.unwrap().status, CommitStatus::Committed);
    
    // Verify consensus
    let audit = ledger.audit_trail();
    assert!(!audit.is_empty(), "Audit trail should exist");
    
    println!("✅ Ledger: consensus reached, {} audit entries", audit.len());
}

#[test]
fn test_security_baseline_enforcement() {
    let baseline = SecurityBaseline::production();
    
    // Test permitted access
    let permit = baseline.evaluate("pilot_ayub", "read", "finance", &SecurityLevel::Confidential);
    match permit {
        SecurityDecision::Permit { context, audit } => {
            assert_eq!(context.domain, "finance");
            assert_eq!(context.level, SecurityLevel::Confidential);
            println!("✅ Security: PERMIT — {}", audit);
        }
        _ => panic!("Should permit read on finance"),
    }
    
    // Test denied access (unknown domain)
    let deny = baseline.evaluate("pilot_ayub", "read", "unknown_domain", &SecurityLevel::Public);
    match deny {
        SecurityDecision::Deny { reason, .. } => {
            assert!(reason.contains("not in allowed set"));
            println!("✅ Security: DENY — {}", reason);
        }
        _ => panic!("Should deny unknown domain"),
    }
    
    // Test escalation (classification too high)
    let escalate = baseline.evaluate("pilot_ayub", "govern", "finance", &SecurityLevel::Sovereign);
    match escalate {
        SecurityDecision::Escalate { required_level, .. } => {
            assert_eq!(required_level, SecurityLevel::Sovereign);
            println!("✅ Security: ESCALATE — requires {:?}", required_level);
        }
        _ => panic!("Should escalate sovereign classification"),
    }
    
    // Test zero-trust verification
    let ctx = SecurityContext {
        identity: "pilot_ayub".to_string(),
        level: SecurityLevel::Internal,
        domain: "healthcare".to_string(),
        session_nonce: "test_nonce".to_string(),
        capabilities: vec!["read".to_string(), "write".to_string()],
    };
    assert!(baseline.verify_zero_trust(&ctx));
}

#[test]
fn test_foundation_integration_sequence() {
    // This test simulates the full BRICK-41 Phase 1 integration:
    // Security evaluates → Memory stores → Ledger commits → Trust audits
    
    let baseline = SecurityBaseline::production();
    let mut trust = TrustLayer::new();
    let mut ledger = Ledger::new("integration_node", 1);
    let mut memory = MemoryStore::new();
    
    // Step 1: Security evaluation
    let decision = baseline.evaluate("system", "execute", "finance", &SecurityLevel::Internal);
    let permitted = matches!(decision, SecurityDecision::Permit { .. });
    assert!(permitted, "Integration test should permit finance execution");
    
    trust.append("security", "evaluate", "finance_permitted");
    
    // Step 2: Memory storage
    memory.store("finance_exec_1", "finance", "executed trade batch A", vec![0.1, 0.2, 0.3], 0.99);
    trust.append("memory", "store", "finance_exec_1");
    
    // Step 3: Ledger commit
    let proposal = ledger.propose("finance_exec_1", "completed");
    let mut prepared = proposal.clone();
    ledger.prepare(&mut prepared, vec![]); // quorum=1, no extra sigs needed
    ledger.commit(prepared);
    trust.append("ledger", "commit", "finance_exec_1");
    
    // Step 4: Verify full chain
    let verification = trust.verify();
    assert!(verification.valid);
    assert_eq!(verification.chain_length, 4); // genesis + 3 operations
    
    println!("✅ INTEGRATION: Full Phase 1 pipeline verified — {} trust entries", verification.chain_length);
}
