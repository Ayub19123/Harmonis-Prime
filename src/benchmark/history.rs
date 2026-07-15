//! M2.7.14 Layer 4: VersionHistory — SQLite ledger for performance tracking

use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;

/// M2.7.14: Database schema for version-to-version performance tracking
pub struct VersionHistory {
    conn: Connection,
}

impl VersionHistory {
    /// Initialize or open the performance database
    pub fn open(path: &Path) -> SqlResult<Self> {
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS benchmark_runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                git_tag TEXT NOT NULL,
                instance_path TEXT NOT NULL,
                instance_hash TEXT NOT NULL,
                result TEXT NOT NULL,
                decisions INTEGER,
                propagations INTEGER,
                conflicts INTEGER,
                restarts INTEGER DEFAULT 0,
                peak_memory_kb INTEGER DEFAULT 0,
                wall_time_ms INTEGER,
                proof_valid INTEGER,
                timed_out INTEGER,
                memory_exceeded INTEGER,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // M2.7.14: Schema migration for tables created before this brick
        conn.execute(
            "ALTER TABLE benchmark_runs ADD COLUMN restarts INTEGER DEFAULT 0",
            [],
        )
        .ok();
        conn.execute(
            "ALTER TABLE benchmark_runs ADD COLUMN peak_memory_kb INTEGER DEFAULT 0",
            [],
        )
        .ok();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS version_metadata (
                git_tag TEXT PRIMARY KEY,
                commit_hash TEXT,
                test_count INTEGER,
                compiler_warnings INTEGER,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    /// Record a single benchmark run
    pub fn record_run(
        &self,
        git_tag: &str,
        instance_path: &str,
        instance_hash: &str,
        result: &str,
        decisions: u64,
        propagations: u64,
        conflicts: u64,
        restarts: u64,
        peak_memory_kb: u64,
        wall_time_ms: u64,
        proof_valid: Option<bool>,
        timed_out: bool,
        memory_exceeded: bool,
    ) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO benchmark_runs 
             (git_tag, instance_path, instance_hash, result, decisions, propagations, 
              conflicts, restarts, peak_memory_kb, wall_time_ms, proof_valid, timed_out, memory_exceeded)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            [
                git_tag,
                instance_path,
                instance_hash,
                result,
                &decisions.to_string(),
                &propagations.to_string(),
                &conflicts.to_string(),
                &restarts.to_string(),
                &peak_memory_kb.to_string(),
                &wall_time_ms.to_string(),
                &proof_valid.map(|v| if v { 1 } else { 0 }).unwrap_or(-1).to_string(),
                &if timed_out { 1 } else { 0 }.to_string(),
                &if memory_exceeded { 1 } else { 0 }.to_string(),
            ],
        )?;
        Ok(())
    }

    /// Record version metadata (build health snapshot)
    pub fn record_version(
        &self,
        git_tag: &str,
        commit_hash: &str,
        test_count: usize,
        compiler_warnings: usize,
    ) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO version_metadata 
             (git_tag, commit_hash, test_count, compiler_warnings)
             VALUES (?1, ?2, ?3, ?4)",
            [
                git_tag,
                commit_hash,
                &test_count.to_string(),
                &compiler_warnings.to_string(),
            ],
        )?;
        Ok(())
    }

    /// Query all runs for a specific git tag
    pub fn query_tag(
        &self,
        git_tag: &str,
    ) -> SqlResult<Vec<(String, u64, u64, u64, u64, u64, u64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT instance_path, decisions, propagations, conflicts, restarts, peak_memory_kb, wall_time_ms
             FROM benchmark_runs WHERE git_tag = ?1"
        )?;

        let rows = stmt.query_map([git_tag], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, u64>(1)?,
                row.get::<_, u64>(2)?,
                row.get::<_, u64>(3)?,
                row.get::<_, u64>(4)?,
                row.get::<_, u64>(5)?,
                row.get::<_, u64>(6)?,
            ))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
}
