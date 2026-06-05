pub struct WALEngine;

impl WALEngine {
    pub fn new() -> Self {
        WALEngine
    }
    pub fn persist(&self, _entry: &str) -> bool {
        true
    }
}
