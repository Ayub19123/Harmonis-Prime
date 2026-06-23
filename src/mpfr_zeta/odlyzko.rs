//! M2.2: Odlyzko Zero Cache Reader
//! LIMITATION: Reads from local cache ONLY. No network code.
//! LIMITATION: Panics on cache miss — intentional, tests must run after fetch.

use std::fs;
use std::path::PathBuf;

const CACHE_DIR: &str = ".workspace/odlyzko_cache";
const CACHE_FILE: &str = "zeros1.txt";
const HASH_FILE: &str = "zeros1.sha256";

pub struct OdlyzkoCache {
    pub zeros: Vec<f64>,
    pub hash: String,
}

impl OdlyzkoCache {
    pub fn load() -> Self {
        let cache_dir = PathBuf::from(CACHE_DIR);
        let cache_file = cache_dir.join(CACHE_FILE);
        let hash_file = cache_dir.join(HASH_FILE);
        
        assert!(
            cache_file.exists(),
            "Odlyzko cache miss: run `cargo run --bin lmfdb_fetcher` first.\n\
             Expected: {}",
            cache_file.display()
        );
        
        assert!(
            hash_file.exists(),
            "Odlyzko hash miss: run `cargo run --bin lmfdb_fetcher` first."
        );
        
        let data = fs::read_to_string(&cache_file)
            .expect("Failed to read Odlyzko cache file");
        
        let cached_hash = fs::read_to_string(&hash_file)
            .expect("Failed to read Odlyzko hash file")
            .trim()
            .to_string();
        
        let computed_hash = sha256_hex(&data);
        assert_eq!(
            cached_hash, computed_hash,
            "Odlyzko cache integrity check failed. Re-run fetcher."
        );
        
        let zeros: Vec<f64> = data
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .filter_map(|line| line.trim().parse::<f64>().ok())
            .collect();
        
        assert!(!zeros.is_empty(), "Odlyzko cache parsed zero zeros");
        
        Self {
            zeros,
            hash: computed_hash,
        }
    }
    
    pub fn first_n(&self, n: usize) -> &[f64] {
        &self.zeros[..n.min(self.zeros.len())]
    }
    
    pub fn len(&self) -> usize {
        self.zeros.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.zeros.is_empty()
    }
}

fn sha256_hex(data: &str) -> String {
    use ring::digest::{digest, SHA256};
    let hash = digest(&SHA256, data.as_bytes());
    hash.as_ref()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}