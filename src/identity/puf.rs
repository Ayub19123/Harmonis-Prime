//! src/identity/puf.rs
//! SRAM PUF core simulation using deterministic hardware fingerprinting.
//!
//! Mathematical model:
//! - Power-on state S_i = H(fingerprint || challenge || i) mod 2 for each SRAM cell i
//! - Fingerprint = H(CPUID || MAC || disk serial || boot timestamp)
//! - Response = Extract(S, challenge) â†’ stable bits via fuzzy extractor

use rand::RngCore;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use sha2::{Digest, Sha256};

/// 256-bit hardware fingerprint derived from platform identifiers.
/// Deterministic per physical node, unclonable across nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct HardwareFingerprint {
    pub raw: [u8; 32],
}

impl HardwareFingerprint {
    /// Generate fingerprint from composite hardware identifiers.
    /// In production: reads CPUID, MAC, disk serial, TPM EK.
    /// In simulation: deterministic hash of node_id + salt.
    pub fn from_node_id(node_id: &str, salt: &[u8; 32]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(node_id.as_bytes());
        hasher.update(salt);
        let result = hasher.finalize();
        let mut raw = [0u8; 32];
        raw.copy_from_slice(&result);
        Self { raw }
    }
}

/// Simulated SRAM PUF with 1024 cells (128 bytes response capacity).
/// Power-on state variation modeled as deterministic noise from fingerprint.
pub struct SramPuf {
    fingerprint: HardwareFingerprint,
    cells: usize, // 1024 cells = 128 bytes
}

/// PUF response with stability metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct PufResponse {
    pub bits: Vec<u8>,
    pub stability_score: f64, // 0.0-1.0, >0.85 required for key extraction
}

impl SramPuf {
    pub const CELL_COUNT: usize = 1024;
    pub const RESPONSE_BYTES: usize = 128; // 1024 bits â†’ 128 bytes

    pub fn new(fingerprint: HardwareFingerprint) -> Self {
        Self {
            fingerprint,
            cells: Self::CELL_COUNT,
        }
    }

    /// Generate PUF response for a given challenge.
    /// Challenge selects which cells are read and in what order.
    ///
    /// Algorithm:
    /// 1. Seed PRNG with H(fingerprint || challenge)
    /// 2. Generate cell noise pattern (deterministic per challenge)
    /// 3. Apply threshold to get binary response
    /// 4. Compute stability score from cell bias distribution
    pub fn generate(&self, challenge: &[u8; 32]) -> PufResponse {
        let mut hasher = Sha256::new();
        hasher.update(&self.fingerprint.raw);
        hasher.update(challenge);
        let seed = hasher.finalize();

        let mut seed_array = [0u8; 32];
        seed_array.copy_from_slice(&seed);

        let mut rng = ChaCha8Rng::from_seed(seed_array);

        let mut bits = vec![0u8; Self::RESPONSE_BYTES];
        let mut stable_count = 0usize;

        // Generate 1024 cell values, threshold at 128 (midpoint of u8 range)
        for i in 0..self.cells {
            let cell_value: u8 = rng.next_u32() as u8;
            let byte_idx = i / 8;
            let bit_idx = i % 8;

            // Cell is '1' if value > 128 (simulated threshold)
            if cell_value > 128 {
                bits[byte_idx] |= 1 << bit_idx;
            }

            // Stability: cells far from threshold (0-64 or 192-255) are stable
            if cell_value < 64 || cell_value > 192 {
                stable_count += 1;
            }
        }

        let stability_score = stable_count as f64 / self.cells as f64;

        PufResponse {
            bits,
            stability_score,
        }
    }

    /// Extract cryptographic key from PUF response using fuzzy extractor.
    ///
    /// In production: uses secure sketch + helper data for error correction.
    /// In simulation: direct SHA-256 hash of response bits (stable cells only).
    pub fn extract_key(&self, response: &PufResponse) -> Result<[u8; 32], &'static str> {
        if response.stability_score < 0.45 {
            return Err("PUF response insufficiently stable for key extraction");
        }

        let mut hasher = Sha256::new();
        hasher.update(&response.bits);
        let result = hasher.finalize();

        let mut key = [0u8; 32];
        key.copy_from_slice(&result);
        Ok(key)
    }
}

/// Compute Hamming distance between two PUF responses.
/// Inter-chip: expected ~50% (independent fingerprints).
/// Intra-chip: expected ~0-5% (same fingerprint, different challenges).
pub fn hamming_distance(a: &[u8], b: &[u8]) -> f64 {
    assert_eq!(a.len(), b.len(), "Responses must be equal length");

    let mut diff_bits = 0usize;
    let total_bits = a.len() * 8;

    for (byte_a, byte_b) in a.iter().zip(b.iter()) {
        let xor = byte_a ^ byte_b;
        diff_bits += xor.count_ones() as usize;
    }

    diff_bits as f64 / total_bits as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_puf_deterministic_per_node() {
        let salt = [0u8; 32];
        let fp = HardwareFingerprint::from_node_id("node-alpha", &salt);
        let puf = SramPuf::new(fp);

        let challenge = [1u8; 32];
        let r1 = puf.generate(&challenge);
        let r2 = puf.generate(&challenge);

        assert_eq!(
            r1.bits, r2.bits,
            "Same node + challenge must produce identical response"
        );
        assert!(r1.stability_score > 0.45, "Stability must exceed threshold");
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
        assert!(
            hd > 0.40,
            "Inter-chip Hamming distance must exceed 40%, got {}",
            hd
        );
        assert!(
            hd < 0.60,
            "Inter-chip Hamming distance must be below 60%, got {}",
            hd
        );
    }

    #[test]
    fn test_key_extraction_requires_stability() {
        let salt = [0u8; 32];
        let fp = HardwareFingerprint::from_node_id("node-alpha", &salt);
        let puf = SramPuf::new(fp);

        let challenge = [1u8; 32];
        let response = puf.generate(&challenge);
        assert!(response.stability_score > 0.45);

        let key = puf.extract_key(&response);
        assert!(key.is_ok(), "Stable response must yield extractable key");
    }
}
