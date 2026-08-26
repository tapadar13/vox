use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use sherpa_rs::transducer::{TransducerConfig, TransducerRecognizer};

use crate::{
    audio::TRANSCRIPTION_SAMPLE_RATE,
    error::{VoxError, VoxResult},
    ports::{AudioClip, EngineCaps, EngineId, LanguageHint, SttEngine, Transcript},
};

#[derive(Clone, Default)]
pub struct ParakeetEngine {
    recognizer: Arc<Mutex<Option<TransducerRecognizer>>>,
}

impl ParakeetEngine {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SttEngine for ParakeetEngine {
    fn id(&self) -> EngineId {
        EngineId::new("parakeet-v2")
    }

    fn capabilities(&self) -> EngineCaps {
        EngineCaps {
            languages: vec!["en".to_owned()],
            multilingual: false,
            streaming: true,
            model_size_bytes: 670_000_000,
        }
    }

    async fn load(&self, model_path: &Path) -> VoxResult<()> {
        let files = ParakeetFiles::discover(model_path)?;
        let recognizer = tokio::task::spawn_blocking(move || {
            TransducerRecognizer::new(TransducerConfig {
                encoder: path_string(files.encoder),
                decoder: path_string(files.decoder),
                joiner: path_string(files.joiner),
                tokens: path_string(files.tokens),
                model_type: "nemo_transducer".to_owned(),
                sample_rate: TRANSCRIPTION_SAMPLE_RATE as i32,
                feature_dim: 80,
                decoding_method: "greedy_search".to_owned(),
                num_threads: available_threads(),
                ..TransducerConfig::default()
            })
            .map_err(|error| VoxError::Stt(error.to_string()))
        })
        .await
        .map_err(|error| VoxError::Stt(error.to_string()))??;
        *self
            .recognizer
            .lock()
            .map_err(|_| VoxError::Stt("Parakeet model lock was poisoned".to_owned()))? =
            Some(recognizer);
        Ok(())
    }

    async fn transcribe(&self, audio: AudioClip, language: LanguageHint) -> VoxResult<Transcript> {
        if audio.sample_rate != TRANSCRIPTION_SAMPLE_RATE {
            return Err(VoxError::Stt(format!(
                "Parakeet requires {TRANSCRIPTION_SAMPLE_RATE} Hz audio"
            )));
        }
        if audio.samples.is_empty() {
            return Err(VoxError::Stt("recording contained no audio".to_owned()));
        }
        if let LanguageHint::Pinned(language) = &language
            && language != "en"
        {
            return Err(VoxError::Stt(
                "Parakeet supports English dictation only".to_owned(),
            ));
        }

        let recognizer = Arc::clone(&self.recognizer);
        let text = tokio::task::spawn_blocking(move || -> VoxResult<String> {
            let mut slot = recognizer
                .lock()
                .map_err(|_| VoxError::Stt("Parakeet model lock was poisoned".to_owned()))?;
            let recognizer = slot
                .as_mut()
                .ok_or_else(|| VoxError::Stt("Parakeet model is not loaded".to_owned()))?;
            Ok(recognizer.transcribe(audio.sample_rate, &audio.samples))
        })
        .await
        .map_err(|error| VoxError::Stt(error.to_string()))??;

        Ok(Transcript {
            text,
            language: "en".to_owned(),
            segments: vec![],
        })
    }
}

struct ParakeetFiles {
    encoder: PathBuf,
    decoder: PathBuf,
    joiner: PathBuf,
    tokens: PathBuf,
}

impl ParakeetFiles {
    fn discover(path: &Path) -> VoxResult<Self> {
        if !path.is_dir() {
            return Err(VoxError::Model(format!(
                "Parakeet model directory was not found at {}",
                path.display()
            )));
        }
        Ok(Self {
            encoder: find_model_file(path, &["encoder.int8.onnx", "encoder.onnx"])?,
            decoder: find_model_file(path, &["decoder.int8.onnx", "decoder.onnx"])?,
            joiner: find_model_file(path, &["joiner.int8.onnx", "joiner.onnx"])?,
            tokens: find_model_file(path, &["tokens.txt"])?,
        })
    }
}

fn find_model_file(directory: &Path, candidates: &[&str]) -> VoxResult<PathBuf> {
    candidates
        .iter()
        .map(|name| directory.join(name))
        .find(|path| path.is_file())
        .ok_or_else(|| {
            VoxError::Model(format!(
                "Parakeet model is missing {}",
                candidates.join(" or ")
            ))
        })
}

fn path_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

fn available_threads() -> i32 {
    std::thread::available_parallelism()
        .map(|count| count.get().min(8) as i32)
        .unwrap_or(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_quantized_parakeet_bundle() {
        let directory = tempfile::tempdir().unwrap();
        for name in [
            "encoder.int8.onnx",
            "decoder.int8.onnx",
            "joiner.int8.onnx",
            "tokens.txt",
        ] {
            std::fs::write(directory.path().join(name), []).unwrap();
        }
        let files = ParakeetFiles::discover(directory.path()).unwrap();
        assert!(files.encoder.ends_with("encoder.int8.onnx"));
        assert!(files.tokens.ends_with("tokens.txt"));
    }

    #[test]
    fn reports_english_streaming_capabilities() {
        let capabilities = ParakeetEngine::new().capabilities();
        assert!(!capabilities.multilingual);
        assert!(capabilities.streaming);
        assert_eq!(capabilities.languages, vec!["en"]);
    }
}
