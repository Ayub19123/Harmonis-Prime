//! BRICK-51 Layer 0: Cognitive Mesh Controller
//! Governing equation: ∀ query q: Response(q) = f(LocalExec(q), GlobalModel(Ω))
//! Latency target: → 0

use crate::brick51::collective_reasoning::CollectiveReasoning;
use crate::brick51::shared_memory_graph::SharedMemoryGraph;

pub struct CognitiveMesh {
    pub memory: SharedMemoryGraph,
    pub reasoning: CollectiveReasoning,
    pub query_count: u64,
    pub total_latency_ns: u64,
}

impl CognitiveMesh {
    pub fn new(node_id: usize, node_count: usize) -> Self {
        Self {
            memory: SharedMemoryGraph::new(node_id, node_count),
            reasoning: CollectiveReasoning::new(),
            query_count: 0,
            total_latency_ns: 0,
        }
    }

    pub fn query(&mut self, q: &str, nodes: usize) -> String {
        let start = std::time::Instant::now();

        // Global awareness: check shared memory
        let _context = self.memory.get(q);

        // Local execution: collective reasoning
        let result = self.reasoning.solve(q, nodes, 1.0);

        let elapsed = start.elapsed().as_nanos() as u64;
        self.total_latency_ns += elapsed;
        self.query_count += 1;

        format!(
            "RESPONSE[{}]: quality={:.2}, time={}ms, nodes={}",
            q, result.solution_quality, result.time_ms, result.node_count
        )
    }

    pub fn avg_latency_ns(&self) -> f64 {
        if self.query_count == 0 {
            return 0.0;
        }
        self.total_latency_ns as f64 / self.query_count as f64
    }

    pub fn stats(&self) -> (u64, f64) {
        (self.query_count, self.avg_latency_ns())
    }
}
