use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type ActorId = String;
pub type MessagePayload = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorMessage {
    pub from: ActorId,
    pub to: ActorId,
    pub payload: MessagePayload,
    pub timestamp: u64,
    pub message_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Command,
    Query,
    Event,
    Heartbeat,
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct ActorState {
    pub actor_id: ActorId,
    pub actor_type: String,
    pub mailbox: Vec<ActorMessage>,
    pub processed_count: u64,
    pub failed_count: u64,
    pub is_alive: bool,
    pub metadata: HashMap<String, String>,
}

pub struct ActorSystem {
    pub actors: HashMap<ActorId, Arc<Mutex<ActorState>>>,
    pub message_log: Vec<ActorMessage>,
    pub system_id: String,
}

impl ActorSystem {
    pub fn new(system_id: String) -> Self {
        Self {
            actors: HashMap::new(),
            message_log: Vec::new(),
            system_id,
        }
    }

    pub fn spawn_actor(&mut self, actor_id: ActorId, actor_type: String) -> bool {
        if self.actors.contains_key(&actor_id) {
            return false;
        }
        let state = ActorState {
            actor_id: actor_id.clone(),
            actor_type,
            mailbox: Vec::new(),
            processed_count: 0,
            failed_count: 0,
            is_alive: true,
            metadata: HashMap::new(),
        };
        self.actors.insert(actor_id, Arc::new(Mutex::new(state)));
        true
    }

    pub fn send_message(&mut self, message: ActorMessage) -> bool {
        if let Some(actor_arc) = self.actors.get(&message.to) {
            if let Ok(mut state) = actor_arc.lock() {
                if state.is_alive {
                    state.mailbox.push(message.clone());
                    self.message_log.push(message);
                    return true;
                }
            }
        }
        false
    }

    pub fn process_mailbox(&mut self, actor_id: &ActorId) -> Vec<ActorMessage> {
        if let Some(actor_arc) = self.actors.get(actor_id) {
            if let Ok(mut state) = actor_arc.lock() {
                if !state.is_alive {
                    return Vec::new();
                }
                let messages: Vec<ActorMessage> = state.mailbox.drain(..).collect();
                state.processed_count += messages.len() as u64;
                return messages;
            }
        }
        Vec::new()
    }

    pub fn kill_actor(&mut self, actor_id: &ActorId) -> bool {
        if let Some(actor_arc) = self.actors.get(actor_id) {
            if let Ok(mut state) = actor_arc.lock() {
                state.is_alive = false;
                return true;
            }
        }
        false
    }

    pub fn actor_count(&self) -> usize {
        self.actors.len()
    }

    pub fn alive_count(&self) -> usize {
        self.actors
            .values()
            .filter(|a| a.lock().map(|s| s.is_alive).unwrap_or(false))
            .count()
    }

    pub fn get_actor_state(&self, actor_id: &ActorId) -> Option<ActorState> {
        self.actors
            .get(actor_id)
            .and_then(|a| a.lock().ok().map(|s| s.clone()))
    }
}
