use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, get_lang_str,
};

use crate::{
    audio::TRANSCRIPTION_SAMPLE_RATE,
    error::{VoxError, VoxResult},
    ports::{AudioClip, EngineCaps, EngineId, LanguageHint, SttEngine, Transcript},
};

#[derive(Default)]
pub struct WhisperEngine {
    context: RwLock<Option<Arc<WhisperContext>>>,
}

impl WhisperEngine {
    pub fn new() -> Self {
        Self::default()
    }

    fn context(&self) -> VoxResult<Arc<WhisperContext>> {
        self.context
            .read()
            .map_err(|_| VoxError::Stt("Whisper model lock was poisoned".to_owned()))?
            .clone()
            .ok_or_else(|| VoxError::Stt("Whisper model is not loaded".to_owned()))
    }
}

#[async_trait]
impl SttEngine for WhisperEngine {
    fn id(&self) -> EngineId {
        EngineId::new("whisper-turbo")
    }

    fn capabilities(&self) -> EngineCaps {
        EngineCaps {
            languages: [
                "auto", "en", "hi", "es", "it", "fr", "ar", "de", "pt", "ja", "zh", "ko", "ru",
                "nl", "tr", "pl", "sv", "id", "uk", "el",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            multilingual: true,
            streaming: false,
            model_size_bytes: 617_000_000,
        }
    }

    async fn load(&self, model_path: &Path) -> VoxResult<()> {
        if !model_path.is_file() {
            return Err(VoxError::Model(format!(
                "Whisper model was not found at {}",
                model_path.display()
            )));
        }
        let model_path = model_path.to_path_buf();
        let context = tokio::task::spawn_blocking(move || {
            let mut parameters = WhisperContextParameters::default();
            parameters.use_gpu(true).flash_attn(true);
            WhisperContext::new_with_params(&model_path, parameters)
                .map(Arc::new)
                .map_err(|error| VoxError::Stt(error.to_string()))
        })
        .await
        .map_err(|error| VoxError::Stt(error.to_string()))??;

        *self
            .context
            .write()
            .map_err(|_| VoxError::Stt("Whisper model lock was poisoned".to_owned()))? =
            Some(context);
        Ok(())
    }

    async fn transcribe(&self, audio: AudioClip, language: LanguageHint) -> VoxResult<Transcript> {
        if audio.sample_rate != TRANSCRIPTION_SAMPLE_RATE {
            return Err(VoxError::Stt(format!(
                "Whisper requires {TRANSCRIPTION_SAMPLE_RATE} Hz audio"
            )));
        }
        if audio.samples.is_empty() {
            return Err(VoxError::Stt("recording contained no audio".to_owned()));
        }

        let context = self.context()?;
        tokio::task::spawn_blocking(move || transcribe_blocking(context, audio, language))
            .await
            .map_err(|error| VoxError::Stt(error.to_string()))?
    }
}

fn transcribe_blocking(
    context: Arc<WhisperContext>,
    audio: AudioClip,
    language: LanguageHint,
) -> VoxResult<Transcript> {
    let mut state = context
        .create_state()
        .map_err(|error| VoxError::Stt(error.to_string()))?;
    let mut parameters = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    let thread_count = std::thread::available_parallelism()
        .map(|count| count.get().min(8) as i32)
        .unwrap_or(4);
    parameters.set_n_threads(thread_count);
    parameters.set_translate(false);
    parameters.set_no_context(true);
    parameters.set_print_progress(false);
    parameters.set_print_realtime(false);
    parameters.set_print_special(false);
    parameters.set_print_timestamps(false);
    parameters.set_suppress_blank(true);
    parameters.set_suppress_nst(true);

    let pinned_language = match &language {
        LanguageHint::Auto => None,
        LanguageHint::Pinned(language) => Some(language.as_str()),
    };
    parameters.set_language(pinned_language);
    parameters.set_detect_language(pinned_language.is_none());
    state
        .full(parameters, &audio.samples)
        .map_err(|error| VoxError::Stt(error.to_string()))?;

    let text = state
        .as_iter()
        .map(|segment| {
            segment
                .to_str_lossy()
                .map(|text| text.into_owned())
                .map_err(|error| VoxError::Stt(error.to_string()))
        })
        .collect::<VoxResult<Vec<_>>>()?
        .join("");
    let detected = pinned_language.map(str::to_owned).unwrap_or_else(|| {
        get_lang_str(state.full_lang_id_from_state())
            .unwrap_or("und")
            .to_owned()
    });

    Ok(Transcript {
        text,
        language: detected,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_model_returns_actionable_error() {
        let engine = WhisperEngine::new();
        let error = engine
            .load(Path::new("/definitely/missing/vox-model.bin"))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("not found"));
    }

    #[test]
    fn reports_multilingual_capabilities() {
        let capabilities = WhisperEngine::new().capabilities();
        assert!(capabilities.multilingual);
        assert!(capabilities.languages.contains(&"hi".to_owned()));
        assert!(capabilities.languages.contains(&"ar".to_owned()));
    }
}
