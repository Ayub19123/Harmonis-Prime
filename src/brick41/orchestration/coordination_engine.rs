use crate::brick41::foundation::{Ledger, TrustLayer};
use std::collections::{HashMap, VecDeque};

/// BRICK-41 Phase 2: Orchestration - Multi-Agent Coordination Engine
/// Byzantine fault-tolerant task distribution with event-driven consensus
#[derive(Debug, Clone)]
pub struct CoordinationEngine {
    pub node_id: String,
    pub trust: TrustLayer,
    pub ledger: Ledger,
    pub agents: HashMap<String, Agent>,
    pub task_queue: VecDeque<Task>,
    pub event_log: Vec<CoordinationEvent>,
    pub quorum_size: usize,
}

#[derive(Debug, Clone)]
pub struct Agent {
    pub agent_id: String,
    pub domain: String,
    pub status: AgentStatus,
    pub capabilities: Vec<String>,
    pub last_heartbeat_ns: u128,
    pub task_count: u32,
    pub failure_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Idle,
    Busy,
    Offline,
    Suspended,
    Recovering,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub task_id: String,
    pub domain: String,
    pub action: String,
    pub payload: String,
    pub priority: u8,
    pub required_capabilities: Vec<String>,
    pub assigned_agent: Option<String>,
    pub status: TaskStatus,
    pub created_ns: u128,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Assigned,
    Executing,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone)]
pub struct CoordinationEvent {
    pub event_id: String,
    pub timestamp_ns: u128,
    pub event_type: EventType,
    pub agent_id: String,
    pub task_id: String,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    AgentRegistered,
    TaskAssigned,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    AgentHeartbeat,
    ConsensusReached,
    RollbackInitiated,
}

/// Alias for backward compatibility with mod.rs exports
pub type DomainEvent = CoordinationEvent;

/// Simple coordination result type
#[derive(Debug, Clone, PartialEq)]
pub struct CoordinationResult {
    pub success: bool,
    pub task_id: String,
    pub agent_id: String,
    pub details: String,
}

impl CoordinationEngine {
    pub fn new(node_id: &str, quorum_size: usize) -> Self {
        Self {
            node_id: node_id.to_string(),
            trust: TrustLayer::new(),
            ledger: Ledger::new(node_id, quorum_size),
            agents: HashMap::new(),
            task_queue: VecDeque::new(),
            event_log: Vec::new(),
            quorum_size,
        }
    }

    pub fn register_agent(
        &mut self,
        agent_id: &str,
        domain: &str,
        capabilities: Vec<String>,
    ) -> bool {
        if self.agents.contains_key(agent_id) {
            return false;
        }

        let agent = Agent {
            agent_id: agent_id.to_string(),
            domain: domain.to_string(),
            status: AgentStatus::Idle,
            capabilities,
            last_heartbeat_ns: now_ns(),
            task_count: 0,
            failure_count: 0,
        };

        self.agents.insert(agent_id.to_string(), agent);
        self.trust
            .append("coordination", "agent_registered", agent_id);
        self.event_log.push(CoordinationEvent {
            event_id: format!("evt_{}", now_ns()),
            timestamp_ns: now_ns(),
            event_type: EventType::AgentRegistered,
            agent_id: agent_id.to_string(),
            task_id: "".to_string(),
            details: format!("domain: {}", domain),
        });

        true
    }

    pub fn submit_task(&mut self, task: Task) -> bool {
        self.task_queue.push_back(task.clone());
        self.trust
            .append("coordination", "task_submitted", &task.task_id);
        true
    }

    pub fn assign_next_task(&mut self) -> Option<Task> {
        if self.task_queue.is_empty() {
            return None;
        }

        let mut best_agent: Option<String> = None;
        let mut best_score: i32 = -1;

        // Find best idle agent for next task
        for (agent_id, agent) in &self.agents {
            if agent.status != AgentStatus::Idle {
                continue;
            }
            let score = agent.capabilities.len() as i32;
            if score > best_score {
                best_score = score;
                best_agent = Some(agent_id.clone());
            }
        }

        let agent_id = best_agent?;
        let mut task = self.task_queue.pop_front()?;

        task.assigned_agent = Some(agent_id.clone());
        task.status = TaskStatus::Assigned;

        if let Some(agent) = self.agents.get_mut(&agent_id) {
            agent.status = AgentStatus::Busy;
            agent.task_count += 1;
        }

        self.event_log.push(CoordinationEvent {
            event_id: format!("evt_{}", now_ns()),
            timestamp_ns: now_ns(),
            event_type: EventType::TaskAssigned,
            agent_id: agent_id.clone(),
            task_id: task.task_id.clone(),
            details: "".to_string(),
        });

        self.trust
            .append("coordination", "task_assigned", &task.task_id);

        Some(task)
    }

    pub fn report_task_complete(&mut self, agent_id: &str, task_id: &str, result: &str) -> bool {
        let proposal = self.ledger.propose(task_id, result);
        let mut prepared = proposal.clone();
        let ready = self.ledger.prepare(&mut prepared, vec![]);

        if !ready {
            return false;
        }

        let committed = self.ledger.commit(prepared);
        if !committed {
            return false;
        }

        if let Some(agent) = self.agents.get_mut(agent_id) {
            agent.status = AgentStatus::Idle;
        }

        self.event_log.push(CoordinationEvent {
            event_id: format!("evt_{}", now_ns()),
            timestamp_ns: now_ns(),
            event_type: EventType::TaskCompleted,
            agent_id: agent_id.to_string(),
            task_id: task_id.to_string(),
            details: result.to_string(),
        });

        self.trust.append("coordination", "task_completed", task_id);
        true
    }

    pub fn report_task_failure(&mut self, agent_id: &str, task_id: &str, reason: &str) -> bool {
        if let Some(agent) = self.agents.get_mut(agent_id) {
            agent.failure_count += 1;
            agent.status = AgentStatus::Idle;
            if agent.failure_count >= 3 {
                agent.status = AgentStatus::Suspended;
            }
        }

        self.event_log.push(CoordinationEvent {
            event_id: format!("evt_{}", now_ns()),
            timestamp_ns: now_ns(),
            event_type: EventType::TaskFailed,
            agent_id: agent_id.to_string(),
            task_id: task_id.to_string(),
            details: reason.to_string(),
        });

        self.trust.append("coordination", "task_failed", task_id);
        true
    }

    pub fn heartbeat(&mut self, agent_id: &str) -> bool {
        if let Some(agent) = self.agents.get_mut(agent_id) {
            agent.last_heartbeat_ns = now_ns();
            if agent.status == AgentStatus::Offline {
                agent.status = AgentStatus::Recovering;
            }
            true
        } else {
            false
        }
    }

    pub fn get_agent_status(&self, agent_id: &str) -> Option<AgentStatus> {
        self.agents.get(agent_id).map(|a| a.status.clone())
    }

    pub fn active_agent_count(&self) -> usize {
        self.agents
            .values()
            .filter(|a| a.status != AgentStatus::Offline && a.status != AgentStatus::Suspended)
            .count()
    }

    pub fn pending_task_count(&self) -> usize {
        self.task_queue.len()
    }

    pub fn get_domain_events(&self, domain: &str) -> Vec<CoordinationEvent> {
        self.event_log
            .iter()
            .filter(|e| {
                if let Some(agent) = self.agents.get(&e.agent_id) {
                    agent.domain == domain
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
}

pub fn now_ns() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
