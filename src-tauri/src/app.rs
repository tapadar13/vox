use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use chrono::{Local, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tokio::sync::{Mutex, RwLock};

use crate::{
    audio::CpalAudioInput,
    delivery::MacTextDelivery,
    domain::{AppState, DeliveryMode, DictationController, DictationEvent, Effect, StatsSnapshot},
    error::{VoxError, VoxResult},
    hotkeys::HotkeyManager,
    models_mgr::{ModelDownloadProgress, ModelManager, ModelSpec},
    ports::{
        AudioClip, AudioInput, DeliveryOutcome, HistoryQuery, StatsStore, SttEngine, TextDelivery,
        TextRefiner, TranscriptRecord, TranscriptStore,
    },
    settings::{JsonSettings, Settings},
    store::SqliteStore,
    stt::EngineRegistry,
    text::{FormatterConfig, RuleTextRefiner},
};

#[cfg(feature = "whisper")]
use crate::stt::WhisperEngine;

const STATE_EVENT: &str = "vox://state";
const HISTORY_EVENT: &str = "vox://history-changed";
const MODEL_PROGRESS_EVENT: &str = "vox://model-progress";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedModel {
    #[serde(flatten)]
    pub spec: ModelSpec,
    pub installed: bool,
    pub active: bool,
}

#[derive(Clone)]
pub struct VoxRuntime {
    inner: Arc<RuntimeInner>,
}

struct RuntimeInner {
    controller: Mutex<DictationController>,
    audio: Arc<dyn AudioInput>,
    engines: Arc<EngineRegistry>,
    refiner: RwLock<Arc<dyn TextRefiner>>,
    store: SqliteStore,
    delivery: Arc<dyn TextDelivery>,
    settings: RwLock<Settings>,
    settings_store: JsonSettings,
    models: ModelManager,
    hotkeys: HotkeyManager,
    last_audio: Mutex<Option<AudioClip>>,
    latency_started: Mutex<Option<Instant>>,
    cancel_epoch: AtomicU64,
    recording_epoch: AtomicU64,
    reset_epoch: AtomicU64,
}

impl VoxRuntime {
    pub fn new(data_directory: &Path, settings: Settings) -> VoxResult<Self> {
        #[allow(unused_mut)]
        let mut engines = EngineRegistry::default();
        #[cfg(feature = "whisper")]
        engines.register(Arc::new(WhisperEngine::new()))?;

        let store = SqliteStore::open(&data_directory.join("vox.db"))?;
        let settings_store = JsonSettings::new(data_directory.join("settings.json"));
        let models = ModelManager::new(data_directory.join("models"));
        let refiner = RuleTextRefiner::new(FormatterConfig {
            trim_filler_words: settings.trim_filler_words,
        });

        Ok(Self {
            inner: Arc::new(RuntimeInner {
                controller: Mutex::new(DictationController::default()),
                audio: Arc::new(CpalAudioInput::new()),
                engines: Arc::new(engines),
                refiner: RwLock::new(Arc::new(refiner)),
                store,
                delivery: Arc::new(MacTextDelivery),
                settings: RwLock::new(settings),
                settings_store,
                models,
                hotkeys: HotkeyManager::default(),
                last_audio: Mutex::new(None),
                latency_started: Mutex::new(None),
                cancel_epoch: AtomicU64::new(0),
                recording_epoch: AtomicU64::new(0),
                reset_epoch: AtomicU64::new(0),
            }),
        })
    }

    pub async fn initialize(&self, app: &AppHandle) -> VoxResult<()> {
        let settings = self.settings().await;
        self.register_main_hotkey(app, &settings.hotkey)?;
        self.apply_autostart(app, settings.launch_at_login)?;
        self.prewarm_selected_model(app.clone());
        Ok(())
    }

    pub async fn state(&self) -> AppState {
        self.inner.controller.lock().await.state().clone()
    }

    pub async fn settings(&self) -> Settings {
        self.inner.settings.read().await.clone()
    }

    pub async fn update_settings(&self, app: &AppHandle, settings: Settings) -> VoxResult<()> {
        settings.validate()?;
        crate::hotkeys::validate(&settings.hotkey)?;
        let previous = self.settings().await;

        if settings.hotkey != previous.hotkey {
            self.register_main_hotkey(app, &settings.hotkey)?;
        }
        if settings.launch_at_login != previous.launch_at_login
            && let Err(error) = self.apply_autostart(app, settings.launch_at_login)
        {
            if settings.hotkey != previous.hotkey {
                let _ = self.register_main_hotkey(app, &previous.hotkey);
            }
            return Err(error);
        }
        if let Err(error) = self.inner.settings_store.save(&settings).await {
            if settings.hotkey != previous.hotkey {
                let _ = self.register_main_hotkey(app, &previous.hotkey);
            }
            if settings.launch_at_login != previous.launch_at_login {
                let _ = self.apply_autostart(app, previous.launch_at_login);
            }
            return Err(error);
        }
        if settings.trim_filler_words != previous.trim_filler_words {
            *self.inner.refiner.write().await = Arc::new(RuleTextRefiner::new(FormatterConfig {
                trim_filler_words: settings.trim_filler_words,
            }));
        }
        *self.inner.settings.write().await = settings;
        Ok(())
    }

    pub async fn dispatch(&self, app: &AppHandle, event: DictationEvent) -> VoxResult<AppState> {
        let transition = {
            let mut controller = self.inner.controller.lock().await;
            controller.dispatch(event)?
        };
        app.emit(STATE_EVENT, &transition.state)
            .map_err(|error| VoxError::Other(error.to_string()))?;
        for effect in transition.effects {
            self.execute_effect(app, effect);
        }
        Ok(transition.state)
    }

    pub async fn history(&self, query: HistoryQuery) -> VoxResult<Vec<TranscriptRecord>> {
        self.inner.store.history(query).await
    }

    pub async fn delete_history(&self, app: &AppHandle, id: i64) -> VoxResult<()> {
        self.inner.store.delete(id).await?;
        let _ = app.emit(HISTORY_EVENT, ());
        Ok(())
    }

    pub async fn clear_history(&self, app: &AppHandle) -> VoxResult<()> {
        self.inner.store.clear().await?;
        let _ = app.emit(HISTORY_EVENT, ());
        Ok(())
    }

    pub async fn stats(&self) -> VoxResult<StatsSnapshot> {
        let settings = self.settings().await;
        self.inner
            .store
            .stats(settings.typing_wpm, Local::now().date_naive())
            .await
    }

    pub async fn models(&self) -> VoxResult<Vec<ManagedModel>> {
        let active = self.settings().await.model_id;
        let mut models = Vec::new();
        for spec in ModelManager::registry() {
            models.push(ManagedModel {
                installed: self.inner.models.installed(&spec.id).await?,
                active: spec.id == active,
                spec,
            });
        }
        Ok(models)
    }

    pub async fn download_model(&self, app: &AppHandle, id: &str) -> VoxResult<PathBuf> {
        let progress_app = app.clone();
        let path = self
            .inner
            .models
            .download(
                id,
                Arc::new(move |progress: ModelDownloadProgress| {
                    let _ = progress_app.emit(MODEL_PROGRESS_EVENT, progress);
                }),
            )
            .await?;

        if self.settings().await.model_id == id {
            self.load_model(app, &path).await?;
        }
        Ok(path)
    }

    pub async fn load_selected_model(&self, app: &AppHandle) -> VoxResult<()> {
        let settings = self.settings().await;
        let path = self.inner.models.model_path(&settings.model_id)?;
        self.load_model(app, &path).await
    }

    fn execute_effect(&self, app: &AppHandle, effect: Effect) {
        match effect {
            Effect::ShowPill => {
                if let Some(window) = app.get_webview_window("pill") {
                    let _ = window.show();
                }
            }
            Effect::HidePill => {
                if let Some(window) = app.get_webview_window("pill") {
                    let _ = window.hide();
                }
            }
            Effect::StartCapture => {
                let epoch = self.inner.recording_epoch.fetch_add(1, Ordering::SeqCst) + 1;
                let runtime = self.clone();
                let capture_runtime = self.clone();
                let event_app = app.clone();
                let callback_app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let max_duration = Duration::from_secs(u64::from(
                        runtime.settings().await.max_recording_seconds,
                    ));
                    let callback = Arc::new(move |level| {
                        capture_runtime
                            .queue_event(callback_app.clone(), DictationEvent::AudioLevel(level));
                    });
                    if let Err(error) = runtime.inner.audio.start(max_duration, callback).await {
                        runtime.queue_event(
                            event_app,
                            DictationEvent::OperationFailed {
                                message: error.to_string(),
                            },
                        );
                        return;
                    }
                    runtime.run_recording_clock(event_app, epoch, max_duration);
                });
            }
            Effect::StopAndTranscribe => {
                self.inner.recording_epoch.fetch_add(1, Ordering::SeqCst);
                let runtime = self.clone();
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    runtime.transcribe_capture(app, false).await;
                });
            }
            Effect::RetryTranscription => {
                let runtime = self.clone();
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    runtime.transcribe_capture(app, true).await;
                });
            }
            Effect::DiscardCapture => {
                self.inner.recording_epoch.fetch_add(1, Ordering::SeqCst);
                let runtime = self.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = runtime.inner.audio.discard().await;
                    *runtime.inner.last_audio.lock().await = None;
                });
            }
            Effect::RegisterEscape => {
                let runtime = self.clone();
                let result = self.inner.hotkeys.register_escape(app, move |app| {
                    runtime.queue_event(app, DictationEvent::CancelRequested);
                });
                if let Err(error) = result {
                    tracing::error!(%error, "could not register the recording Escape shortcut");
                }
            }
            Effect::UnregisterEscape => {
                if let Err(error) = self.inner.hotkeys.unregister_escape(app) {
                    tracing::warn!(%error, "could not unregister the recording Escape shortcut");
                }
            }
            Effect::ScheduleCancel { after_ms } => {
                let epoch = self.inner.cancel_epoch.fetch_add(1, Ordering::SeqCst) + 1;
                let runtime = self.clone();
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(after_ms)).await;
                    if runtime.inner.cancel_epoch.load(Ordering::SeqCst) == epoch {
                        runtime.queue_event(app, DictationEvent::CancelExpired);
                    }
                });
            }
            Effect::CancelScheduledCancel => {
                self.inner.cancel_epoch.fetch_add(1, Ordering::SeqCst);
            }
            Effect::DeliverTranscript {
                raw_text,
                text,
                language,
                duration_ms,
                engine_id,
            } => {
                let runtime = self.clone();
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    runtime
                        .deliver_and_store(app, raw_text, text, language, duration_ms, engine_id)
                        .await;
                });
            }
            Effect::ScheduleReset { after_ms } => {
                let epoch = self.inner.reset_epoch.fetch_add(1, Ordering::SeqCst) + 1;
                let runtime = self.clone();
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(after_ms)).await;
                    if runtime.inner.reset_epoch.load(Ordering::SeqCst) == epoch {
                        runtime.queue_event(app, DictationEvent::Reset);
                    }
                });
            }
        }
    }

    fn queue_event(&self, app: AppHandle, event: DictationEvent) {
        let runtime = self.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(error) = runtime.dispatch(&app, event).await {
                tracing::debug!(%error, "ignored stale or invalid runtime event");
            }
        });
    }

    fn run_recording_clock(&self, app: AppHandle, epoch: u64, max_duration: Duration) {
        let runtime = self.clone();
        tauri::async_runtime::spawn(async move {
            let started = Instant::now();
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                if runtime.inner.recording_epoch.load(Ordering::SeqCst) != epoch {
                    break;
                }
                let elapsed = started.elapsed();
                if elapsed >= max_duration {
                    runtime.queue_event(app, DictationEvent::RecordingTimedOut);
                    break;
                }
                runtime.queue_event(
                    app.clone(),
                    DictationEvent::Elapsed {
                        elapsed_ms: elapsed.as_millis() as u64,
                    },
                );
            }
        });
    }

    async fn transcribe_capture(&self, app: AppHandle, retry: bool) {
        let audio = if retry {
            self.inner.last_audio.lock().await.clone()
        } else {
            match self.inner.audio.stop().await {
                Ok(audio) => {
                    *self.inner.last_audio.lock().await = Some(audio.clone());
                    Some(audio)
                }
                Err(error) => {
                    self.queue_event(
                        app,
                        DictationEvent::OperationFailed {
                            message: error.to_string(),
                        },
                    );
                    return;
                }
            }
        };
        let Some(audio) = audio else {
            self.queue_event(
                app,
                DictationEvent::TranscriptionFailed {
                    message: "The previous recording is no longer available".to_owned(),
                },
            );
            return;
        };
        *self.inner.latency_started.lock().await = Some(Instant::now());
        let settings = self.settings().await;
        let engine = match self.inner.engines.get(&settings.engine_id) {
            Ok(engine) => engine,
            Err(error) => {
                self.queue_event(
                    app,
                    DictationEvent::TranscriptionFailed {
                        message: error.to_string(),
                    },
                );
                return;
            }
        };
        let duration_ms = audio.duration_ms;
        let transcript = match engine.transcribe(audio, settings.language).await {
            Ok(transcript) => transcript,
            Err(error) => {
                self.queue_event(
                    app,
                    DictationEvent::TranscriptionFailed {
                        message: error.to_string(),
                    },
                );
                return;
            }
        };
        let refiner = self.inner.refiner.read().await.clone();
        let formatted = match refiner.refine(&transcript.text).await {
            Ok(text) => text,
            Err(error) => {
                self.queue_event(
                    app,
                    DictationEvent::TranscriptionFailed {
                        message: error.to_string(),
                    },
                );
                return;
            }
        };
        self.queue_event(
            app,
            DictationEvent::TranscriptionReady {
                raw_text: transcript.text,
                text: formatted,
                language: transcript.language,
                duration_ms,
                engine_id: engine.id().0,
            },
        );
    }

    #[allow(clippy::too_many_arguments)]
    async fn deliver_and_store(
        &self,
        app: AppHandle,
        raw_text: String,
        text: String,
        language: String,
        duration_ms: u64,
        engine_id: String,
    ) {
        let settings = self.settings().await;
        let outcome = match self
            .inner
            .delivery
            .deliver(&text, settings.auto_paste)
            .await
        {
            Ok(outcome) => outcome,
            Err(error) => {
                tracing::error!(%error, "text delivery failed; preserving transcript in history");
                DeliveryOutcome::Failed
            }
        };
        let latency_ms = self
            .inner
            .latency_started
            .lock()
            .await
            .take()
            .map(|started| started.elapsed().as_millis() as u64)
            .unwrap_or_default();
        let record = TranscriptRecord {
            id: None,
            created_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            word_count: text.split_whitespace().count() as u64,
            text,
            raw_text,
            language,
            duration_ms,
            latency_ms,
            engine_id,
            delivered: outcome == DeliveryOutcome::Pasted,
        };
        if let Err(error) = self.inner.store.insert(record).await {
            tracing::error!(%error, "could not save transcript history");
        } else {
            let _ = app.emit(HISTORY_EVENT, ());
        }
        let mode = match outcome {
            DeliveryOutcome::Pasted => DeliveryMode::Pasted,
            DeliveryOutcome::Clipboard => DeliveryMode::Clipboard,
            DeliveryOutcome::Failed => DeliveryMode::Failed,
        };
        self.queue_event(app, DictationEvent::DeliveryFinished { mode });
    }

    fn register_main_hotkey(&self, app: &AppHandle, shortcut: &str) -> VoxResult<()> {
        let runtime = self.clone();
        self.inner.hotkeys.register_main(app, shortcut, move |app| {
            runtime.queue_event(app, DictationEvent::Toggle);
        })
    }

    fn apply_autostart(&self, app: &AppHandle, enabled: bool) -> VoxResult<()> {
        let launcher = app.autolaunch();
        let result = if enabled {
            launcher.enable()
        } else {
            launcher.disable()
        };
        result.map_err(|error| VoxError::Settings(error.to_string()))
    }

    fn prewarm_selected_model(&self, app: AppHandle) {
        let runtime = self.clone();
        tauri::async_runtime::spawn(async move {
            let settings = runtime.settings().await;
            match runtime.inner.models.installed(&settings.model_id).await {
                Ok(true) => {
                    if let Err(error) = runtime.load_selected_model(&app).await {
                        tracing::error!(%error, "could not prewarm the selected model");
                    }
                }
                Ok(false) => {
                    tracing::info!(model = %settings.model_id, "selected model is not installed")
                }
                Err(error) => tracing::warn!(%error, "could not inspect the selected model"),
            }
        });
    }

    async fn load_model(&self, app: &AppHandle, path: &Path) -> VoxResult<()> {
        let engine_id = self.settings().await.engine_id;
        let engine: Arc<dyn SttEngine> = self.inner.engines.get(&engine_id)?;
        engine.load(path).await?;
        let state = {
            let mut controller = self.inner.controller.lock().await;
            controller.set_model_ready(true);
            controller.state().clone()
        };
        let _ = app.emit(STATE_EVENT, state);
        Ok(())
    }
}
