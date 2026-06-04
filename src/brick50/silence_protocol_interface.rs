//! BRICK-50 Pillar 3: Absolute Silence Protocol Interface
//! Zero-fear deterministic gate logic
//! SEV-650:3 — Grounded, emotionless evaluation

#[derive(Clone, Debug, PartialEq)]
pub enum EmotionalState {
    Neutral,
    FearDetected,
    Excitement,
    DriftDetected,
}

pub struct SilenceGate {
    evaluations: u64,
    emotional_violations: u64,
    hallucination_events: u64,
    current_state: EmotionalState,
}

impl SilenceGate {
    pub fn new() -> Self {
        Self {
            evaluations: 0,
            emotional_violations: 0,
            hallucination_events: 0,
            current_state: EmotionalState::Neutral,
        }
    }

    pub fn evaluate(&mut self, input: &str) -> Result<String, String> {
        self.evaluations += 1;

        let emotional_keywords = [
            "fear",
            "panic",
            "urgent",
            "emergency",
            "crisis",
            "desperate",
        ];
        let has_emotion = emotional_keywords
            .iter()
            .any(|&kw| input.to_lowercase().contains(kw));

        if has_emotion {
            self.emotional_violations += 1;
            self.current_state = EmotionalState::FearDetected;
            return Ok(self.neutralize(input));
        }

        let hallucination_markers = ["maybe", "perhaps", "i think", "probably", "guess"];
        let has_hallucination = hallucination_markers
            .iter()
            .any(|&mk| input.to_lowercase().contains(mk));

        if has_hallucination {
            self.hallucination_events += 1;
            self.current_state = EmotionalState::DriftDetected;
            return Err("HALLUCINATION_DETECTED: Uncertain output rejected".to_string());
        }

        self.current_state = EmotionalState::Neutral;
        Ok(input.to_string())
    }

    fn neutralize(&self, input: &str) -> String {
        input
            .to_lowercase()
            .replace("fear", "calm")
            .replace("panic", "steady")
            .replace("urgent", "ordered")
            .replace("emergency", "routine")
            .replace("crisis", "state")
            .replace("desperate", "composed")
    }

    pub fn stats(&self) -> (u64, u64, u64, &EmotionalState) {
        (
            self.evaluations,
            self.emotional_violations,
            self.hallucination_events,
            &self.current_state,
        )
    }
}
