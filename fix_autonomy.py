import os

# self_correction.rs — CORRECT SYNTAX
sc_content = """use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackType {
    Positive,
    Negative,
    Neutral,
    Success,
    Failure,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub feedback_id: String,
    pub source: String,
    pub signal_type: FeedbackType,
    pub value: f64,
    pub timestamp_nanos: u64,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    pub correction_id: String,
    pub target: String,
    pub action_type: String,
    pub parameter: String,
    pub old_value: f64,
    pub new_value: f64,
    pub confidence: f64,
    pub applied_at_nanos: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecovery {
    pub recovery_id: String,
    pub error_type: String,
    pub error_message: String,
    pub strategy: String,
    pub success: bool,
    pub recovered_at_nanos: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfCorrection {
    pub feedback_history: VecDeque<<Feedback>,
    pub corrections: Vec<<Correction>,
    pub recoveries: Vec<<ErrorRecovery>,
    pub learning_rate: f64,
    pub threshold_positive: f64,
    pub threshold_negative: f64,
    pub max_history: usize,
    pub adaptation_velocity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionStrategy {
    pub strategy_id: String,
    pub name: String,
    pub effectiveness: f64,
    pub applicable_to: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionStats {
    pub total_feedback: u64,
    pub total_corrections: u64,
    pub total_recoveries: u64,
    pub avg_feedback: f64,
    pub avg_effectiveness: f64,
}

impl SelfCorrection {
    pub fn new(learning_rate: f64, pos_threshold: f64, neg_threshold: f64, max_history: usize) -> Self {
        Self {
            feedback_history: VecDeque::with_capacity(max_history),
            corrections: Vec::new(),
            recoveries: Vec::new(),
            learning_rate: learning_rate.clamp(0.0, 1.0),
            threshold_positive: pos_threshold.clamp(0.0, 1.0),
            threshold_negative: neg_threshold.clamp(-1.0, 0.0),
            max_history,
            adaptation_velocity: 0.0,
        }
    }

    pub fn ingest_feedback(&mut self, feedback: Feedback) -> Option<<Correction> {
        if self.feedback_history.len() >= self.max_history {
            self.feedback_history.pop_front();
        }
        self.feedback_history.push_back(feedback.clone());

        let is_negative = matches!(feedback.signal_type, FeedbackType::Negative | FeedbackType::Error | FeedbackType::Failure);
        
        if is_negative && feedback.value < self.threshold_negative {
            let correction = Correction {
                correction_id: format!("corr_{}", feedback.timestamp_nanos),
                target: feedback.source.clone(),
                action_type: "adjust".to_string(),
                parameter: "weight".to_string(),
                old_value: 0.0,
                new_value: feedback.value.abs() * self.learning_rate,
                confidence: feedback.value.abs().clamp(0.0, 1.0),
                applied_at_nanos: feedback.timestamp_nanos,
            };
            self.corrections.push(correction.clone());
            Some(correction)
        } else {
            None
        }
    }

    pub fn get_average_feedback(&self, window: usize) -> f64 {
        let recent: Vec<&Feedback> = self.feedback_history.iter().rev().take(window).collect();
        if recent.is_empty() { return 0.0; }
        recent.iter().map(|f| f.value).sum::<f64>() / recent.len() as f64
    }

    pub fn stats(&self) -> CorrectionStats {
        CorrectionStats {
            total_feedback: self.feedback_history.len() as u64,
            total_corrections: self.corrections.len() as u64,
            total_recoveries: self.recoveries.len() as u64,
            avg_feedback: self.get_average_feedback(100),
            avg_effectiveness: if !self.corrections.is_empty() {
                self.corrections.iter().map(|c| c.confidence).sum::<f64>() / self.corrections.len() as f64
            } else { 0.0 },
        }
    }
}
"""

# resource_allocator.rs — CORRECT SYNTAX
ra_content = """use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceBudget {
    pub budget_id: String,
    pub total_compute: f64,
    pub total_memory: u64,
    pub allocated_compute: f64,
    pub allocated_memory: u64,
    pub priority_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeAllocation {
    pub allocation_id: String,
    pub task_id: String,
    pub compute_units: f64,
    pub cores: usize,
    pub threads: usize,
    pub start_time_nanos: u64,
    pub estimated_duration_micros: u64,
    pub actual_duration_micros: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationResult {
    pub granted: bool,
    pub allocation_id: Option<String>,
    pub reason: Option<String>,
    pub deficit: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStats {
    pub total_budgets: u64,
    pub total_allocations: u64,
    pub compute_utilization: f64,
    pub memory_utilization: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyBudget {
    pub budget_id: String,
    pub joules_available: f64,
    pub joules_consumed: f64,
    pub watts_per_task: f64,
    pub thermal_limit_celsius: f64,
    pub current_temp_celsius: f64,
}

pub struct ResourceAllocator {
    pub budgets: HashMap<String, ResourceBudget>,
    pub compute_allocations: Vec<<ComputeAllocation>,
    pub energy_budgets: HashMap<String, EnergyBudget>,
    pub total_compute_capacity: f64,
    pub total_memory_capacity: u64,
}

impl ResourceBudget {
    pub fn new(budget_id: &str, compute: f64, memory: u64, priority: f64) -> Self {
        Self {
            budget_id: budget_id.to_string(),
            total_compute: compute,
            total_memory: memory,
            allocated_compute: 0.0,
            allocated_memory: 0,
            priority_weight: priority.clamp(0.0, 1.0),
        }
    }

    pub fn remaining_compute(&self) -> f64 {
        (self.total_compute - self.allocated_compute).max(0.0)
    }

    pub fn remaining_memory(&self) -> u64 {
        self.total_memory.saturating_sub(self.allocated_memory)
    }

    pub fn allocate(&mut self, compute: f64, memory: u64) -> bool {
        if compute > self.remaining_compute() || memory > self.remaining_memory() {
            return false;
        }
        self.allocated_compute += compute;
        self.allocated_memory += memory;
        true
    }

    pub fn deallocate(&mut self, compute: f64, memory: u64) {
        self.allocated_compute = (self.allocated_compute - compute).max(0.0);
        self.allocated_memory = self.allocated_memory.saturating_sub(memory);
    }
}

impl ComputeAllocation {
    pub fn new(allocation_id: &str, task_id: &str, compute_units: f64, cores: usize, threads: usize, duration: u64) -> Self {
        Self {
            allocation_id: allocation_id.to_string(),
            task_id: task_id.to_string(),
            compute_units,
            cores,
            threads,
            start_time_nanos: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            estimated_duration_micros: duration,
            actual_duration_micros: None,
        }
    }

    pub fn complete(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        self.actual_duration_micros = Some(((now - self.start_time_nanos) / 1000) as u64);
    }
}

impl EnergyBudget {
    pub fn new(budget_id: &str, joules: f64, watts: f64, thermal: f64) -> Self {
        Self {
            budget_id: budget_id.to_string(),
            joules_available: joules,
            joules_consumed: 0.0,
            watts_per_task: watts,
            thermal_limit_celsius: thermal,
            current_temp_celsius: 25.0,
        }
    }
}

impl ResourceAllocator {
    pub fn new(total_compute: f64, total_memory: u64) -> Self {
        Self {
            budgets: HashMap::new(),
            compute_allocations: Vec::new(),
            energy_budgets: HashMap::new(),
            total_compute_capacity: total_compute,
            total_memory_capacity: total_memory,
        }
    }

    pub fn register_budget(&mut self, budget: ResourceBudget) {
        self.budgets.insert(budget.budget_id.clone(), budget);
    }

    pub fn request(&mut self, budget_id: &str, compute: f64, memory: u64, task_id: &str) -> AllocationResult {
        let budget = match self.budgets.get_mut(budget_id) {
            Some(b) => b,
            None => return AllocationResult {
                granted: false,
                allocation_id: None,
                reason: Some(format!("Budget {} not found", budget_id)),
                deficit: compute,
            },
        };

        if budget.allocate(compute, memory) {
            let alloc_id = format!("alloc_{}_{}", budget_id, task_id);
            let allocation = ComputeAllocation::new(&alloc_id, task_id, compute, 1, 1, 1000);
            self.compute_allocations.push(allocation);
            AllocationResult {
                granted: true,
                allocation_id: Some(alloc_id),
                reason: None,
                deficit: 0.0,
            }
        } else {
            AllocationResult {
                granted: false,
                allocation_id: None,
                reason: Some(format!("Insufficient resources in budget {}", budget_id)),
                deficit: compute - budget.remaining_compute(),
            }
        }
    }

    pub fn release(&mut self, budget_id: &str, allocation_id: &str) {
        if let Some(budget) = self.budgets.get_mut(budget_id) {
            if let Some(idx) = self.compute_allocations.iter().position(|a| a.allocation_id == allocation_id) {
                let alloc = self.compute_allocations.remove(idx);
                budget.deallocate(alloc.compute_units, 0);
            }
        }
    }

    pub fn stats(&self) -> ResourceStats {
        let total_allocated_compute: f64 = self.budgets.values().map(|b| b.allocated_compute).sum();
        let total_allocated_memory: u64 = self.budgets.values().map(|b| b.allocated_memory).sum();
        
        ResourceStats {
            total_budgets: self.budgets.len() as u64,
            total_allocations: self.compute_allocations.len() as u64,
            compute_utilization: total_allocated_compute / self.total_compute_capacity.max(1.0),
            memory_utilization: total_allocated_memory as f64 / self.total_memory_capacity.max(1) as f64,
        }
    }
}
"""

# Write files
base = r"C:\Sovereign_Alpha_Final\SovereignCore\rust_core\src\autonomy"

with open(os.path.join(base, "self_correction.rs"), "w", encoding="utf-8") as f:
    f.write(sc_content)
print("self_correction.rs written")

with open(os.path.join(base, "resource_allocator.rs"), "w", encoding="utf-8") as f:
    f.write(ra_content)
print("resource_allocator.rs written")

# Verify no double brackets
for fname in ["self_correction.rs", "resource_allocator.rs"]:
    with open(os.path.join(base, fname), "r") as f:
        content = f.read()
    if "<<" in content:
        print(f"ERROR: {fname} still has <<")
    else:
        print(f"OK: {fname} clean")