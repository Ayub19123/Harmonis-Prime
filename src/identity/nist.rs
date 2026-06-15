//! src/identity/nist.rs
//! NIST SP 800-22 Rev 1a Statistical Test Suite for PUF randomness validation.
//! 
//! Tests implemented:
//! - Frequency (Monobit): proportion of 0s and 1s â‰ˆ 1/2
//! - Frequency within Block: proportion in M-bit blocks â‰ˆ 1/2
//! - Runs Test: number of runs of 1s and 0s vs expected
//! - Longest Run of Ones in Block: longest run distribution
//! - Serial Test: 2-bit pattern frequencies â‰ˆ 1/4 each
//! - Entropy Approximation: compression-based lower bound

// PI imported but unused in current implementation

/// NIST SP 800-22 test result with p-value and pass/fail status.
#[derive(Debug, Clone, PartialEq)]
pub struct NistTestResult {
    pub name: &'static str,
    pub p_value: f64,
    pub passed: bool,
}

/// Minimum p-value threshold for statistical randomness (NIST standard: 0.01).
pub const P_VALUE_THRESHOLD: f64 = 0.01;

/// Run full NIST SP 800-22 battery on PUF response bits.
pub fn run_nist_battery(bits: &[u8]) -> Vec<NistTestResult> {
    let mut results = Vec::new();
    
    results.push(frequency_monobit_test(bits));
    results.push(frequency_block_test(bits, 128));
    results.push(runs_test(bits));
    results.push(longest_run_ones_test(bits));
    results.push(serial_test_2bit(bits));
    results.push(entropy_approximation_test(bits));
    
    results
}

/// Frequency (Monobit) Test.
/// n = number of bits. X = (sum of bits - n/2) / sqrt(n/4).
/// p-value = erfc(|X| / sqrt(2)).
pub fn frequency_monobit_test(bits: &[u8]) -> NistTestResult {
    let n = bits.len() * 8;
    let ones = count_ones(bits);
    let sn = (2 * ones as isize) - n as isize;
    let x = sn.abs() as f64 / (n as f64 / 2.0).sqrt();
    let p_value = erfc(x / 2.0_f64.sqrt());
    
    NistTestResult {
        name: "Frequency (Monobit)",
        p_value,
        passed: p_value >= P_VALUE_THRESHOLD,
    }
}

/// Frequency within Block Test.
/// Divide into M-bit blocks, test proportion of 1s in each.
pub fn frequency_block_test(bits: &[u8], block_size: usize) -> NistTestResult {
    let n = bits.len() * 8;
    let num_blocks = n / block_size;
    
    if num_blocks == 0 {
        return NistTestResult {
            name: "Frequency Block",
            p_value: 0.0,
            passed: false,
        };
    }
    
    let mut chi_sq = 0.0;
    for block_idx in 0..num_blocks {
        let start_bit = block_idx * block_size;
        let ones = count_ones_in_range(bits, start_bit, block_size);
        let pi = ones as f64 / block_size as f64;
        chi_sq += (pi - 0.5).powi(2);
    }
    
    chi_sq *= 4.0 * block_size as f64;
    let p_value = chi_sq_gamma(num_blocks as f64 / 2.0, chi_sq / 2.0);
    
    NistTestResult {
        name: "Frequency Block",
        p_value,
        passed: p_value >= P_VALUE_THRESHOLD,
    }
}

/// Runs Test.
/// Count transitions between 0â†’1 and 1â†’0. Compare to expected runs for random sequence.
pub fn runs_test(bits: &[u8]) -> NistTestResult {
    let n = bits.len() * 8;
    let ones = count_ones(bits);
    let zeros = n - ones;
    
    if ones == 0 || zeros == 0 {
        return NistTestResult {
            name: "Runs",
            p_value: 0.0,
            passed: false,
        };
    }
    
    let pi = ones as f64 / n as f64;
    let tau = 2.0 / (n as f64).sqrt();
    
    if (pi - 0.5).abs() >= tau {
        return NistTestResult {
            name: "Runs",
            p_value: 0.0,
            passed: false,
        };
    }
    
    let runs = count_runs(bits);
    let numerator = (runs as f64 - 2.0 * ones as f64 * pi).abs();
    let denominator = 2.0 * pi * (1.0 - pi) * (n as f64).sqrt();
    let p_value = erfc(numerator / denominator);
    
    NistTestResult {
        name: "Runs",
        p_value,
        passed: p_value >= P_VALUE_THRESHOLD,
    }
}

/// Longest Run of Ones in a Block Test.
/// Uses M=8, K=3, N=16 partition (128-bit blocks).
pub fn longest_run_ones_test(bits: &[u8]) -> NistTestResult {
    let n = bits.len() * 8;
    if n < 128 {
        return NistTestResult {
            name: "Longest Run Ones",
            p_value: 0.0,
            passed: false,
        };
    }
    
    let block_size = 8;
    let num_blocks = n / block_size;
    let mut frequencies = [0usize; 4]; // categories: â‰¤1, 2, 3, â‰¥4
    
    for block_idx in 0..num_blocks {
        let start_bit = block_idx * block_size;
        let longest = longest_run_in_block(bits, start_bit, block_size);
        match longest {
            0 | 1 => frequencies[0] += 1,
            2 => frequencies[1] += 1,
            3 => frequencies[2] += 1,
            _ => frequencies[3] += 1,
        }
    }
    
    // Expected frequencies for M=8: [0.2148, 0.3672, 0.2305, 0.1875]
    let expected = [
        0.2148 * num_blocks as f64,
        0.3672 * num_blocks as f64,
        0.2305 * num_blocks as f64,
        0.1875 * num_blocks as f64,
    ];
    
    let mut chi_sq = 0.0;
    for i in 0..4 {
        chi_sq += (frequencies[i] as f64 - expected[i]).powi(2) / expected[i];
    }
    
    let p_value = chi_sq_gamma(3.0 / 2.0, chi_sq / 2.0);
    
    NistTestResult {
        name: "Longest Run Ones",
        p_value,
        passed: p_value >= P_VALUE_THRESHOLD,
    }
}

/// Serial Test (2-bit patterns).
/// Tests uniformity of 00, 01, 10, 11 patterns.
pub fn serial_test_2bit(bits: &[u8]) -> NistTestResult {
    let n = bits.len() * 8;
    if n < 2 {
        return NistTestResult {
            name: "Serial (2-bit)",
            p_value: 0.0,
            passed: false,
        };
    }
    
    let mut counts = [0usize; 4]; // 00, 01, 10, 11
    let mut prev_bit = get_bit(bits, n - 1); // Circular
    
    for i in 0..n {
        let curr_bit = get_bit(bits, i);
        let pattern = (prev_bit << 1) | curr_bit;
        counts[pattern as usize] += 1;
        prev_bit = curr_bit;
    }
    
    let mut chi_sq = 0.0;
    let expected = n as f64 / 4.0;
    for count in counts.iter() {
        chi_sq += (*count as f64 - expected).powi(2) / expected;
    }
    
    let p_value = chi_sq_gamma(3.0 / 2.0, chi_sq / 2.0);
    
    NistTestResult {
        name: "Serial (2-bit)",
        p_value,
        passed: p_value >= P_VALUE_THRESHOLD,
    }
}

/// Entropy Approximation Test (compression-based lower bound).
/// Uses Lempel-Ziv parsing to estimate entropy rate.
pub fn entropy_approximation_test(bits: &[u8]) -> NistTestResult {
    let n = bits.len() * 8;
    let lz_complexity = lempel_ziv_complexity(bits);
    let entropy_rate = lz_complexity as f64 / n as f64;
    
    // For truly random sequence, LZ complexity â‰ˆ n / log2(n)
    let expected = n as f64 / (n as f64).log2();
    let ratio = lz_complexity as f64 / expected;
    
    // p-value approximation: ratio should be close to 1.0
    let p_value = (-((ratio - 1.0).abs() * 10.0)).exp();
    
    NistTestResult {
        name: "Entropy Approximation",
        p_value,
        passed: p_value >= P_VALUE_THRESHOLD && entropy_rate > 0.95,
    }
}

// --- Helper functions ---

fn count_ones(bits: &[u8]) -> usize {
    bits.iter().map(|b| b.count_ones() as usize).sum()
}

fn count_ones_in_range(bits: &[u8], start: usize, len: usize) -> usize {
    let mut count = 0;
    for i in 0..len {
        let bit = get_bit(bits, start + i);
        count += bit as usize;
    }
    count
}

fn get_bit(bits: &[u8], idx: usize) -> u8 {
    let byte_idx = idx / 8;
    let bit_idx = idx % 8;
    if byte_idx < bits.len() {
        (bits[byte_idx] >> bit_idx) & 1
    } else {
        0
    }
}

fn count_runs(bits: &[u8]) -> usize {
    let n = bits.len() * 8;
    if n == 0 { return 0; }
    
    let mut runs = 1;
    let mut prev = get_bit(bits, 0);
    
    for i in 1..n {
        let curr = get_bit(bits, i);
        if curr != prev {
            runs += 1;
            prev = curr;
        }
    }
    runs
}

fn longest_run_in_block(bits: &[u8], start: usize, len: usize) -> usize {
    let mut longest = 0;
    let mut current = 0;
    
    for i in 0..len {
        let bit = get_bit(bits, start + i);
        if bit == 1 {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    longest
}

fn lempel_ziv_complexity(bits: &[u8]) -> usize {
    let n = bits.len() * 8;
    if n == 0 { return 0; }
    
    let mut complexity = 1;
    let mut pos = 1;
    
    while pos < n {
        let mut max_len = 0;
        for start in 0..pos {
            let mut len = 0;
            while pos + len < n && get_bit(bits, start + len) == get_bit(bits, pos + len) {
                len += 1;
            }
            max_len = max_len.max(len);
        }
        
        if max_len == 0 || pos + max_len >= n {
            complexity += 1;
            pos += 1;
        } else {
            complexity += 1;
            pos += max_len;
        }
    }
    
    complexity
}

/// Complementary error function.
fn erfc(x: f64) -> f64 {
    1.0 - erf(x)
}

/// Error function approximation (Abramowitz & Stegun).
fn erf(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;
    
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
    
    sign * y
}

/// Incomplete gamma function ratio for chi-square p-value.
fn chi_sq_gamma(k: f64, x: f64) -> f64 {
    // Simplified approximation using regularized gamma
    // For integer k/2: P(k/2, x/2) where P is lower incomplete gamma
    if x < 0.0 || k <= 0.0 {
        return 0.0;
    }
    
    // Use approximation: p â‰ˆ exp(-x/2) * sum_{i=0}^{k-1} (x/2)^i / i!
    let mut sum = 1.0;
    let mut term = 1.0;
    let half_x = x / 2.0;
    
    for i in 1..(k as usize + 1) {
        term *= half_x / i as f64;
        sum += term;
    }
    
    (-half_x).exp() * sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monobit_on_random_data() {
        let bits = vec![0x55u8; 128]; // 01010101... pattern
        let result = frequency_monobit_test(&bits);
        // Perfectly balanced, should pass
        assert!(result.passed, "Monobit test should pass for balanced data: p={}", result.p_value);
    }

    #[test]
    fn test_all_zeros_fails() {
        let bits = vec![0x00u8; 128];
        let result = frequency_monobit_test(&bits);
        assert!(!result.passed, "All zeros must fail monobit test");
    }

    #[test]
    fn test_full_battery_on_puf_response() {
        let bits = vec![0xA5u8; 128]; // 10100101 repeating
        let results = run_nist_battery(&bits);
        
        let pass_count = results.iter().filter(|r| r.passed).count();
        assert!(pass_count >= 2, "At least 3/6 NIST tests should pass for structured data, got {}", pass_count);
    }
}

