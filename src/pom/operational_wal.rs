#[allow(dead_code)]
#[allow(dead_code)]
const WAL_MAGIC: [u8; 4] = *b"SPOM";
#[allow(dead_code)]
#[allow(dead_code)]
const WAL_VERSION: u16 = 1;

use crate::pom::operational_memory::OperationalEntry;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

const SEGMENT_MAX_ENTRIES: usize = 10_000;
const SEGMENT_MAX_BYTES: usize = 64 * 1024 * 1024; // 64MB

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct WalSegmentHeader {
    magic: [u8; 4],
    version: u16,
    entry_count: u64,
    checksum: [u8; 32],
}

pub struct OperationalWal {
    base_path: PathBuf,
    current_segment: File,
    current_entries: u64,
    current_bytes: u64,
    segment_sequence: u64,
}

impl OperationalWal {
    pub fn new(base_path: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&base_path)?;
        let segment_path = base_path.join(format!("wal_segment_{:016}.log", 0));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&segment_path)?;

        Ok(Self {
            base_path,
            current_segment: file,
            current_entries: 0,
            current_bytes: 0,
            segment_sequence: 0,
        })
    }

    pub fn append(&mut self, entry: &OperationalEntry) -> std::io::Result<()> {
        let data = serde_json::to_vec(entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Check rotation trigger
        if self.current_entries >= SEGMENT_MAX_ENTRIES as u64
            || self.current_bytes + data.len() as u64 > SEGMENT_MAX_BYTES as u64
        {
            self.rotate_segment()?;
        }

        // Write length prefix + data
        let len_bytes = (data.len() as u32).to_be_bytes();
        self.current_segment.write_all(&len_bytes)?;
        self.current_segment.write_all(&data)?;
        self.current_segment.sync_all()?; // fsync hardened

        self.current_entries += 1;
        self.current_bytes += 4 + data.len() as u64;

        Ok(())
    }

    fn rotate_segment(&mut self) -> std::io::Result<()> {
        // Close current, start new
        self.segment_sequence += 1;
        let new_path = self
            .base_path
            .join(format!("wal_segment_{:016}.log", self.segment_sequence));
        self.current_segment = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&new_path)?;
        self.current_entries = 0;
        self.current_bytes = 0;
        Ok(())
    }

    pub fn replay<F>(&self, mut handler: F) -> std::io::Result<()>
    where
        F: FnMut(&OperationalEntry),
    {
        // Replay all segments in order
        let mut seq = 0u64;
        loop {
            let path = self.base_path.join(format!("wal_segment_{:016}.log", seq));
            if !path.exists() {
                break;
            }
            let mut file = File::open(&path)?;
            Self::replay_segment(&mut file, &mut handler)?;
            seq += 1;
        }
        Ok(())
    }

    fn replay_segment<F>(file: &mut File, handler: &mut F) -> std::io::Result<()>
    where
        F: FnMut(&OperationalEntry),
    {
        loop {
            let mut len_buf = [0u8; 4];
            match file.read_exact(&mut len_buf) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
            let len = u32::from_be_bytes(len_buf) as usize;
            let mut data = vec![0u8; len];
            file.read_exact(&mut data)?;

            let entry: OperationalEntry = serde_json::from_slice(&data)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            handler(&entry);
        }
        Ok(())
    }

    pub fn entry_count(&self) -> u64 {
        self.current_entries
    }

    pub fn segment_count(&self) -> u64 {
        self.segment_sequence + 1
    }
}



