use crate::flow::actor_system::{ActorId, ActorMessage, ActorSystem, MessageType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub task_id: String,
    pub priority: u8,
    pub cpu_estimate: f64,
    pub memory_estimate_mb: u64,
    pub target_actor_type: String,
    pub payload: String,
}

#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub task: TaskSpec,
    pub assigned_actor: Option<ActorId>,
    pub scheduled_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Scheduled,
    Running,
    Completed,
    Failed,
    Retrying,
}

pub struct TaskScheduler {
    pub pending_queue: Vec<TaskSpec>,
    pub running_tasks: HashMap<String, ScheduledTask>,
    pub completed_tasks: Vec<ScheduledTask>,
    pub max_concurrent: usize,
    pub retry_limit: u32,
}

impl TaskScheduler {
    pub fn new(max_concurrent: usize, retry_limit: u32) -> Self {
        Self {
            pending_queue: Vec::new(),
            running_tasks: HashMap::new(),
            completed_tasks: Vec::new(),
            max_concurrent: max_concurrent.max(1),
            retry_limit: retry_limit.max(1),
        }
    }

    pub fn submit_task(&mut self, task: TaskSpec) {
        self.pending_queue.push(task);
        self.pending_queue
            .sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn schedule_round(&mut self, actor_system: &mut ActorSystem) -> Vec<ScheduledTask> {
        let mut scheduled = Vec::new();
        let available_slots = self.max_concurrent.saturating_sub(self.running_tasks.len());

        for _ in 0..available_slots {
            if let Some(task) = self.pending_queue.pop() {
                let actor_id = format!("{}_{}", task.target_actor_type, task.task_id);

                if !actor_system.actors.contains_key(&actor_id) {
                    actor_system.spawn_actor(actor_id.clone(), task.target_actor_type.clone());
                }

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let scheduled_task = ScheduledTask {
                    task: task.clone(),
                    assigned_actor: Some(actor_id.clone()),
                    scheduled_at: now,
                    started_at: None,
                    completed_at: None,
                    status: TaskStatus::Scheduled,
                };

                let message = ActorMessage {
                    from: String::from("scheduler"),
                    to: actor_id,
                    payload: task.payload.clone(),
                    timestamp: now,
                    message_type: MessageType::Command,
                };

                actor_system.send_message(message);
                self.running_tasks
                    .insert(task.task_id.clone(), scheduled_task.clone());
                scheduled.push(scheduled_task);
            }
        }

        scheduled
    }

    pub fn complete_task(&mut self, task_id: &str, success: bool) -> Option<ScheduledTask> {
        if let Some(mut task) = self.running_tasks.remove(task_id) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            task.completed_at = Some(now);
            task.status = if success {
                TaskStatus::Completed
            } else {
                TaskStatus::Failed
            };

            if !success {
                let retries = self
                    .completed_tasks
                    .iter()
                    .filter(|t| t.task.task_id == task_id && t.status == TaskStatus::Retrying)
                    .count() as u32;

                if retries < self.retry_limit {
                    let mut retry_task = task.clone();
                    retry_task.status = TaskStatus::Retrying;
                    retry_task.completed_at = None;
                    self.submit_task(retry_task.task);
                }
            }

            self.completed_tasks.push(task.clone());
            Some(task)
        } else {
            None
        }
    }

    pub fn get_stats(&self) -> (usize, usize, usize) {
        (
            self.pending_queue.len(),
            self.running_tasks.len(),
            self.completed_tasks.len(),
        )
    }
}
