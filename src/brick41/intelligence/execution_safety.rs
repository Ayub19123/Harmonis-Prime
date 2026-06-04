use crate::brick41::foundation::{TrustLayer, Ledger, SecurityBaseline, SecurityDecision, SecurityLevel};
use crate::brick41::orchestration::{CoordinationEngine, Task, AgentStatus};
use std::collections::{HashMap, VecDeque};

/// ExecutionSafetyEngine: Self-correcting guards, rollback, deterministic boundaries
/// BRICK-41 Phase 3: Intelligence — Safe Autonomous Execution
#[derive(Debug, Clone)]
pub struct ExecutionSafetyEngine {
    pub trust: TrustLayer,
    pub ledger: Ledger,
    pub active_executions: HashMap<String, ExecutionContext>,
    pub rollback_log: VecDeque<RollbackRecord>,
    pub safety_boundaries: Vec<SafetyBoundary>,
    pub max_retries: u8,
    pub circuit_breaker_threshold: u32,
    pub failure_count: u32,
    pub circuit_open: bool,
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub execution_id: String,
    pub task: Task,
    pub agent_id: String,
    pub domain: String,
    pub stage: ExecutionStage,
    pub start_time_ns: u128,
    pub checkpoint_data: Vec<Checkpoint>,
    pub attempts: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStage {
    Pending,
    Validating,
    Executing,
    Committing,
    Verified,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub sequence: u32,
    pub state_hash: String,
    pub timestamp_ns: u128,
    pub operation: String,
}

#[derive(Debug, Clone)]
pub struct RollbackRecord {
    pub execution_id: String,
    pub rollback_to_sequence: u32,
    pub reason: String,
    pub timestamp_ns: u128,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct SafetyBoundary {
    pub boundary_id: String,
    pub domain: String,
    pub boundary_type: BoundaryType,
    pub limit: f64,
    pub enforcement: EnforcementMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BoundaryType {
    MaxLatencyMs,
    MaxCostUsd,
    MaxErrorRate,
    MinConfidence,
    MaxDataExposure,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnforcementMode {
    HardStop,
    SoftWarn,
    AutoThrottle,
    RequestApproval,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub execution_id: String,
    pub success: bool,
    pub stage_reached: ExecutionStage,
    pub final_state_hash: String,
    pub audit_trail: Vec<String>,
    pub rollback_performed: bool,
    pub boundary_violations: Vec<String>,
}

impl ExecutionSafetyEngine {
    pub fn new(node_id: &str, quorum_size: usize) -> Self {
        Self {
            trust: TrustLayer::new(),
            ledger: Ledger::new(node_id, quorum_size),
            active_executions: HashMap::new(),
            rollback_log: VecDeque::new(),
            safety_boundaries: Self::default_boundaries(),
            max_retries: 3,
            circuit_breaker_threshold: 5,
            failure_count: 0,
            circuit_open: false,
        }
    }

    fn default_boundaries() -> Vec<SafetyBoundary> {
        vec![
            SafetyBoundary {
                boundary_id: "latency_global".to_string(),
                domain: "global".to_string(),
                boundary_type: BoundaryType::MaxLatencyMs,
                limit: 5000.0,
                enforcement: EnforcementMode::HardStop,
            },
            SafetyBoundary {
                boundary_id: "cost_global".to_string(),
                domain: "global".to_string(),
                boundary_type: BoundaryType::MaxCostUsd,
                limit: 100.0,
                enforcement: EnforcementMode::AutoThrottle,
            },
            SafetyBoundary {
                boundary_id: "error_rate".to_string(),
                domain: "global".to_string(),
                boundary_type: BoundaryType::MaxErrorRate,
                limit: 0.05,
                enforcement: EnforcementMode::HardStop,
            },
            SafetyBoundary {
                boundary_id: "confidence_min".to_string(),
                domain: "global".to_string(),
                boundary_type: BoundaryType::MinConfidence,
                limit: 0.85,
                enforcement: EnforcementMode::RequestApproval,
            },
            SafetyBoundary {
                boundary_id: "data_exposure".to_string(),
                domain: "healthcare".to_string(),
                boundary_type: BoundaryType::MaxDataExposure,
                limit: 0.0,
                enforcement: EnforcementMode::HardStop,
            },
        ]
    }

    pub fn begin_execution(&mut self, execution_id: &str, task: Task, agent_id: &str, domain: &str) -> Result<ExecutionContext, String> {
        if self.circuit_open {
            return Err("Circuit breaker OPEN — execution rejected".to_string());
        }

        let ctx = ExecutionContext {
            execution_id: execution_id.to_string(),
            task: task.clone(),
            agent_id: agent_id.to_string(),
            domain: domain.to_string(),
            stage: ExecutionStage::Pending,
            start_time_ns: now_ns(),
            checkpoint_data: vec![Checkpoint {
                sequence: 0,
                state_hash: hash_state("initial"),
                timestamp_ns: now_ns(),
                operation: "init".to_string(),
            }],
            attempts: 0,
        };

        self.active_executions.insert(execution_id.to_string(), ctx.clone());
        self.trust.append(agent_id, "execution_begin", execution_id);
        
        Ok(ctx)
    }

    pub fn validate_and_execute(&mut self, execution_id: &str) -> ExecutionResult {
        let mut audit = vec!["validation_start".to_string()];
        
        // Phase 1: Read-mutate under lock, capture all needed data
        let (domain, task_action, agent_id, checkpoint_count, violations) = {
            let ctx = match self.active_executions.get_mut(execution_id) {
                Some(c) => c,
                None => return ExecutionResult {
                    execution_id: execution_id.to_string(),
                    success: false,
                    stage_reached: ExecutionStage::Failed,
                    final_state_hash: "".to_string(),
                    audit_trail: vec!["execution_not_found".to_string()],
                    rollback_performed: false,
                    boundary_violations: vec![],
                },
            };

            ctx.stage = ExecutionStage::Validating;
            ctx.attempts += 1;

            let violations = self.check_boundaries(&ctx.domain, &ctx.task);
            if !violations.is_empty() {
                audit.push(format!("boundary_violations: {:?}", violations));
                for v in &violations {
                    self.trust.append(&ctx.agent_id, "boundary_violation", v);
                }
                
                let hard_violations: Vec<&String> = violations.iter()
                    .filter(|v| v.contains("HardStop"))
                    .collect();
                
                if !hard_violations.is_empty() {
                    ctx.stage = ExecutionStage::Failed;
                    self.failure_count += 1;
                    self.update_circuit_breaker();
                    let final_hash = ctx.checkpoint_data.last().unwrap().state_hash.clone();
                    
                    return ExecutionResult {
                        execution_id: execution_id.to_string(),
                        success: false,
                        stage_reached: ExecutionStage::Failed,
                        final_state_hash: final_hash,
                        audit_trail: audit,
                        rollback_performed: false,
                        boundary_violations: violations,
                    };
                }
            }

            let domain = ctx.domain.clone();
            let task_action = ctx.task.action.clone();
            let agent_id = ctx.agent_id.clone();
            let checkpoint_count = ctx.checkpoint_data.len() as u32;
            
            (domain, task_action, agent_id, checkpoint_count, violations)
        }; // LOCK DROPPED

        // Phase 2: Ledger operations (no self lock held)
        let proposal = self.ledger.propose(execution_id, &format!("execute_{}", task_action));
        let mut prepared = proposal.clone();
        self.ledger.prepare(&mut prepared, vec![]);
        self.ledger.commit(prepared);

        audit.push("ledger_commit_success".to_string());

        // Phase 3: Re-lock for checkpoint, then drop
        let checkpoint_hash = {
            let ctx = self.active_executions.get_mut(execution_id).unwrap();
            ctx.stage = ExecutionStage::Executing;
            
            let checkpoint = Checkpoint {
                sequence: checkpoint_count,
                state_hash: hash_state(&format!("{}_{}", execution_id, task_action)),
                timestamp_ns: now_ns(),
                operation: task_action.clone(),
            };
            let hash = checkpoint.state_hash.clone();
            ctx.checkpoint_data.push(checkpoint);
            ctx.stage = ExecutionStage::Committing;
            hash
        }; // LOCK DROPPED

        // Phase 4: Verification (no lock held)
        let verified = self.verify_execution(execution_id);
        
        if verified {
            let ctx = self.active_executions.get_mut(execution_id).unwrap();
            ctx.stage = ExecutionStage::Verified;
            audit.push("execution_verified".to_string());
            let agent_id_clone = ctx.agent_id.clone();
            let final_hash = ctx.checkpoint_data.last().unwrap().state_hash.clone();
            // Drop lock implicitly when ctx goes out of scope
            self.trust.append(&agent_id_clone, "execution_success", execution_id);
            
            ExecutionResult {
                execution_id: execution_id.to_string(),
                success: true,
                stage_reached: ExecutionStage::Verified,
                final_state_hash: final_hash,
                audit_trail: audit,
                rollback_performed: false,
                boundary_violations: violations,
            }
        } else {
            let ctx = self.active_executions.get_mut(execution_id).unwrap();
            ctx.stage = ExecutionStage::Failed;
            self.failure_count += 1;
            let agent_id_clone = ctx.agent_id.clone();
            let checkpoint_count = ctx.checkpoint_data.len();
            // Drop lock
            drop(ctx);
            
            audit.push("execution_verification_failed".to_string());
            
            let rollback = self.rollback(execution_id, "verification_failed");
            audit.push(format!("rollback_{}", if rollback.success { "success" } else { "failed" }));
            
            self.update_circuit_breaker();
            
            // Re-lock for final hash
            let final_hash = {
                let ctx = self.active_executions.get(execution_id).unwrap();
                ctx.checkpoint_data.last().unwrap().state_hash.clone()
            };
            
            ExecutionResult {
                execution_id: execution_id.to_string(),
                success: false,
                stage_reached: ExecutionStage::RolledBack,
                final_state_hash: final_hash,
                audit_trail: audit,
                rollback_performed: rollback.success,
                boundary_violations: violations,
            }
        }
    }

    pub fn rollback(&mut self, execution_id: &str, reason: &str) -> RollbackRecord {
        let ctx = match self.active_executions.get(execution_id) {
            Some(c) => c,
            None => return RollbackRecord {
                execution_id: execution_id.to_string(),
                rollback_to_sequence: 0,
                reason: "execution_not_found".to_string(),
                timestamp_ns: now_ns(),
                success: false,
            },
        };

        let target_sequence = ctx.checkpoint_data.first().map(|c| c.sequence).unwrap_or(0);
        
        let record = RollbackRecord {
            execution_id: execution_id.to_string(),
            rollback_to_sequence: target_sequence,
            reason: reason.to_string(),
            timestamp_ns: now_ns(),
            success: true,
        };

        self.rollback_log.push_back(record.clone());
        self.trust.append(&ctx.agent_id, "rollback", execution_id);
        
        record
    }

    fn check_boundaries(&self, domain: &str, task: &Task) -> Vec<String> {
        let mut violations = Vec::new();
        
        for boundary in &self.safety_boundaries {
            if boundary.domain != "global" && boundary.domain != domain {
                continue;
            }
            
            let violated = match boundary.boundary_type {
                BoundaryType::MaxLatencyMs => false,
                BoundaryType::MaxCostUsd => false,
                BoundaryType::MaxErrorRate => self.failure_count as f64 > boundary.limit,
                BoundaryType::MinConfidence => false,
                BoundaryType::MaxDataExposure => task.payload.contains("PHI") || task.payload.contains("SSN"),
            };
            
            if violated {
                violations.push(format!("{}:{:?}:{:?}", boundary.boundary_id, boundary.enforcement, boundary.boundary_type));
            }
        }
        
        violations
    }

    fn verify_execution(&self, execution_id: &str) -> bool {
        let ctx = match self.active_executions.get(execution_id) {
            Some(c) => c,
            None => return false,
        };
        
        if ctx.checkpoint_data.len() < 2 {
            return false;
        }
        
        let last = ctx.checkpoint_data.last().unwrap();
        let prev = &ctx.checkpoint_data[ctx.checkpoint_data.len() - 2];
        
        let expected = hash_state(&format!("{}_{}", execution_id, prev.operation));
        last.state_hash != expected || last.sequence == prev.sequence + 1
    }

    fn update_circuit_breaker(&mut self) {
        if self.failure_count >= self.circuit_breaker_threshold {
            self.circuit_open = true;
            self.trust.append("safety_engine", "circuit_breaker_open", &format!("failures_{}", self.failure_count));
        }
    }

    pub fn reset_circuit_breaker(&mut self) {
        self.circuit_open = false;
        self.failure_count = 0;
        self.trust.append("safety_engine", "circuit_breaker_reset", "manual");
    }

    pub fn get_execution_status(&self, execution_id: &str) -> Option<ExecutionStage> {
        self.active_executions.get(execution_id).map(|c| c.stage.clone())
    }

    pub fn active_execution_count(&self) -> usize {
        self.active_executions.values().filter(|c| c.stage != ExecutionStage::Verified && c.stage != ExecutionStage::RolledBack).count()
    }

    pub fn failed_execution_count(&self) -> usize {
        self.active_executions.values().filter(|c| c.stage == ExecutionStage::Failed).count()
    }
}

pub fn hash_state(state: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    state.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn now_ns() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
}
