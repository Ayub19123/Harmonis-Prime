//! M2.2: Odlyzko Zero Fetcher (University of Minnesota)
//! LIMITATION: Requires internet connection. Run manually before tests.
//! LIMITATION: Rate limit: 2s delay between requests (respectful).
//! LIMITATION: Windows-compatible. No platform-specific code.
//! OUTPUT: Downloads zero data to .workspace/odlyzko_cache/ with SHA-256 integrity.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

const CACHE_DIR: &str = ".workspace/odlyzko_cache";
const ODLYZKO_URL: &str = "https://www-users.cse.umn.edu/~odlyzko/zeta_tables/zeros1";
const RATE_LIMIT_MS: u64 = 2000; // 2 seconds between requests

fn main() -> io::Result<()> {
    println!("============================================================");
    println!("ODLYZKO ZERO FETCHER — M2.2");
    println!("============================================================");
    
    // Ensure cache directory exists
    let cache_dir = PathBuf::from(CACHE_DIR);
    fs::create_dir_all(&cache_dir)?;
    
    let cache_file = cache_dir.join("zeros1.txt");
    let hash_file = cache_dir.join("zeros1.sha256");
    
    // Check if already cached
    if cache_file.exists() && hash_file.exists() {
        println!("Cache hit: {}", cache_file.display());
        let cached_hash = fs::read_to_string(&hash_file)?.trim().to_string();
        let data = fs::read_to_string(&cache_file)?;
        let computed_hash = sha256_hex(&data);
        
        if cached_hash == computed_hash {
            println!("✅ Integrity verified. No fetch needed.");
            return Ok(());
        } else {
            println!("⚠️ Hash mismatch. Re-fetching...");
        }
    }
    
    println!("Fetching from Odlyzko dataset...");
    println!("URL: {}", ODLYZKO_URL);
    println!("Rate limit: {} ms between requests", RATE_LIMIT_MS);
    
    // Rate limit delay
    std::thread::sleep(Duration::from_millis(RATE_LIMIT_MS));
    
    // Synchronous HTTP GET — ureq v3 API
    let response = ureq::get(ODLYZKO_URL)
        .header("User-Agent", "Harmonis-Prime/6.2.0 (Research; contact: github.com/Ayub19123/Harmonis-Prime)")
        .call()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("HTTP error: {}", e)))?;
    
    let body = response.into_body()
        .read_to_string()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Read error: {}", e)))?;
    
    println!("Downloaded: {} bytes", body.len());
    
    // Compute SHA-256
    let hash = sha256_hex(&body);
    println!("SHA-256: {}", hash);
    
    // Write cache
    let mut file = File::create(&cache_file)?;
    file.write_all(body.as_bytes())?;
    
    let mut hash_file_handle = File::create(&hash_file)?;
    hash_file_handle.write_all(hash.as_bytes())?;
    hash_file_handle.write_all(b"\n")?;
    
    println!("✅ Cached: {}", cache_file.display());
    println!("✅ Hash:   {}", hash_file.display());
    println!("============================================================");
    println!("NEXT: Run `cargo test --lib -- --ignored odlyzko_cache` to verify.");
    
    Ok(())
}

/// Compute SHA-256 hex digest of string data using ring.
fn sha256_hex(data: &str) -> String {
    use ring::digest::{digest, SHA256};
    let hash = digest(&SHA256, data.as_bytes());
    hash.as_ref()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}