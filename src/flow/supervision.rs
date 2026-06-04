use crate::flow::actor_system::{ActorId, ActorSystem};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupervisionStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
}

#[derive(Debug, Clone)]
pub struct SupervisorPolicy {
    pub max_restarts: u32,
    pub restart_window_seconds: u64,
    pub strategy: SupervisionStrategy,
}

pub struct SupervisionTree {
    pub supervisors: HashMap<ActorId, SupervisorPolicy>,
    pub restart_counts: HashMap<ActorId, Vec<u64>>,
    pub child_map: HashMap<ActorId, Vec<ActorId>>,
}

impl SupervisionTree {
    pub fn new() -> Self {
        Self {
            supervisors: HashMap::new(),
            child_map: HashMap::new(),
            restart_counts: HashMap::new(),
        }
    }

    pub fn register_supervisor(
        &mut self,
        supervisor_id: ActorId,
        policy: SupervisorPolicy,
        children: Vec<ActorId>,
    ) {
        self.supervisors.insert(supervisor_id.clone(), policy);
        self.child_map.insert(supervisor_id.clone(), children);
        self.restart_counts.insert(supervisor_id, Vec::new());
    }

    pub fn handle_failure(
        &mut self,
        actor_system: &mut ActorSystem,
        failed_actor: &ActorId,
    ) -> Vec<ActorId> {
        let mut restarted = Vec::new();

        for (supervisor_id, policy) in &self.supervisors {
            let children = self
                .child_map
                .get(supervisor_id)
                .cloned()
                .unwrap_or_default();

            if !children.contains(failed_actor) {
                continue;
            }

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let restarts = self
                .restart_counts
                .entry(supervisor_id.clone())
                .or_default();
            restarts.retain(|&t| now - t < policy.restart_window_seconds);
            restarts.push(now);

            if restarts.len() as u32 > policy.max_restarts {
                for child in &children {
                    actor_system.kill_actor(child);
                }
                continue;
            }

            match policy.strategy {
                SupervisionStrategy::OneForOne => {
                    if let Some(state) = actor_system.get_actor_state(failed_actor) {
                        actor_system.kill_actor(failed_actor);
                        if actor_system.spawn_actor(failed_actor.clone(), state.actor_type.clone())
                        {
                            restarted.push(failed_actor.clone());
                        }
                    }
                }
                SupervisionStrategy::OneForAll => {
                    for child in &children {
                        if let Some(state) = actor_system.get_actor_state(child) {
                            actor_system.kill_actor(child);
                            if actor_system.spawn_actor(child.clone(), state.actor_type.clone()) {
                                restarted.push(child.clone());
                            }
                        }
                    }
                }
                SupervisionStrategy::RestForOne => {
                    let failed_idx = children.iter().position(|c| c == failed_actor).unwrap_or(0);
                    for child in children.iter().skip(failed_idx) {
                        if let Some(state) = actor_system.get_actor_state(child) {
                            actor_system.kill_actor(child);
                            if actor_system.spawn_actor(child.clone(), state.actor_type.clone()) {
                                restarted.push(child.clone());
                            }
                        }
                    }
                }
            }
        }

        restarted
    }

    pub fn get_restart_count(&self, supervisor_id: &ActorId) -> usize {
        self.restart_counts
            .get(supervisor_id)
            .map(|v| v.len())
            .unwrap_or(0)
    }
}
