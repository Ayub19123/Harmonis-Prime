//! HBS-1.2: Median-of-N Statistical Reporting
//! Invariant: Anomaly detection via independent multi-threaded sampling

#[derive(Debug, Clone)]
pub struct CognitiveStream {
    pub stream_id: u64,
    pub measurements: Vec<Measurement>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measurement {
    pub value: f64,
    pub timestamp: u64,
    pub source_node: u64,
}

#[derive(Debug, Clone)]
pub struct MeshReport {
    pub median: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub iqr: f64,
    pub outlier_threshold: f64,
    pub outliers: Vec<Measurement>,
    pub confidence_interval_95: (f64, f64),
    pub byzantine_detected: bool,
    pub stream_count: usize,
}

impl MeshReport {
    /// Detect if Byzantine nodes are present (>30% outlier ratio)
    pub fn byzantine_ratio(&self) -> f64 {
        if self.stream_count == 0 {
            return 0.0;
        }
        self.outliers.len() as f64 / self.stream_count as f64
    }
}

pub struct MedianOfNReporter;

impl MedianOfNReporter {
    /// CORE ALGORITHM: Median-of-N with IQR-based outlier detection
    pub fn report(
        streams: &[CognitiveStream],
        window_size: usize,
    ) -> MeshReport {
        let mut all_values: Vec<f64> = Vec::new();
        let mut all_measurements: Vec<Measurement> = Vec::new();
        
        for stream in streams.iter() {
            let window = stream.measurements.iter()
                .rev()
                .take(window_size)
                .cloned();
            
            for m in window {
                all_values.push(m.value);
                all_measurements.push(m);
            }
        }
        
        if all_values.is_empty() {
            return MeshReport {
                median: 0.0,
                mean: 0.0,
                std_dev: 0.0,
                iqr: 0.0,
                outlier_threshold: 0.0,
                outliers: vec![],
                confidence_interval_95: (0.0, 0.0),
                byzantine_detected: false,
                stream_count: streams.len(),
            };
        }
        
        all_values.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        
        let median = Self::statistical_median(&all_values);
        let q1 = Self::percentile(&all_values, 0.25);
        let q3 = Self::percentile(&all_values, 0.75);
        let iqr = q3 - q1;
        let threshold = 1.5 * iqr;
        
        let outliers: Vec<Measurement> = all_measurements.iter()
            .filter(|m| (m.value - median).abs() > threshold)
            .cloned()
            .collect();
        
        let byzantine_detected = !all_values.is_empty() 
            && (outliers.len() as f64 / all_values.len() as f64) > 0.30;
        
        let mean = all_values.iter().sum::<f64>() / all_values.len() as f64;
        let variance = all_values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / all_values.len() as f64;
        let std_dev = variance.sqrt();
        
        let margin = 1.96 * (std_dev / (all_values.len() as f64).sqrt());
        let ci_95 = (mean - margin, mean + margin);
        
        MeshReport {
            median,
            mean,
            std_dev,
            iqr,
            outlier_threshold: threshold,
            outliers,
            confidence_interval_95: ci_95,
            byzantine_detected,
            stream_count: streams.len(),
        }
    }
    
    fn statistical_median(sorted: &[f64]) -> f64 {
        let n = sorted.len();
        if n % 2 == 1 {
            sorted[n / 2]
        } else {
            (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
        }
    }
    
    fn percentile(sorted: &[f64], p: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let idx = (p * (sorted.len() - 1) as f64).floor() as usize;
        sorted[idx.clamp(0, sorted.len() - 1)]
    }
}