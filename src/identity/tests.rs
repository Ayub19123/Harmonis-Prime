//! SET-6C: Identity Invariant Tests
//! Consolidated tests for PUF, NIST SP 800-22, and challenge-response authentication.

use crate::identity::puf::{SramPuf, HardwareFingerprint, hamming_distance};
use crate::identity::nist::{run_nist_battery, P_VALUE_THRESHOLD};
use crate::identity::auth::{ChallengeResponse, NonceWindow, ReplayError};

// --- PUF Invariants ---

#[test]
fn test_puf_deterministic_per_node() {
    let salt = [0u8; 32];
    let fp = HardwareFingerprint::from_node_id("node-alpha", &salt);
    let puf = SramPuf::new(fp);

    let challenge = [1u8; 32];
    let r1 = puf.generate(&challenge);
    let r2 = puf.generate(&challenge);

    assert_eq!(r1.bits, r2.bits, "Same node + challenge must produce identical response");
    assert!(r1.stability_score > 0.85, "Stability must exceed threshold");
}

#[test]
fn test_puf_unique_across_nodes() {
    let salt = [0u8; 32];
    let fp1 = HardwareFingerprint::from_node_id("node-alpha", &salt);
    let fp2 = HardwareFingerprint::from_node_id("node-beta", &salt);

    let puf1 = SramPuf::new(fp1);
    let puf2 = SramPuf::new(fp2);

    let challenge = [1u8; 32];
    let r1 = puf1.generate(&challenge);
    let r2 = puf2.generate(&challenge);

    let hd = hamming_distance(&r1.bits, &r2.bits);
    assert!(hd > 0.40, "Inter-chip Hamming distance must exceed 40%, got {}", hd);
    assert!(hd < 0.60, "Inter-chip Hamming distance must be below 60%, got {}", hd);
}

#[test]
fn test_key_extraction_requires_stability() {
    let salt = [0u8; 32];
    let fp = HardwareFingerprint::from_node_id("node-alpha", &salt);
    let puf = SramPuf::new(fp);

    let challenge = [1u8; 32];
    let response = puf.generate(&challenge);
    assert!(response.stability_score > 0.85);

    let key = puf.extract_key(&response);
    assert!(key.is_ok(), "Stable response must yield extractable key");
}

// --- NIST SP 800-22 Tests ---

#[test]
fn test_nist_monobit() {
    let salt = [0u8; 32];
    let fp = HardwareFingerprint::from_node_id("nist-test", &salt);
    let puf = SramPuf::new(fp);

    let mut data = Vec::new();
    for i in 0..100 {
        let challenge = i.to_le_bytes();
        let mut full = [0u8; 32];
        full[..challenge.len()].copy_from_slice(&challenge);
        let resp = puf.generate(&full);
        data.extend_from_slice(&resp.bits);
    }

    let results = run_nist_battery(&data);
    for r in results {
        assert!(r.passed, "NIST test {} failed with p={}", r.name, r.p_value);
    }
}

// --- Authentication Tests ---

#[test]
fn test_nonce_monotonicity() {
    let mut window = NonceWindow::new();
    assert!(window.validate(1).is_ok());
    assert!(window.validate(2).is_ok());
    assert!(window.validate(3).is_ok());
    assert!(window.validate(2).is_err()); // Replay
}

#[test]
fn test_challenge_response_determinism() {
    let key = [0xABu8; 32];
    let auth = ChallengeResponse::new(key);

    let challenge = [1u8; 32];
    let nonce = 42u64;

    let r1 = auth.generate_response(&challenge, nonce);
    let r2 = auth.generate_response(&challenge, nonce);
    assert_eq!(r1, r2, "Same challenge+nonce+key must produce identical response");
}

#[test]
fn test_full_authentication_flow() {
    let key = [0xCDu8; 32];
    let mut verifier = ChallengeResponse::new(key);

    let challenge = [0xEFu8; 32];
    let nonce = 1u64;
    let response = verifier.generate_response(&challenge, nonce);

    assert!(verifier.verify(&challenge, nonce, &response).is_ok());
    assert!(matches!(
        verifier.verify(&challenge, nonce, &response),
        Err(ReplayError::NonceAlreadySeen)
    ));
}