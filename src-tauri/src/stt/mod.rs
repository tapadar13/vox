pub mod registry;

#[cfg(feature = "whisper")]
pub mod whisper;

pub use registry::EngineRegistry;
#[cfg(feature = "whisper")]
pub use whisper::WhisperEngine;
