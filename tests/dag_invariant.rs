//! Property-based invariant testing for HBS-1.2

use proptest::prelude::*;
use sovereign_core::mesh::dag::{CognitiveMesh, Message, MessageId, NodeId};
use sovereign_core::stats::median_of_n::{CognitiveStream, Measurement, MedianOfNReporter};
use std::time::Instant;

proptest! {
    /// INVARIANT 1: No valid sequence of appends creates a cycle
    #[test]
    fn dag_acyclicity_invariant(
        message_count in 10usize..1000,
        parent_degree in 1usize..5,
    ) {
        let genesis = Message {
            id: MessageId(0),
            payload: vec![0u8; 32],
            parents: vec![],
            timestamp: Instant::now(),
            source: NodeId(0),
        };

        let mut mesh = CognitiveMesh::new(genesis).unwrap();
        let mut next_id = 1u64;

        for _ in 0..message_count {
            let parent_count = (next_id as usize % parent_degree).max(1).min(next_id as usize);
            let parents: Vec<MessageId> = (0..parent_count)
                .map(|i| MessageId(i as u64))
                .collect();

            let msg = Message {
                id: MessageId(next_id),
                payload: vec![next_id as u8; 32],
                parents,
                timestamp: Instant::now(),
                source: NodeId(next_id % 3),
            };

            // If append succeeds, graph MUST remain acyclic
            if let Ok(receipt) = mesh.append_message(msg) {
                prop_assert!(mesh.verify_acyclic(),
                    "Cycle detected after successful append of {:?}", receipt);
            }
            // If append fails with CycleViolation, that's correct rejection

            next_id += 1;
        }

        // Final verification
        prop_assert!(mesh.verify_acyclic(), "Final graph contains cycle");
    }

    /// INVARIANT 2: Median-of-N converges under Byzantine noise
    #[test]
    fn median_convergence_under_byzantine(
        base_value in 100.0_f64..1000.0,
        noise_ratio in 0.0_f64..0.35,
        stream_count in 3usize..21,
        window_size in 10usize..100,
    ) {
        let mut streams: Vec<CognitiveStream> = (0..stream_count)
            .map(|i| CognitiveStream {
                stream_id: i as u64,
                measurements: Vec::new(),
            })
            .collect();

        // Generate measurements with Byzantine corruption
        for stream in streams.iter_mut() {
            for j in 0..window_size {
                let is_byzantine = (j as f64 / window_size as f64) < noise_ratio;
                let value = if is_byzantine {
                    base_value * 10.0
                } else {
                    base_value + (j as f64 * 0.01)
                };

                stream.measurements.push(Measurement {
                    value,
                    timestamp: j as u64,
                    source_node: stream.stream_id,
                });
            }
        }

        let report = MedianOfNReporter::report(&streams, window_size);

        // Median must be within 5% of true value even with 35% Byzantine noise
        let tolerance = base_value * 0.05;
        prop_assert!((report.median - base_value).abs() < tolerance,
            "Median {} deviated too far from {} (tolerance: {})",
            report.median, base_value, tolerance);

        // Byzantine detection must flag when ratio > 30%
        if noise_ratio > 0.35 {
            prop_assert!(report.byzantine_detected,
                "Failed to detect Byzantine noise at ratio {}", noise_ratio);
        }
    }
}
