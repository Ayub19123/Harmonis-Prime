//! M2.7: Lit-Pack Binary Protocol
//! Compact clause serialization for distributed registry.
//!
//! Packet Layout:
//!   Header (32-bit):   [Origin Node ID: 8 bits][Clause Length: 16 bits][LBD Score: 8 bits]
//!   Metadata (32-bit): Birth timestamp (seconds since epoch)
//!   Payload:           Contiguous i32 literals

use std::io::{self, Read, Write};

/// Compact clause packet for DHT transmission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LitPack {
    /// Origin node identifier (8-bit, 0-255)
    pub origin_id: u8,
    /// Number of literals in the clause (16-bit, 0-65535)
    pub clause_len: u16,
    /// Literal Block Distance score (8-bit, lower is better)
    pub lbd_score: u8,
    /// Birth timestamp (seconds since UNIX epoch)
    pub birth_timestamp: u32,
    /// Clause literals: positive = variable, negative = negation
    pub literals: Vec<i32>,
}

impl LitPack {
    /// Create a new Lit-Pack from components.
    pub fn new(origin_id: u8, lbd_score: u8, literals: Vec<i32>) -> Self {
        let clause_len = literals.len() as u16;
        let birth_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        Self {
            origin_id,
            clause_len,
            lbd_score,
            birth_timestamp,
            literals,
        }
    }

    /// Serialize to bytes (little-endian, network-friendly).
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(8 + self.literals.len() * 4);

        // Header: [origin_id: u8][clause_len: u16][lbd_score: u8]
        buf.write_all(&[self.origin_id])?;
        buf.write_all(&self.clause_len.to_le_bytes())?;
        buf.write_all(&[self.lbd_score])?;

        // Metadata: birth_timestamp (u32)
        buf.write_all(&self.birth_timestamp.to_le_bytes())?;

        // Payload: contiguous i32 literals
        for lit in &self.literals {
            buf.write_all(&lit.to_le_bytes())?;
        }

        Ok(buf)
    }

    /// Deserialize from bytes.
    pub fn decode(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Lit-Pack too short (< 8 bytes)",
            ));
        }

        let mut cursor = io::Cursor::new(bytes);

        let mut origin_buf = [0u8; 1];
        cursor.read_exact(&mut origin_buf)?;
        let origin_id = origin_buf[0];

        let mut len_buf = [0u8; 2];
        cursor.read_exact(&mut len_buf)?;
        let clause_len = u16::from_le_bytes(len_buf);

        let mut lbd_buf = [0u8; 1];
        cursor.read_exact(&mut lbd_buf)?;
        let lbd_score = lbd_buf[0];

        let mut ts_buf = [0u8; 4];
        cursor.read_exact(&mut ts_buf)?;
        let birth_timestamp = u32::from_le_bytes(ts_buf);

        let expected_payload = clause_len as usize * 4;
        let actual_payload = bytes.len() - 8;

        if expected_payload != actual_payload {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Payload size mismatch: expected {} bytes, got {}",
                    expected_payload, actual_payload
                ),
            ));
        }

        let mut literals = Vec::with_capacity(clause_len as usize);
        for _ in 0..clause_len {
            let mut lit_buf = [0u8; 4];
            cursor.read_exact(&mut lit_buf)?;
            literals.push(i32::from_le_bytes(lit_buf));
        }

        Ok(Self {
            origin_id,
            clause_len,
            lbd_score,
            birth_timestamp,
            literals,
        })
    }

    /// Compute utility score for epistemic filtering.
    /// Utility(C) = α·LBD(C) + β·Size(C) + γ·Activity(C)
    /// For now, activity is approximated by inverse age.
    pub fn utility(&self, alpha: f64, beta: f64, gamma: f64) -> f64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as f64;

        let age_seconds = now - self.birth_timestamp as f64;
        let activity = if age_seconds > 0.0 {
            1.0 / age_seconds
        } else {
            1.0
        };

        alpha * self.lbd_score as f64
            + beta * self.clause_len as f64
            + gamma * activity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let pack = LitPack::new(42, 3, vec![1, -2, 3, -4, 5]);
        let encoded = pack.encode().unwrap();
        let decoded = LitPack::decode(&encoded).unwrap();
        assert_eq!(pack.origin_id, decoded.origin_id);
        assert_eq!(pack.clause_len, decoded.clause_len);
        assert_eq!(pack.lbd_score, decoded.lbd_score);
        assert_eq!(pack.literals, decoded.literals);
    }

    #[test]
    fn test_utility_bounds() {
        let pack = LitPack::new(1, 2, vec![1, -2]);
        let u = pack.utility(1.0, 1.0, 1.0);
        assert!(u > 0.0);
    }
}