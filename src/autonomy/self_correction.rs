use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackType {
    Positive,
    Negative,
    Neutral,
    Success,
    Failure,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub feedback_id: String,
    pub source: String,
    pub signal_type: FeedbackType,
    pub value: f64,
    pub timestamp_nanos: u64,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    pub correction_id: String,
    pub target: String,
    pub action_type: String,
    pub parameter: String,
    pub old_value: f64,
    pub new_value: f64,
    pub confidence: f64,
    pub applied_at_nanos: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecovery {
    pub recovery_id: String,
    pub error_type: String,
    pub error_message: String,
    pub strategy: String,
    pub success: bool,
    pub recovered_at_nanos: u64,
}

pub struct FeedbackLoop {
    pub feedback_history: VecDeque<Feedback>,
    pub corrections: Vec<Correction>,
    pub recoveries: Vec<ErrorRecovery>,
    pub learning_rate: f64,
    pub threshold_positive: f64,
    pub threshold_negative: f64,
    pub max_history: usize,
    pub adaptation_velocity: f64,
}

impl FeedbackLoop {
    pub fn new(
        learning_rate: f64,
        pos_threshold: f64,
        neg_threshold: f64,
        max_history: usize,
    ) -> Self {
        Self {
            feedback_history: VecDeque::with_capacity(max_history),
            corrections: Vec::new(),
            recoveries: Vec::new(),
            learning_rate: learning_rate.clamp(0.0, 1.0),
            threshold_positive: pos_threshold.clamp(0.0, 1.0),
            threshold_negative: neg_threshold.clamp(-1.0, 0.0),
            max_history,
            adaptation_velocity: 0.0,
        }
    }

    pub fn ingest_feedback(&mut self, feedback: Feedback) -> Option<Correction> {
        if self.feedback_history.len() >= self.max_history {
            self.feedback_history.pop_front();
        }
        self.feedback_history.push_back(feedback.clone());

        let is_negative = matches!(
            feedback.signal_type,
            FeedbackType::Negative | FeedbackType::Error | FeedbackType::Failure
        );

        if is_negative && feedback.value < self.threshold_negative {
            let correction = Correction {
                correction_id: format!("corr_{}", feedback.timestamp_nanos),
                target: feedback.source.clone(),
                action_type: "adjust".to_string(),
                parameter: "weight".to_string(),
                old_value: 0.0,
                new_value: feedback.value.abs() * self.learning_rate,
                confidence: feedback.value.abs().clamp(0.0, 1.0),
                applied_at_nanos: feedback.timestamp_nanos,
            };
            self.corrections.push(correction.clone());
            Some(correction)
        } else {
            None
        }
    }

    pub fn get_average_feedback(&self, window: usize) -> f64 {
        let recent: Vec<&Feedback> = self.feedback_history.iter().rev().take(window).collect();
        if recent.is_empty() {
            return 0.0;
        }
        recent.iter().map(|f| f.value).sum::<f64>() / recent.len() as f64
    }

    pub fn stats(&self) -> FeedbackStats {
        FeedbackStats {
            total_feedback: self.feedback_history.len() as u64,
            total_corrections: self.corrections.len() as u64,
            total_recoveries: self.recoveries.len() as u64,
            avg_feedback: self.get_average_feedback(100),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackStats {
    pub total_feedback: u64,
    pub total_corrections: u64,
    pub total_recoveries: u64,
    pub avg_feedback: f64,
}
