use crate::pom::operational_memory::Snapshot;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub struct SnapshotEngine {
    #[allow(dead_code)]
    #[allow(dead_code)]
    last_snapshot_epoch: u64,

    snapshot_dir: PathBuf,
    compaction_threshold_entries: u64,
}

impl SnapshotEngine {
    pub fn new(snapshot_dir: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&snapshot_dir)?;
        Ok(Self {
            snapshot_dir,
            #[allow(dead_code)]
            last_snapshot_epoch: 0,
            compaction_threshold_entries: 10_000,
        })
    }

    pub fn should_compact(&self, entry_count: u64) -> bool {
        entry_count >= self.compaction_threshold_entries
    }

    pub fn create_snapshot(&self, snapshot: &Snapshot) -> std::io::Result<PathBuf> {
        let path = self.snapshot_dir.join(format!(
            "snapshot_{:016}_{:016}.snap",
            snapshot.epoch_id, snapshot.sequence
        ));

        let data = serde_json::to_vec(snapshot)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?;

        file.write_all(&data)?;
        file.sync_all()?; // fsync hardened

        Ok(path)
    }

    pub fn load_latest_snapshot(&self) -> std::io::Result<Option<Snapshot>> {
        let mut latest: Option<(u64, u64, PathBuf)> = None;

        for entry in std::fs::read_dir(&self.snapshot_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("snapshot_") && name.ends_with(".snap") {
                let parts: Vec<&str> = name
                    .trim_start_matches("snapshot_")
                    .trim_end_matches(".snap")
                    .split('_')
                    .collect();
                if parts.len() == 2 {
                    let epoch = parts[0].parse::<u64>().unwrap_or(0);
                    let seq = parts[1].parse::<u64>().unwrap_or(0);
                    if latest.is_none()
                        || (epoch, seq) > (latest.as_ref().unwrap().0, latest.as_ref().unwrap().1)
                    {
                        latest = Some((epoch, seq, entry.path()));
                    }
                }
            }
        }

        match latest {
            Some((_, _, path)) => {
                let data = std::fs::read(&path)?;
                let snapshot = serde_json::from_slice(&data)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                Ok(Some(snapshot))
            }
            None => Ok(None),
        }
    }

    pub fn set_compaction_threshold(&mut self, threshold: u64) {
        self.compaction_threshold_entries = threshold;
    }
}


