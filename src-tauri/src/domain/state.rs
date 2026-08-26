use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DictationPhase {
    #[default]
    Idle,
    Recording,
    CancelPending,
    Transcribing,
    Delivering,
    Success,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeliveryMode {
    Pasted,
    Clipboard,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub phase: DictationPhase,
    pub elapsed_ms: u64,
    pub cancel_remaining_ms: Option<u64>,
    pub audio_level: f32,
    pub partial_transcript: Option<String>,
    pub stable_words: usize,
    pub message: Option<String>,
    pub last_transcript: Option<String>,
    pub delivery_mode: Option<DeliveryMode>,
    pub active_engine: String,
    pub model_ready: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            phase: DictationPhase::Idle,
            elapsed_ms: 0,
            cancel_remaining_ms: None,
            audio_level: 0.0,
            partial_transcript: None,
            stable_words: 0,
            message: None,
            last_transcript: None,
            delivery_mode: None,
            active_engine: "whisper-turbo".to_owned(),
            model_ready: false,
        }
    }
}

impl AppState {
    pub fn is_capturing(&self) -> bool {
        matches!(
            self.phase,
            DictationPhase::Recording | DictationPhase::CancelPending
        )
    }

    pub fn pill_visible(&self) -> bool {
        self.phase != DictationPhase::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_states_are_the_only_capturing_states() {
        let mut state = AppState {
            phase: DictationPhase::Recording,
            ..AppState::default()
        };
        assert!(state.is_capturing());

        state.phase = DictationPhase::CancelPending;
        assert!(state.is_capturing());

        state.phase = DictationPhase::Transcribing;
        assert!(!state.is_capturing());
    }
}
