use serde::{Deserialize, Serialize};

/// Represents a TLA+ state variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVariable {
    pub name: String,
    pub domain: Vec<String>,
    pub current_value: String,
}

/// Represents a TLA+ action (state transition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlaAction {
    pub name: String,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
    pub enabled: bool,
}

/// Represents a TLA+ state machine specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlaSpec {
    pub spec_name: String,
    pub variables: Vec<StateVariable>,
    pub initial_predicate: String,
    pub actions: Vec<TlaAction>,
    pub next_state_relation: String,
    pub fairness_conditions: Vec<String>,
}

/// Specification for Raft consensus safety
#[derive(Debug, Clone)]
pub struct RaftSafetySpec {
    pub spec: TlaSpec,
    pub node_count: usize,
    pub quorum_size: usize,
}

impl RaftSafetySpec {
    pub fn new(node_count: usize) -> Self {
        let quorum = (node_count / 2) + 1;
        let mut variables = Vec::new();

        for i in 0..node_count {
            variables.push(StateVariable {
                name: format!("node_{}_log", i),
                domain: vec!["empty".to_string(), "committed".to_string()],
                current_value: "empty".to_string(),
            });
            variables.push(StateVariable {
                name: format!("node_{}_term", i),
                domain: (0..=100).map(|n| n.to_string()).collect(),
                current_value: "0".to_string(),
            });
        }

        let actions = vec![
            TlaAction {
                name: "AppendEntries".to_string(),
                preconditions: vec!["leader_valid".to_string()],
                postconditions: vec!["log_replicated".to_string()],
                enabled: true,
            },
            TlaAction {
                name: "RequestVote".to_string(),
                preconditions: vec!["term_valid".to_string()],
                postconditions: vec!["vote_granted".to_string()],
                enabled: true,
            },
            TlaAction {
                name: "CommitEntry".to_string(),
                preconditions: vec![format!("quorum >= {}", quorum)],
                postconditions: vec!["entry_committed".to_string()],
                enabled: true,
            },
        ];

        Self {
            spec: TlaSpec {
                spec_name: "RaftConsensusSafety".to_string(),
                variables,
                initial_predicate: r"ALL n IN Nodes : node_n_log = empty /\ node_n_term = 0"
                    .to_string(),
                actions,
                next_state_relation: "? a ? Actions : ENABLED(a) ? a".to_string(),
                fairness_conditions: vec![
                    "WF_Vars(AppendEntries)".to_string(),
                    "WF_Vars(CommitEntry)".to_string(),
                ],
            },
            node_count,
            quorum_size: quorum,
        }
    }

    pub fn generate_tla_module(&self) -> String {
        let mut module = format!("---- MODULE {} ----\n", self.spec.spec_name);
        module.push_str("EXTENDS Naturals, FiniteSets, Sequences\n\n");

        module.push_str(&format!(
            "CONSTANTS Nodes, QuorumSize = {}\n",
            self.quorum_size
        ));
        module.push_str("VARIABLES ");
        for (i, var) in self.spec.variables.iter().enumerate() {
            if i > 0 {
                module.push_str(", ");
            }
            module.push_str(&var.name);
        }
        module.push_str("\n\n");

        module.push_str("vars == <<");
        for (i, var) in self.spec.variables.iter().enumerate() {
            if i > 0 {
                module.push_str(", ");
            }
            module.push_str(&var.name);
        }
        module.push_str(">>\n\n");

        module.push_str(&format!("Init == {}\n\n", self.spec.initial_predicate));

        for action in &self.spec.actions {
            module.push_str(&format!("{} ==\n", action.name));
            module.push_str(&format!("  ? {}\n", action.preconditions.join("\n  ? ")));
            module.push_str(&format!("  ? {}\n\n", action.postconditions.join("\n  ? ")));
        }

        module.push_str(&format!("Next == {}\n\n", self.spec.next_state_relation));
        module.push_str("Spec == Init ? ?[Next]_vars\n");
        module.push_str(&format!(
            "  ? {}\n",
            self.spec.fairness_conditions.join("\n  ? ")
        ));

        module.push_str("\n====\n");
        module
    }
}

/// Specification for Actor Model liveness
#[derive(Debug, Clone)]
pub struct ActorLivenessSpec {
    pub spec: TlaSpec,
    pub actor_count: usize,
}

impl ActorLivenessSpec {
    pub fn new(actor_count: usize) -> Self {
        let mut variables = Vec::new();

        for i in 0..actor_count {
            variables.push(StateVariable {
                name: format!("actor_{}_state", i),
                domain: vec![
                    "idle".to_string(),
                    "processing".to_string(),
                    "failed".to_string(),
                    "restarted".to_string(),
                ],
                current_value: "idle".to_string(),
            });
        }

        let actions = vec![
            TlaAction {
                name: "ProcessMessage".to_string(),
                preconditions: vec!["actor_alive".to_string(), "mailbox_non_empty".to_string()],
                postconditions: vec!["message_processed".to_string()],
                enabled: true,
            },
            TlaAction {
                name: "HandleFailure".to_string(),
                preconditions: vec!["actor_failed".to_string()],
                postconditions: vec!["actor_restarted".to_string()],
                enabled: true,
            },
            TlaAction {
                name: "SupervisorRestart".to_string(),
                preconditions: vec!["restart_count < max_restarts".to_string()],
                postconditions: vec!["actor_restarted".to_string()],
                enabled: true,
            },
        ];

        Self {
            spec: TlaSpec {
                spec_name: "ActorLiveness".to_string(),
                variables,
                initial_predicate: "ALL a IN Actors : actor_a_state = idle".to_string(),
                actions,
                next_state_relation: "? a ? Actions : ENABLED(a) ? a".to_string(),
                fairness_conditions: vec![
                    "WF_Vars(ProcessMessage)".to_string(),
                    "SF_Vars(HandleFailure)".to_string(),
                ],
            },
            actor_count,
        }
    }

    pub fn generate_tla_module(&self) -> String {
        let mut module = format!("---- MODULE {} ----\n", self.spec.spec_name);
        module.push_str("EXTENDS Naturals, FiniteSets\n\n");

        module.push_str(&format!("CONSTANTS Actors, MaxRestarts\n"));
        module.push_str("VARIABLES ");
        for (i, var) in self.spec.variables.iter().enumerate() {
            if i > 0 {
                module.push_str(", ");
            }
            module.push_str(&var.name);
        }
        module.push_str(", restart_count\n\n");

        module.push_str("vars == <<");
        for var in &self.spec.variables {
            module.push_str(&format!("{}, ", var.name));
        }
        module.push_str("restart_count>>\n\n");

        module.push_str(&format!("Init == {}\n\n", self.spec.initial_predicate));
        module.push_str("  ? restart_count = 0\n\n");

        for action in &self.spec.actions {
            module.push_str(&format!("{} ==\n", action.name));
            module.push_str(&format!("  ? {}\n", action.preconditions.join("\n  ? ")));
            module.push_str(&format!("  ? {}\n\n", action.postconditions.join("\n  ? ")));
        }

        module.push_str(&format!("Next == {}\n\n", self.spec.next_state_relation));
        module.push_str("Spec == Init ? ?[Next]_vars\n");
        module.push_str(&format!(
            "  ? {}\n",
            self.spec.fairness_conditions.join("\n  ? ")
        ));

        module.push_str(
            "\nLiveness == ? a ? Actors : actor_a_state = failed ~> actor_a_state = restarted\n",
        );
        module.push_str("\n====\n");
        module
    }
}
