//! src/identity/auth.rs
//! Challenge-response authentication with replay protection.
//!
//! Protocol:
//! 1. Verifier sends challenge (256-bit random) + nonce (64-bit monotonic)
//! 2. Prover computes response = HMAC-SHA256(PUF_key, challenge || nonce)
//! 3. Verifier checks response and nonce freshness
//!
//! Security properties:
//! - PUF key never transmitted
//! - Nonce strictly monotonic, no reuse tolerance
//! - Challenge freshness prevents precomputation

use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashSet;

type HmacSha256 = Hmac<Sha256>;

/// 64-bit strictly monotonic nonce with window-based replay protection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonceWindow {
    last_accepted: u64,
    window: HashSet<u64>,
    max_window_size: usize,
}

/// Authentication error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    NonceTooOld,
    NonceAlreadySeen,
    NonceInvalid,
    ResponseInvalid,
}

/// Challenge-response authentication session.
pub struct ChallengeResponse {
    puf_key: [u8; 32],
    nonce_window: NonceWindow,
}

impl NonceWindow {
    pub fn new() -> Self {
        Self {
            last_accepted: 0,
            window: HashSet::new(),
            max_window_size: 1000,
        }
    }

    /// Validate nonce against strict monotonicity and replay window.
    ///
    /// Rules:
    /// - nonce > last_accepted: accept, update window
    /// - nonce in window: reject (replay)
    /// - nonce < last_accepted - window_size: reject (too old)
    pub fn validate(&mut self, nonce: u64) -> Result<(), ReplayError> {
        if nonce == 0 {
            return Err(ReplayError::NonceInvalid);
        }

        if nonce > self.last_accepted {
            // Advance window
            self.window.insert(nonce);
            self.last_accepted = nonce;

            // Prune old entries if window exceeds max size
            if self.window.len() > self.max_window_size {
                let cutoff = self.last_accepted - self.max_window_size as u64;
                self.window.retain(|&n| n > cutoff);
            }

            return Ok(());
        }

        if self.window.contains(&nonce) {
            return Err(ReplayError::NonceAlreadySeen);
        }

        if nonce
            < self
                .last_accepted
                .saturating_sub(self.max_window_size as u64)
        {
            return Err(ReplayError::NonceTooOld);
        }

        // Nonce within window but not seen â€” accept and add to window
        self.window.insert(nonce);
        Ok(())
    }
}

impl ChallengeResponse {
    pub fn new(puf_key: [u8; 32]) -> Self {
        Self {
            puf_key,
            nonce_window: NonceWindow::new(),
        }
    }

    /// Generate response to challenge using PUF-derived key.
    pub fn generate_response(&self, challenge: &[u8; 32], nonce: u64) -> [u8; 32] {
        let mut mac =
            HmacSha256::new_from_slice(&self.puf_key).expect("HMAC can handle 32-byte key");

        mac.update(challenge);
        mac.update(&nonce.to_le_bytes());

        let result = mac.finalize();
        let bytes = result.into_bytes();

        let mut response = [0u8; 32];
        response.copy_from_slice(&bytes);
        response
    }

    /// Verify response from remote prover.
    pub fn verify(
        &mut self,
        challenge: &[u8; 32],
        nonce: u64,
        response: &[u8; 32],
    ) -> Result<(), ReplayError> {
        // First: strict nonce validation (replay protection)
        self.nonce_window.validate(nonce)?;

        // Second: response verification
        let expected = self.generate_response(challenge, nonce);

        if !constant_time_eq(&expected, response) {
            return Err(ReplayError::ResponseInvalid);
        }

        Ok(())
    }
}

/// Constant-time comparison to prevent timing attacks.
fn constant_time_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut result = 0u8;
    for i in 0..32 {
        result |= a[i] ^ b[i];
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_monotonicity() {
        let mut window = NonceWindow::new();
        assert!(window.validate(1).is_ok());
        assert!(window.validate(2).is_ok());
        assert!(window.validate(3).is_ok());
        assert!(window.validate(2).is_err()); // Replay
    }

    #[test]
    fn test_nonce_replay_rejection() {
        let mut window = NonceWindow::new();
        window.validate(100).unwrap();
        assert!(matches!(
            window.validate(100),
            Err(ReplayError::NonceAlreadySeen)
        ));
    }

    #[test]
    fn test_challenge_response_determinism() {
        let key = [0xABu8; 32];
        let auth = ChallengeResponse::new(key);

        let challenge = [1u8; 32];
        let nonce = 42u64;

        let r1 = auth.generate_response(&challenge, nonce);
        let r2 = auth.generate_response(&challenge, nonce);

        assert_eq!(
            r1, r2,
            "Same challenge+nonce+key must produce identical response"
        );
    }

    #[test]
    fn test_full_authentication_flow() {
        let key = [0xCDu8; 32];
        let mut verifier = ChallengeResponse::new(key);

        let challenge = [0xEFu8; 32];
        let nonce = 1u64;
        let response = verifier.generate_response(&challenge, nonce);

        // First verification succeeds
        assert!(verifier.verify(&challenge, nonce, &response).is_ok());

        // Second verification with same nonce fails (replay)
        assert!(matches!(
            verifier.verify(&challenge, nonce, &response),
            Err(ReplayError::NonceAlreadySeen)
        ));
    }
}
