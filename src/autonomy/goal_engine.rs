use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Goal: A high-level objective with constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub goal_id: String,
    pub description: String,
    pub priority: GoalPriority,
    pub constraints: Vec<String>,
    pub sub_goals: Vec<Goal>,
    pub status: AchievementStatus,
    pub progress: f64,
    pub created_nanos: u64,
    pub deadline_nanos: Option<u64>,
    pub substrate_source: String,
}

/// GoalPriority: Urgency classification
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GoalPriority {
    Critical,
    High,
    Medium,
    Low,
    Background,
}

/// AchievementStatus: State machine
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementStatus {
    Pending,
    Active,
    Blocked,
    Partial,
    Completed,
    Failed,
    Abandoned,
}

/// Task: Atomic executable unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_id: String,
    pub action: String,
    pub prerequisites: Vec<String>,
    pub estimated_cost: ResourceEstimate,
    pub actual_cost: Option<ResourceEstimate>,
    pub status: TaskStatus,
    pub assigned_goal: String,
}

/// TaskStatus: Execution state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Ready,
    Running,
    Waiting,
    Completed,
    Failed,
    Retrying,
}

/// ResourceEstimate: Predicted consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEstimate {
    pub compute_units: f64,
    pub memory_bytes: u64,
    pub time_micros: u64,
    pub energy_joules: f64,
}

/// ActionPlan: Sequenced operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPlan {
    pub plan_id: String,
    pub goal_id: String,
    pub tasks: Vec<Task>,
    pub total_estimated_cost: ResourceEstimate,
    pub critical_path: Vec<String>,
    pub parallel_groups: Vec<Vec<String>>,
}

/// GoalEngine: Goal decomposition system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalEngine {
    pub goals: VecDeque<Goal>,
    pub active_plans: VecDeque<ActionPlan>,
    pub completed_goals: Vec<Goal>,
    pub failed_goals: Vec<Goal>,
    pub max_goals: usize,
    pub max_plans: usize,
    pub global_goal_counter: u64,
}

/// GoalEngineStats: Observability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalEngineStats {
    pub total_goals_defined: u64,
    pub active_goals: u64,
    pub completed_goals: u64,
    pub failed_goals: u64,
    pub active_plans: u64,
    pub average_progress: f64,
}

impl GoalPriority {
    pub fn weight(&self) -> f64 {
        match self {
            GoalPriority::Critical => 1000.0,
            GoalPriority::High => 100.0,
            GoalPriority::Medium => 10.0,
            GoalPriority::Low => 1.0,
            GoalPriority::Background => 0.1,
        }
    }
}

impl Goal {
    pub fn new(goal_id: &str, description: &str, priority: GoalPriority, substrate: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self {
            goal_id: goal_id.to_string(),
            description: description.to_string(),
            priority,
            constraints: Vec::new(),
            sub_goals: Vec::new(),
            status: AchievementStatus::Pending,
            progress: 0.0,
            created_nanos: now,
            deadline_nanos: None,
            substrate_source: substrate.to_string(),
        }
    }

    pub fn constrain(&mut self, constraint: &str) {
        self.constraints.push(constraint.to_string());
    }

    pub fn decompose(&mut self, sub_goals: Vec<Goal>) {
        self.sub_goals = sub_goals;
        if !self.sub_goals.is_empty() {
            self.status = AchievementStatus::Active;
        }
    }

    pub fn update_progress(&mut self) {
        if self.sub_goals.is_empty() {
            self.progress = match self.status {
                AchievementStatus::Completed => 1.0,
                _ => 0.0,
            };
            return;
        }
        let total = self.sub_goals.len() as f64;
        let completed = self
            .sub_goals
            .iter()
            .filter(|g| g.status == AchievementStatus::Completed)
            .count() as f64;
        self.progress = (completed / total).clamp(0.0, 1.0);
        self.status = if self.progress >= 1.0 {
            AchievementStatus::Completed
        } else if self.progress > 0.0 {
            AchievementStatus::Partial
        } else {
            AchievementStatus::Active
        };
    }

    pub fn is_overdue(&self) -> bool {
        match self.deadline_nanos {
            Some(deadline) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;
                now > deadline
            }
            None => false,
        }
    }

    pub fn constraints_satisfied(&self, context: &[String]) -> bool {
        self.constraints.iter().all(|c| context.contains(c))
    }
}

impl Task {
    pub fn new(
        task_id: &str,
        action: &str,
        prerequisites: Vec<String>,
        estimated_cost: ResourceEstimate,
        goal_id: &str,
    ) -> Self {
        Self {
            task_id: task_id.to_string(),
            action: action.to_string(),
            prerequisites,
            estimated_cost,
            actual_cost: None,
            status: TaskStatus::Waiting,
            assigned_goal: goal_id.to_string(),
        }
    }

    pub fn check_readiness(&mut self, completed_tasks: &[String]) {
        if self
            .prerequisites
            .iter()
            .all(|p| completed_tasks.contains(p))
        {
            self.status = TaskStatus::Ready;
        }
    }
}

impl ResourceEstimate {
    pub fn zero() -> Self {
        Self {
            compute_units: 0.0,
            memory_bytes: 0,
            time_micros: 0,
            energy_joules: 0.0,
        }
    }

    pub fn add(&self, other: &ResourceEstimate) -> ResourceEstimate {
        ResourceEstimate {
            compute_units: self.compute_units + other.compute_units,
            memory_bytes: self.memory_bytes + other.memory_bytes,
            time_micros: self.time_micros + other.time_micros,
            energy_joules: self.energy_joules + other.energy_joules,
        }
    }
}

impl ActionPlan {
    pub fn new(plan_id: &str, goal_id: &str) -> Self {
        Self {
            plan_id: plan_id.to_string(),
            goal_id: goal_id.to_string(),
            tasks: Vec::new(),
            total_estimated_cost: ResourceEstimate::zero(),
            critical_path: Vec::new(),
            parallel_groups: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.total_estimated_cost = self.total_estimated_cost.add(&task.estimated_cost);
        self.tasks.push(task);
    }

    pub fn compute_critical_path(&mut self) {
        let mut task_prereq_count: Vec<(String, usize)> = self
            .tasks
            .iter()
            .map(|t| (t.task_id.clone(), t.prerequisites.len()))
            .collect();
        task_prereq_count.sort_by(|a, b| b.1.cmp(&a.1));
        let critical_count = (self.tasks.len() as f64 * 0.2).ceil() as usize;
        self.critical_path = task_prereq_count
            .iter()
            .take(critical_count.max(1))
            .map(|(id, _)| id.clone())
            .collect();
    }

    pub fn compute_parallel_groups(&mut self) {
        let mut groups: Vec<Vec<String>> = Vec::new();
        let mut assigned: Vec<String> = Vec::new();
        for task in &self.tasks {
            if assigned.contains(&task.task_id) {
                continue;
            }
            let mut group = vec![task.task_id.clone()];
            assigned.push(task.task_id.clone());
            for other in &self.tasks {
                if assigned.contains(&other.task_id) {
                    continue;
                }
                let no_mutual_deps = !task.prerequisites.contains(&other.task_id)
                    && !other.prerequisites.contains(&task.task_id);
                if no_mutual_deps {
                    group.push(other.task_id.clone());
                    assigned.push(other.task_id.clone());
                }
            }
            if group.len() > 1 {
                groups.push(group);
            }
        }
        self.parallel_groups = groups;
    }
}

impl GoalEngine {
    pub fn new(max_goals: usize, max_plans: usize) -> Self {
        Self {
            goals: VecDeque::with_capacity(max_goals),
            active_plans: VecDeque::with_capacity(max_plans),
            completed_goals: Vec::new(),
            failed_goals: Vec::new(),
            max_goals,
            max_plans,
            global_goal_counter: 0,
        }
    }

    pub fn define_goal(
        &mut self,
        description: &str,
        priority: GoalPriority,
        substrate: &str,
    ) -> String {
        self.global_goal_counter += 1;
        let goal_id = format!("goal_{}", self.global_goal_counter);
        let goal = Goal::new(&goal_id, description, priority, substrate);
        if self.goals.len() >= self.max_goals {
            self.goals.pop_front();
        }
        self.goals.push_back(goal);
        goal_id
    }

    pub fn plan(&mut self, goal_id: &str, tasks: Vec<Task>) -> Result<String, String> {
        let goal = self
            .goals
            .iter_mut()
            .find(|g| g.goal_id == goal_id)
            .ok_or("Goal not found")?;
        let plan_id = format!("plan_{}", goal_id);
        let mut plan = ActionPlan::new(&plan_id, goal_id);
        for task in tasks {
            plan.add_task(task);
        }
        plan.compute_critical_path();
        plan.compute_parallel_groups();
        goal.status = AchievementStatus::Active;
        if self.active_plans.len() >= self.max_plans {
            self.active_plans.pop_front();
        }
        self.active_plans.push_back(plan);
        Ok(plan_id)
    }

    /// CORRECTED: next_task without double mutable borrow
    pub fn next_task(&mut self) -> Option<&mut Task> {
        let mut best_idx: Option<usize> = None;
        let mut best_priority: Option<GoalPriority> = None;

        for (idx, plan) in self.active_plans.iter().enumerate() {
            if plan.tasks.iter().any(|t| t.status == TaskStatus::Ready) {
                if let Some(goal) = self.goals.iter().find(|g| g.goal_id == plan.goal_id) {
                    if best_priority.is_none() || goal.priority > best_priority.clone().unwrap() {
                        best_priority = Some(goal.priority.clone());
                        best_idx = Some(idx);
                    }
                }
            }
        }

        if let Some(idx) = best_idx {
            return self.active_plans[idx]
                .tasks
                .iter_mut()
                .find(|t| t.status == TaskStatus::Ready);
        }
        None
    }

    pub fn complete_task(&mut self, task_id: &str) -> Result<(), String> {
        for plan in self.active_plans.iter_mut() {
            if let Some(task) = plan.tasks.iter_mut().find(|t| t.task_id == task_id) {
                task.status = TaskStatus::Completed;
                if let Some(goal) = self
                    .goals
                    .iter_mut()
                    .find(|g| g.goal_id == task.assigned_goal)
                {
                    goal.update_progress();
                }
                return Ok(());
            }
        }
        Err("Task not found".to_string())
    }

    pub fn top_priority_goal(&self) -> Option<&Goal> {
        self.goals
            .iter()
            .filter(|g| {
                g.status == AchievementStatus::Active || g.status == AchievementStatus::Partial
            })
            .max_by(|a, b| a.priority.cmp(&b.priority))
    }

    pub fn stats(&self) -> GoalEngineStats {
        GoalEngineStats {
            total_goals_defined: self.global_goal_counter,
            active_goals: self
                .goals
                .iter()
                .filter(|g| g.status == AchievementStatus::Active)
                .count() as u64,
            completed_goals: self.completed_goals.len() as u64,
            failed_goals: self.failed_goals.len() as u64,
            active_plans: self.active_plans.len() as u64,
            average_progress: if !self.goals.is_empty() {
                self.goals.iter().map(|g| g.progress).sum::<f64>() / self.goals.len() as f64
            } else {
                0.0
            },
        }
    }
}
