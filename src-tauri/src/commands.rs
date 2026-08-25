use arboard::Clipboard;
use tauri::{AppHandle, Manager, State};

use crate::{
    app::{ManagedModel, VoxRuntime},
    domain::{AppState, DictationEvent, StatsSnapshot},
    error::CommandError,
    ports::{HistoryQuery, TranscriptRecord},
    settings::Settings,
};

#[tauri::command]
pub async fn get_state(runtime: State<'_, VoxRuntime>) -> Result<AppState, CommandError> {
    Ok(runtime.state().await)
}

#[tauri::command]
pub async fn toggle_dictation(
    app: AppHandle,
    runtime: State<'_, VoxRuntime>,
) -> Result<AppState, CommandError> {
    runtime
        .dispatch(&app, DictationEvent::Toggle)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn cancel_dictation(
    app: AppHandle,
    runtime: State<'_, VoxRuntime>,
) -> Result<AppState, CommandError> {
    runtime
        .dispatch(&app, DictationEvent::CancelRequested)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn retry_dictation(
    app: AppHandle,
    runtime: State<'_, VoxRuntime>,
) -> Result<AppState, CommandError> {
    runtime
        .dispatch(&app, DictationEvent::Retry)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn dismiss_dictation(
    app: AppHandle,
    runtime: State<'_, VoxRuntime>,
) -> Result<AppState, CommandError> {
    runtime
        .dispatch(&app, DictationEvent::Reset)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_settings(runtime: State<'_, VoxRuntime>) -> Result<Settings, CommandError> {
    Ok(runtime.settings().await)
}

#[tauri::command]
pub async fn update_settings(
    app: AppHandle,
    runtime: State<'_, VoxRuntime>,
    settings: Settings,
) -> Result<(), CommandError> {
    runtime
        .update_settings(&app, settings)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_history(
    runtime: State<'_, VoxRuntime>,
    search: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<TranscriptRecord>, CommandError> {
    runtime
        .history(HistoryQuery {
            search,
            limit: limit.unwrap_or(50),
            offset: offset.unwrap_or(0),
        })
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn delete_history(
    app: AppHandle,
    runtime: State<'_, VoxRuntime>,
    id: i64,
) -> Result<(), CommandError> {
    runtime.delete_history(&app, id).await.map_err(Into::into)
}

#[tauri::command]
pub async fn clear_history(
    app: AppHandle,
    runtime: State<'_, VoxRuntime>,
) -> Result<(), CommandError> {
    runtime.clear_history(&app).await.map_err(Into::into)
}

#[tauri::command]
pub async fn get_stats(runtime: State<'_, VoxRuntime>) -> Result<StatsSnapshot, CommandError> {
    runtime.stats().await.map_err(Into::into)
}

#[tauri::command]
pub async fn list_models(
    runtime: State<'_, VoxRuntime>,
) -> Result<Vec<ManagedModel>, CommandError> {
    runtime.models().await.map_err(Into::into)
}

#[tauri::command]
pub async fn download_model(
    app: AppHandle,
    runtime: State<'_, VoxRuntime>,
    id: String,
) -> Result<String, CommandError> {
    runtime
        .download_model(&app, &id)
        .await
        .map(|path| path.display().to_string())
        .map_err(Into::into)
}

#[tauri::command]
pub async fn select_model(
    app: AppHandle,
    runtime: State<'_, VoxRuntime>,
    id: String,
) -> Result<(), CommandError> {
    let mut settings = runtime.settings().await;
    settings.model_id = id;
    runtime.update_settings(&app, settings).await?;
    if runtime
        .models()
        .await?
        .iter()
        .any(|model| model.active && model.installed)
    {
        runtime.load_selected_model(&app).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn copy_text(text: String) -> Result<(), CommandError> {
    tokio::task::spawn_blocking(move || {
        let mut clipboard = Clipboard::new()
            .map_err(|error| crate::error::VoxError::Delivery(error.to_string()))?;
        clipboard
            .set_text(text)
            .map_err(|error| crate::error::VoxError::Delivery(error.to_string()))
    })
    .await
    .map_err(|error| crate::error::VoxError::Delivery(error.to_string()))??;
    Ok(())
}

#[tauri::command]
pub fn show_dashboard(app: AppHandle) -> Result<(), CommandError> {
    if let Some(window) = app.get_webview_window("dashboard") {
        window
            .show()
            .and_then(|_| window.set_focus())
            .map_err(|error| crate::error::VoxError::Other(error.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn hide_dashboard(app: AppHandle) -> Result<(), CommandError> {
    if let Some(window) = app.get_webview_window("dashboard") {
        window
            .hide()
            .map_err(|error| crate::error::VoxError::Other(error.to_string()))?;
    }
    Ok(())
}
