//! SET-5.5: Highly Composite Numbers for optimization input sizes

/// Generate first N highly composite numbers (simplified)
pub fn highly_composite_numbers(n: usize) -> Vec<u64> {
    let mut hcn = Vec::new();
    let mut max_divisors = 0;
    let mut num = 1;
    while hcn.len() < n {
        let div_count = divisor_count(num);
        if div_count > max_divisors {
            hcn.push(num);
            max_divisors = div_count;
        }
        num += 1;
    }
    hcn
}

fn divisor_count(n: u64) -> u64 {
    let mut count = 0;
    let limit = (n as f64).sqrt() as u64;
    for i in 1..=limit {
        if n % i == 0 {
            count += 1;
            if i != n / i {
                count += 1;
            }
        }
    }
    count
}
