use serde::Serialize;
use thiserror::Error;

pub type VoxResult<T> = Result<T, VoxError>;

#[derive(Debug, Error)]
pub enum VoxError {
    #[error("settings error: {0}")]
    Settings(String),
    #[error("audio error: {0}")]
    Audio(String),
    #[error("transcription error: {0}")]
    Stt(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("delivery error: {0}")]
    Delivery(String),
    #[error("model error: {0}")]
    Model(String),
    #[error("invalid transition: {0}")]
    InvalidTransition(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: &'static str,
    pub message: String,
}

impl From<VoxError> for CommandError {
    fn from(error: VoxError) -> Self {
        let code = match &error {
            VoxError::Settings(_) => "settings",
            VoxError::Audio(_) => "audio",
            VoxError::Stt(_) => "transcription",
            VoxError::Storage(_) => "storage",
            VoxError::Delivery(_) => "delivery",
            VoxError::Model(_) => "model",
            VoxError::InvalidTransition(_) => "invalid-transition",
            VoxError::Io(_) => "io",
            VoxError::Other(_) => "unknown",
        };

        Self {
            code,
            message: error.to_string(),
        }
    }
}
