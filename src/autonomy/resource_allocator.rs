use serde::{Deserialize, Serialize};
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

pub struct ResourceAllocator {
    pub budgets: HashMap<String, ResourceBudget>,
    pub compute_allocations: Vec<ComputeAllocation>,
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
    pub fn new(
        allocation_id: &str,
        task_id: &str,
        compute_units: f64,
        cores: usize,
        threads: usize,
        duration: u64,
    ) -> Self {
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

impl ResourceAllocator {
    pub fn new(total_compute: f64, total_memory: u64) -> Self {
        Self {
            budgets: HashMap::new(),
            compute_allocations: Vec::new(),
            total_compute_capacity: total_compute,
            total_memory_capacity: total_memory,
        }
    }

    pub fn register_budget(&mut self, budget: ResourceBudget) {
        self.budgets.insert(budget.budget_id.clone(), budget);
    }

    pub fn request(
        &mut self,
        budget_id: &str,
        compute: f64,
        memory: u64,
        task_id: &str,
    ) -> AllocationResult {
        let budget = match self.budgets.get_mut(budget_id) {
            Some(b) => b,
            None => {
                return AllocationResult {
                    granted: false,
                    allocation_id: None,
                    reason: Some(format!("Budget {} not found", budget_id)),
                    deficit: compute,
                }
            }
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
            if let Some(idx) = self
                .compute_allocations
                .iter()
                .position(|a| a.allocation_id == allocation_id)
            {
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
            memory_utilization: total_allocated_memory as f64
                / self.total_memory_capacity.max(1) as f64,
        }
    }
}
