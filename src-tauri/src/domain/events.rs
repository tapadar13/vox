use serde::{Deserialize, Serialize};

use super::DeliveryMode;

#[derive(Debug, Clone, PartialEq)]
pub enum DictationEvent {
    Toggle,
    CancelRequested,
    CancelTick { remaining_ms: u64 },
    CancelExpired,
    RecordingTimedOut,
    AudioLevel(f32),
    Elapsed { elapsed_ms: u64 },
    TranscriptionReady {
        raw_text: String,
        text: String,
        language: String,
        duration_ms: u64,
        engine_id: String,
    },
    TranscriptionFailed { message: String },
    DeliveryFinished { mode: DeliveryMode },
    Retry,
    Reset,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Effect {
    ShowPill,
    HidePill,
    StartCapture,
    StopAndTranscribe,
    DiscardCapture,
    RegisterEscape,
    UnregisterEscape,
    ScheduleCancel { after_ms: u64 },
    CancelScheduledCancel,
    DeliverTranscript {
        raw_text: String,
        text: String,
        language: String,
        duration_ms: u64,
        engine_id: String,
    },
    RetryTranscription,
    ScheduleReset { after_ms: u64 },
}
