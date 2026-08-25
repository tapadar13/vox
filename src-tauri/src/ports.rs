use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{error::VoxResult, settings::Settings};

#[derive(Debug, Clone, PartialEq)]
pub struct AudioClip {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EngineId(pub String);

impl EngineId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineCaps {
    pub languages: Vec<String>,
    pub multilingual: bool,
    pub streaming: bool,
    pub model_size_bytes: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "mode", content = "language")]
pub enum LanguageHint {
    #[default]
    Auto,
    Pinned(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transcript {
    pub text: String,
    pub language: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptRecord {
    pub id: Option<i64>,
    pub created_at: String,
    pub text: String,
    pub raw_text: String,
    pub language: String,
    pub duration_ms: u64,
    pub latency_ms: u64,
    pub word_count: u64,
    pub engine_id: String,
    pub delivered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryQuery {
    pub search: Option<String>,
    pub limit: u32,
    pub offset: u32,
}

impl Default for HistoryQuery {
    fn default() -> Self {
        Self {
            search: None,
            limit: 50,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryOutcome {
    Pasted,
    Clipboard,
    Failed,
}

#[async_trait]
pub trait AudioInput: Send + Sync {
    async fn start(&self, on_level: Arc<dyn Fn(f32) + Send + Sync>) -> VoxResult<()>;
    async fn stop(&self) -> VoxResult<AudioClip>;
    async fn discard(&self) -> VoxResult<()>;
}

#[async_trait]
pub trait SttEngine: Send + Sync {
    fn id(&self) -> EngineId;
    fn capabilities(&self) -> EngineCaps;
    async fn load(&self, model_path: &Path) -> VoxResult<()>;
    async fn transcribe(&self, audio: AudioClip, language: LanguageHint) -> VoxResult<Transcript>;
}

#[async_trait]
pub trait TextRefiner: Send + Sync {
    async fn refine(&self, text: &str) -> VoxResult<String>;
}

#[async_trait]
pub trait TranscriptStore: Send + Sync {
    async fn insert(&self, record: TranscriptRecord) -> VoxResult<i64>;
    async fn history(&self, query: HistoryQuery) -> VoxResult<Vec<TranscriptRecord>>;
    async fn delete(&self, id: i64) -> VoxResult<()>;
    async fn clear(&self) -> VoxResult<()>;
}

#[async_trait]
pub trait TextDelivery: Send + Sync {
    async fn deliver(&self, text: &str, auto_paste: bool) -> VoxResult<DeliveryOutcome>;
}

#[async_trait]
pub trait SettingsStore: Send + Sync {
    async fn load(&self) -> VoxResult<Settings>;
    async fn save(&self, settings: &Settings) -> VoxResult<()>;
}
