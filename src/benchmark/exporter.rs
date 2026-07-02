
//! M2.7.14 Layer 3: MetricsExporter — JSON/CSV structured output

use std::path::Path;
use crate::benchmark::runner::BenchmarkRun;

/// M2.7.14: Export benchmark results to JSON
pub fn export_json(runs: &[BenchmarkRun], path: &Path) -> Result<(), String> {
    let records: Vec<_> = runs.iter().map(|r| serde_json::json!({
        "instance": r.instance_path.to_string_lossy(),
        "instance_hash": r.instance_hash,
        "result": format!("{:?}", r.result),
        "decisions": r.decisions,
        "propagations": r.propagations,
        "conflicts": r.conflicts,
        "restarts": r.restarts,
        "peak_memory_kb": r.peak_memory_kb,
        "wall_time_ms": r.wall_time_ms,
        "proof_valid": r.proof_valid,
        "timed_out": r.timed_out,
        "memory_exceeded": r.memory_exceeded,
    })).collect();

    let json = serde_json::to_string_pretty(&records)
        .map_err(|e| format!("JSON serialization failed: {}", e))?;
    
    std::fs::write(path, json)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    
    Ok(())
}

/// M2.7.14: Export benchmark results to CSV
pub fn export_csv(runs: &[BenchmarkRun], path: &Path) -> Result<(), String> {
    let mut writer = csv::Writer::from_path(path)
        .map_err(|e| format!("CSV writer failed: {}", e))?;

    // Header
    writer.write_record(&[
        "instance", "instance_hash", "result", "decisions", "propagations",
        "conflicts", "restarts", "peak_memory_kb", "wall_time_ms", "proof_valid", 
        "timed_out", "memory_exceeded",
    ]).map_err(|e| format!("CSV header failed: {}", e))?;

    for r in runs {
        writer.write_record(&[
            r.instance_path.to_string_lossy().to_string(),
            r.instance_hash.clone(),
            format!("{:?}", r.result),
            r.decisions.to_string(),
            r.propagations.to_string(),
            r.conflicts.to_string(),
            r.restarts.to_string(),
            r.peak_memory_kb.to_string(),
            r.wall_time_ms.to_string(),
            r.proof_valid.map(|v| v.to_string()).unwrap_or_default(),
            r.timed_out.to_string(),
            r.memory_exceeded.to_string(),
        ]).map_err(|e| format!("CSV record failed: {}", e))?;
    }

    writer.flush().map_err(|e| format!("CSV flush failed: {}", e))?;
    Ok(())
}
