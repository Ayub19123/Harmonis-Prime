//! M2.7.2: Epistemic DRAT Proof Logging
//! Extends standard DRAT proofs with clause provenance metadata.
//!
//! DRAT format compatibility:
//!   - 'a' lines: clause additions (standard DRAT)
//!   - 'd' lines: clause deletions (standard DRAT)
//!   - 'c' lines: comments (ignored by drat-trim, preserved for audit)
//!
//! Epistemic comment schema:
//!   c epistemic origin_id=<u8> lbd=<u8> timestamp=<u32>

/// Metadata for a learned clause in the DRAT proof trail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EpistemicMeta {
    /// Origin node ID (0 = local solver, 1-255 = distributed peers)
    pub origin_id: u8,
    /// Literal Block Distance: number of unique decision levels in clause
    /// Lower is better (2 = glue clause, highly valuable)
    pub lbd: u8,
    /// Birth timestamp: seconds since UNIX epoch
    pub timestamp: u32,
}

impl EpistemicMeta {
    /// Create metadata for a locally learned clause.
    pub fn local(lbd: u8) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        Self {
            origin_id: 0,
            lbd,
            timestamp,
        }
    }

    /// Create metadata with explicit origin (for distributed clauses).
    pub fn from_origin(origin_id: u8, lbd: u8) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        Self {
            origin_id,
            lbd,
            timestamp,
        }
    }

    /// Serialize to DRAT comment line.
    pub fn to_drat_comment(&self) -> String {
        format!(
            "c epistemic origin_id={} lbd={} timestamp={}",
            self.origin_id, self.lbd, self.timestamp
        )
    }
}

/// Extended proof entry: either a standard DRAT line or epistemic metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProofEntry {
    /// Standard DRAT line (a/d <literals> 0)
    Line(String),
    /// Epistemic metadata comment preceding a clause addition
    Meta(EpistemicMeta),
}

/// Proof trace with epistemic metadata support.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EpistemicProofTrace {
    entries: Vec<ProofEntry>,
}

impl EpistemicProofTrace {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a standard DRAT line.
    pub fn push_line(&mut self, line: String) {
        self.entries.push(ProofEntry::Line(line));
    }

    /// Add an epistemic metadata entry.
    pub fn push_meta(&mut self, meta: EpistemicMeta) {
        self.entries.push(ProofEntry::Meta(meta));
    }

    /// Write proof trace to file in DRAT-compatible format.
    /// 'c' lines are comments (ignored by drat-trim).
    pub fn write_to_file(&self, path: &str) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        for entry in &self.entries {
            match entry {
                ProofEntry::Line(line) => writeln!(file, "{}", line)?,
                ProofEntry::Meta(meta) => writeln!(file, "{}", meta.to_drat_comment())?,
            }
        }
        Ok(())
    }

    /// Return raw entries for inspection.
    pub fn entries(&self) -> &[ProofEntry] {
        &self.entries
    }

    /// Count of epistemic metadata entries.
    pub fn meta_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| matches!(e, ProofEntry::Meta(_)))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_meta_serialization() {
        let meta = EpistemicMeta::local(3);
        let comment = meta.to_drat_comment();
        assert!(comment.starts_with("c epistemic origin_id=0 lbd=3 timestamp="));
        assert!(comment.contains("timestamp="));
    }

    #[test]
    fn test_distributed_meta() {
        let meta = EpistemicMeta::from_origin(42, 2);
        assert_eq!(meta.origin_id, 42);
        assert_eq!(meta.lbd, 2);
    }

    #[test]
    fn test_proof_trace_roundtrip() {
        let mut trace = EpistemicProofTrace::new();
        trace.push_meta(EpistemicMeta::local(2));
        trace.push_line("a 1 -2 0".to_string());
        trace.push_line("d 3 -4 0".to_string());

        assert_eq!(trace.meta_count(), 1);
        assert_eq!(trace.entries().len(), 3);
    }
}
