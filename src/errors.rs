use std::fmt;

#[derive(Debug, Clone)]
pub enum SovereignError {
    Raft(String),
    Wal(String),
    StateMachine(String),
    Router(String),
    Serialization(String),
    Io(String),
    NotLeader(String),
    QuorumNotAchieved,
    TermMismatch { expected: u64, actual: u64 },
    LogMismatch { index: u64 },
}

pub type SovereignResult<T> = Result<T, SovereignError>;

// Implement Display manually to give us .to_string() functionality for PyRuntimeError
impl fmt::Display for SovereignError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SovereignError::Raft(s) => write!(f, "Raft error: {}", s),
            SovereignError::Wal(s) => write!(f, "WAL error: {}", s),
            SovereignError::StateMachine(s) => write!(f, "State machine error: {}", s),
            SovereignError::Router(s) => write!(f, "Federation router error: {}", s),
            SovereignError::Serialization(s) => write!(f, "Serialization error: {}", s),
            SovereignError::Io(s) => write!(f, "IO error: {}", s),
            SovereignError::NotLeader(s) => write!(f, "Not leader: current leader is {}", s),
            SovereignError::QuorumNotAchieved => write!(f, "Quorum not achieved"),
            SovereignError::TermMismatch { expected, actual } => {
                write!(f, "Term mismatch: expected {}, got {}", expected, actual)
            }
            SovereignError::LogMismatch { index } => write!(f, "Log mismatch at index {}", index),
        }
    }
}

impl std::error::Error for SovereignError {}

impl From<serde_json::Error> for SovereignError {
    fn from(e: serde_json::Error) -> Self {
        SovereignError::Serialization(e.to_string())
    }
}
