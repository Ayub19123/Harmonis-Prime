use crate::flow::actor_system::ActorSystem;
use crate::flow::supervision::{SupervisionStrategy, SupervisionTree, SupervisorPolicy};
use crate::flow::task_scheduler::{TaskScheduler, TaskSpec, TaskStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowMetrics {
    pub total_messages: u64,
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub active_actors: usize,
    pub supervisor_restarts: u64,
    pub throughput_ops_per_sec: f64,
}

pub struct FlowRuntime {
    pub actor_system: ActorSystem,
    pub task_scheduler: TaskScheduler,
    pub supervision_tree: SupervisionTree,
    pub runtime_id: String,
    pub start_time: u64,
    pub metrics: FlowMetrics,
}

impl FlowRuntime {
    pub fn new(runtime_id: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            actor_system: ActorSystem::new(runtime_id.clone()),
            task_scheduler: TaskScheduler::new(100, 3),
            supervision_tree: SupervisionTree::new(),
            runtime_id,
            start_time: now,
            metrics: FlowMetrics {
                total_messages: 0,
                total_tasks: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                active_actors: 0,
                supervisor_restarts: 0,
                throughput_ops_per_sec: 0.0,
            },
        }
    }

    pub fn initialize_cluster(&mut self, node_count: usize) {
        for i in 0..node_count {
            let node_id = format!("node_{}", i);
            self.actor_system
                .spawn_actor(node_id.clone(), String::from("cluster_node"));

            let policy = SupervisorPolicy {
                max_restarts: 5,
                restart_window_seconds: 60,
                strategy: SupervisionStrategy::OneForOne,
            };

            self.supervision_tree.register_supervisor(
                format!("supervisor_{}", i),
                policy,
                vec![node_id],
            );
        }
    }

    pub fn submit_flow_task(&mut self, task: TaskSpec) {
        self.task_scheduler.submit_task(task);
        self.metrics.total_tasks += 1;
    }

    pub fn execute_cycle(&mut self) -> Vec<TaskStatus> {
        let scheduled = self.task_scheduler.schedule_round(&mut self.actor_system);

        let mut statuses = Vec::new();
        for task in scheduled {
            if let Some(actor_id) = &task.assigned_actor {
                let messages = self.actor_system.process_mailbox(actor_id);
                self.metrics.total_messages += messages.len() as u64;

                let success = !messages.is_empty();
                if let Some(completed) = self
                    .task_scheduler
                    .complete_task(&task.task.task_id, success)
                {
                    match completed.status {
                        TaskStatus::Completed => self.metrics.completed_tasks += 1,
                        TaskStatus::Failed => {
                            self.metrics.failed_tasks += 1;
                            let restarted = self
                                .supervision_tree
                                .handle_failure(&mut self.actor_system, actor_id);
                            self.metrics.supervisor_restarts += restarted.len() as u64;
                        }
                        _ => {}
                    }
                    statuses.push(completed.status);
                }
            }
        }

        self.metrics.active_actors = self.actor_system.alive_count();
        let elapsed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_sub(self.start_time);

        if elapsed > 0 {
            self.metrics.throughput_ops_per_sec =
                self.metrics.completed_tasks as f64 / elapsed as f64;
        }

        statuses
    }

    pub fn get_metrics(&self) -> FlowMetrics {
        self.metrics.clone()
    }

    pub fn is_healthy(&self) -> bool {
        self.metrics.failed_tasks < self.metrics.completed_tasks.saturating_mul(2)
    }
}
