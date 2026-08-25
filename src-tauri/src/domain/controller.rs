use crate::error::{VoxError, VoxResult};

use super::{AppState, DictationEvent, DictationPhase, Effect};

pub const CANCEL_COUNTDOWN_MS: u64 = 3_000;
pub const SUCCESS_VISIBLE_MS: u64 = 900;

#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    pub state: AppState,
    pub effects: Vec<Effect>,
}

#[derive(Debug, Default)]
pub struct DictationController {
    state: AppState,
}

impl DictationController {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn set_model_ready(&mut self, ready: bool) {
        self.state.model_ready = ready;
    }

    pub fn dispatch(&mut self, event: DictationEvent) -> VoxResult<Transition> {
        let effects = match (self.state.phase, event) {
            (DictationPhase::Idle, DictationEvent::Toggle) => {
                self.state.phase = DictationPhase::Recording;
                self.state.elapsed_ms = 0;
                self.state.audio_level = 0.0;
                self.state.message = None;
                self.state.delivery_mode = None;
                vec![
                    Effect::ShowPill,
                    Effect::RegisterEscape,
                    Effect::StartCapture,
                ]
            }
            (DictationPhase::Recording, DictationEvent::Toggle)
            | (DictationPhase::Recording, DictationEvent::RecordingTimedOut) => {
                self.begin_transcription(false)
            }
            (DictationPhase::Recording, DictationEvent::CancelRequested) => {
                self.state.phase = DictationPhase::CancelPending;
                self.state.cancel_remaining_ms = Some(CANCEL_COUNTDOWN_MS);
                vec![Effect::ScheduleCancel {
                    after_ms: CANCEL_COUNTDOWN_MS,
                }]
            }
            (DictationPhase::CancelPending, DictationEvent::CancelRequested) => {
                self.state.phase = DictationPhase::Recording;
                self.state.cancel_remaining_ms = None;
                vec![Effect::CancelScheduledCancel]
            }
            (DictationPhase::CancelPending, DictationEvent::Toggle) => {
                self.begin_transcription(true)
            }
            (DictationPhase::CancelPending, DictationEvent::CancelExpired) => {
                self.state = AppState {
                    active_engine: self.state.active_engine.clone(),
                    model_ready: self.state.model_ready,
                    ..AppState::default()
                };
                vec![
                    Effect::UnregisterEscape,
                    Effect::DiscardCapture,
                    Effect::HidePill,
                ]
            }
            (DictationPhase::CancelPending, DictationEvent::CancelTick { remaining_ms }) => {
                self.state.cancel_remaining_ms = Some(remaining_ms.min(CANCEL_COUNTDOWN_MS));
                vec![]
            }
            (phase, DictationEvent::AudioLevel(level))
                if matches!(
                    phase,
                    DictationPhase::Recording | DictationPhase::CancelPending
                ) =>
            {
                self.state.audio_level = level.clamp(0.0, 1.0);
                vec![]
            }
            (phase, DictationEvent::Elapsed { elapsed_ms })
                if matches!(
                    phase,
                    DictationPhase::Recording | DictationPhase::CancelPending
                ) =>
            {
                self.state.elapsed_ms = elapsed_ms;
                vec![]
            }
            (
                DictationPhase::Transcribing,
                DictationEvent::TranscriptionReady {
                    raw_text,
                    text,
                    language,
                    duration_ms,
                    engine_id,
                },
            ) => {
                self.state.phase = DictationPhase::Delivering;
                self.state.last_transcript = Some(text.clone());
                vec![Effect::DeliverTranscript {
                    raw_text,
                    text,
                    language,
                    duration_ms,
                    engine_id,
                }]
            }
            (DictationPhase::Transcribing, DictationEvent::TranscriptionFailed { message }) => {
                self.state.phase = DictationPhase::Error;
                self.state.message = Some(message);
                vec![]
            }
            (phase, DictationEvent::OperationFailed { message })
                if phase != DictationPhase::Idle =>
            {
                self.state.phase = DictationPhase::Error;
                self.state.message = Some(message);
                self.state.audio_level = 0.0;
                self.state.cancel_remaining_ms = None;
                vec![
                    Effect::CancelScheduledCancel,
                    Effect::UnregisterEscape,
                    Effect::DiscardCapture,
                ]
            }
            (DictationPhase::Delivering, DictationEvent::DeliveryFinished { mode }) => {
                self.state.phase = DictationPhase::Success;
                self.state.delivery_mode = Some(mode);
                self.state.message = match mode {
                    super::DeliveryMode::Pasted => Some("Pasted".to_owned()),
                    super::DeliveryMode::Clipboard => Some("Copied — press ⌘V to paste".to_owned()),
                    super::DeliveryMode::Failed => {
                        Some("Saved to history — delivery failed".to_owned())
                    }
                };
                vec![Effect::ScheduleReset {
                    after_ms: SUCCESS_VISIBLE_MS,
                }]
            }
            (DictationPhase::Error, DictationEvent::Retry) => {
                self.state.phase = DictationPhase::Transcribing;
                self.state.message = None;
                vec![Effect::RetryTranscription]
            }
            (DictationPhase::Success | DictationPhase::Error, DictationEvent::Reset) => {
                self.state = AppState {
                    active_engine: self.state.active_engine.clone(),
                    model_ready: self.state.model_ready,
                    ..AppState::default()
                };
                vec![Effect::DiscardCapture, Effect::HidePill]
            }
            (phase, event) => {
                return Err(VoxError::InvalidTransition(format!(
                    "{event:?} is not valid while {phase:?}"
                )));
            }
        };

        Ok(Transition {
            state: self.state.clone(),
            effects,
        })
    }

    fn begin_transcription(&mut self, cancel_timer: bool) -> Vec<Effect> {
        self.state.phase = DictationPhase::Transcribing;
        self.state.cancel_remaining_ms = None;
        self.state.audio_level = 0.0;

        let mut effects = Vec::with_capacity(3);
        if cancel_timer {
            effects.push(Effect::CancelScheduledCancel);
        }
        effects.push(Effect::UnregisterEscape);
        effects.push(Effect::StopAndTranscribe);
        effects
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DeliveryMode;

    fn controller() -> DictationController {
        DictationController::default()
    }

    #[test]
    fn toggle_starts_capture_and_registers_escape() {
        let transition = controller().dispatch(DictationEvent::Toggle).unwrap();
        assert_eq!(transition.state.phase, DictationPhase::Recording);
        assert_eq!(
            transition.effects,
            vec![
                Effect::ShowPill,
                Effect::RegisterEscape,
                Effect::StartCapture
            ]
        );
    }

    #[test]
    fn cancel_keeps_capture_running_and_second_escape_resumes() {
        let mut controller = controller();
        controller.dispatch(DictationEvent::Toggle).unwrap();

        let pending = controller
            .dispatch(DictationEvent::CancelRequested)
            .unwrap();
        assert!(pending.state.is_capturing());
        assert!(!pending.effects.contains(&Effect::DiscardCapture));

        let resumed = controller
            .dispatch(DictationEvent::CancelRequested)
            .unwrap();
        assert_eq!(resumed.state.phase, DictationPhase::Recording);
        assert_eq!(resumed.effects, vec![Effect::CancelScheduledCancel]);
    }

    #[test]
    fn cancel_expiry_discards_without_transcribing() {
        let mut controller = controller();
        controller.dispatch(DictationEvent::Toggle).unwrap();
        controller
            .dispatch(DictationEvent::CancelRequested)
            .unwrap();

        let cancelled = controller.dispatch(DictationEvent::CancelExpired).unwrap();
        assert_eq!(cancelled.state.phase, DictationPhase::Idle);
        assert!(cancelled.effects.contains(&Effect::DiscardCapture));
        assert!(!cancelled.effects.contains(&Effect::StopAndTranscribe));
    }

    #[test]
    fn hotkey_overrides_pending_cancel() {
        let mut controller = controller();
        controller.dispatch(DictationEvent::Toggle).unwrap();
        controller
            .dispatch(DictationEvent::CancelRequested)
            .unwrap();

        let transition = controller.dispatch(DictationEvent::Toggle).unwrap();
        assert_eq!(transition.state.phase, DictationPhase::Transcribing);
        assert_eq!(
            transition.effects,
            vec![
                Effect::CancelScheduledCancel,
                Effect::UnregisterEscape,
                Effect::StopAndTranscribe
            ]
        );
    }

    #[test]
    fn delivery_falls_through_to_success_feedback() {
        let mut controller = DictationController::new(AppState {
            phase: DictationPhase::Delivering,
            ..AppState::default()
        });
        let transition = controller
            .dispatch(DictationEvent::DeliveryFinished {
                mode: DeliveryMode::Clipboard,
            })
            .unwrap();

        assert_eq!(transition.state.phase, DictationPhase::Success);
        assert_eq!(
            transition.state.message.as_deref(),
            Some("Copied — press ⌘V to paste")
        );
    }

    #[test]
    fn impossible_transition_is_an_error() {
        let error = controller()
            .dispatch(DictationEvent::CancelRequested)
            .unwrap_err();
        assert!(matches!(error, VoxError::InvalidTransition(_)));
    }

    #[test]
    fn adapter_failures_enter_a_recoverable_error_state() {
        let mut controller = controller();
        controller.dispatch(DictationEvent::Toggle).unwrap();
        let transition = controller
            .dispatch(DictationEvent::OperationFailed {
                message: "Microphone permission denied".to_owned(),
            })
            .unwrap();
        assert_eq!(transition.state.phase, DictationPhase::Error);
        assert_eq!(
            transition.state.message.as_deref(),
            Some("Microphone permission denied")
        );
        assert!(transition.effects.contains(&Effect::DiscardCapture));
    }
}
