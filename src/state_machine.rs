use std::collections::HashMap;

pub struct StateMachine {
    store: HashMap<String, String>,
}

#[derive(serde::Deserialize)]
struct Command {
    op: String,
    key: Option<String>,
    value: Option<String>,
}

impl StateMachine {
    pub fn new() -> Self {
        StateMachine {
            store: HashMap::new(),
        }
    }

    pub fn apply(&mut self, command: &str) -> String {
        if let Ok(cmd) = serde_json::from_str::<Command>(command) {
            match cmd.op.as_str() {
                "set" => {
                    if let (Some(k), Some(v)) = (cmd.key, cmd.value) {
                        self.store.insert(k.clone(), v);
                        format!("{{\"status\":\"ok\",\"key\":\"{}\"}}", k)
                    } else {
                        "{\"status\":\"error\",\"reason\":\"missing key or value\"}".to_string()
                    }
                }
                "get" => {
                    if let Some(k) = cmd.key {
                        match self.store.get(&k) {
                            Some(v) => format!(
                                "{{\"status\":\"ok\",\"key\":\"{}\",\"value\":\"{}\"}}",
                                k, v
                            ),
                            None => format!("{{\"status\":\"not_found\",\"key\":\"{}\"}}", k),
                        }
                    } else {
                        "{\"status\":\"error\",\"reason\":\"missing key\"}".to_string()
                    }
                }
                "delete" => {
                    if let Some(k) = cmd.key {
                        self.store.remove(&k);
                        format!("{{\"status\":\"ok\",\"key\":\"{}\"}}", k)
                    } else {
                        "{\"status\":\"error\",\"reason\":\"missing key\"}".to_string()
                    }
                }
                _ => "{\"status\":\"error\",\"reason\":\"unknown op\"}".to_string(),
            }
        } else {
            "{\"status\":\"error\",\"reason\":\"invalid command\"}".to_string()
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.store.get(key)
    }

    pub fn set(&mut self, key: String, value: String) {
        self.store.insert(key, value);
    }
}
