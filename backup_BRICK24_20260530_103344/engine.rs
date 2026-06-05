use std::sync::{Arc, Mutex};
use crate::errors::SovereignResult;
use crate::types::Command;
use crate::raft_engine::RaftEngine;
use crate::state_machine::StateMachine;
use crate::federation_router::FederationRouter;
use crate::wal::WALEngine;
use crate::config::SovereignConfig;

pub struct SovereignOrchestrator {
    config: SovereignConfig,
    raft: Arc<Mutex<RaftEngine>>,
    state_machine: Arc<Mutex<StateMachine>>,
    router: Arc<Mutex<FederationRouter>>,
    wal: Arc<Mutex<WALEngine>>,
}

impl SovereignOrchestrator {
    pub fn new(node_id: u64) -> Self {
        let config = SovereignConfig::default_for_node(node_id);
        let peers: Vec<u64> = config.peers.iter().map(|p| p.id).collect();
        
        Self {
            raft: Arc::new(Mutex::new(RaftEngine::new(node_id, peers))),
            state_machine: Arc::new(Mutex::new(StateMachine::new())),
            router: Arc::new(Mutex::new(FederationRouter::new(node_id))),
            wal: Arc::new(Mutex::new(WALEngine::new())),
            config,
        }
    }

    pub fn submit_command(&self, command: Command) -> SovereignResult<u64> {
        let term = self.get_current_term();
        let index = self.append_to_log(term, command)?;
        self.replicate_to_followers(term, index)?;
        self.commit_up_to(index)?;
        Ok(index)
    }

    fn append_to_log(&self, term: u64, command: Command) -> SovereignResult<u64> {
        let mut raft = self.raft.lock().unwrap();
        // Serialize as flat JSON for state_machine.rs compatibility
        let data = match &command {
            Command::Set { key, value } => {
                format!(r#"{{"op":"set","key":"{}","value":"{}"}}"#, key, value)
            }
            Command::Delete { key } => {
                format!(r#"{{"op":"delete","key":"{}"}}"#, key)
            }
            Command::Noop => {
                r#"{"op":"noop"}"#.to_string()
            }
        };
        let result = raft.append(term, &data);
        self.wal.lock().unwrap().persist(&format!("{:?}", result));
        Ok(result.index)
    }

    fn replicate_to_followers(&self, _term: u64, _index: u64) -> SovereignResult<()> {
        Ok(())
    }

    fn commit_up_to(&self, index: u64) -> SovereignResult<()> {
        let mut raft = self.raft.lock().unwrap();
        let entries = raft.commit(index);
        let mut sm = self.state_machine.lock().unwrap();
        for (_idx, data) in entries {
            let _result = sm.apply(&data);
        }
        Ok(())
    }

    pub fn get_current_term(&self) -> u64 {
        self.raft.lock().unwrap().get_current_term()
    }

    pub fn get_commit_index(&self) -> u64 {
        self.raft.lock().unwrap().get_commit_index()
    }

    pub fn get_value(&self, key: &str) -> Option<String> {
        self.state_machine.lock().unwrap().get(key).cloned()
    }

    pub fn set_value(&self, key: String, value: String) -> SovereignResult<()> {
        let index = self.submit_command(Command::Set { key, value })?;
        // BRICK-28.1: Auto-commit for single-node test mode
        self.commit_up_to(index)?;
        Ok(())
    }

    pub fn engine_version(&self) -> String {
        "SovereignCore-v6.2.0-BRICK28-Rust".to_string()
    }
}
