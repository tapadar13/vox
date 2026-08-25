use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log::{RotationStrategy, log::LevelFilter};

pub mod app;
pub mod audio;
pub mod commands;
pub mod delivery;
pub mod domain;
pub mod error;
pub mod hotkeys;
pub mod models_mgr;
pub mod platform;
pub mod ports;
pub mod settings;
pub mod store;
pub mod stt;
pub mod text;
pub mod tray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(LevelFilter::Info)
                .max_file_size(2_000_000)
                .rotation_strategy(RotationStrategy::KeepSome(3))
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = tauri::Manager::get_webview_window(app, "dashboard") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .macos_launcher(MacosLauncher::LaunchAgent)
                .build(),
        )
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_nspanel::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::toggle_dictation,
            commands::cancel_dictation,
            commands::retry_dictation,
            commands::dismiss_dictation,
            commands::get_settings,
            commands::update_settings,
            commands::get_history,
            commands::delete_history,
            commands::clear_history,
            commands::get_stats,
            commands::list_models,
            commands::download_model,
            commands::select_model,
            commands::copy_text,
            commands::show_dashboard,
            commands::hide_dashboard,
        ])
        .setup(|app| {
            platform::configure_windows(app)?;
            let data_directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_directory)?;
            let settings_store = settings::JsonSettings::new(data_directory.join("settings.json"));
            let settings_path_exists = settings_store.path().exists();
            let loaded = tauri::async_runtime::block_on(settings_store.load());
            let settings = match loaded {
                Ok(settings) => settings,
                Err(error) => {
                    tracing::error!(%error, "settings were invalid; using safe defaults");
                    settings::Settings::default()
                }
            };
            if !settings_path_exists {
                tauri::async_runtime::block_on(settings_store.save(&settings))
                    .map_err(setup_error)?;
            }

            let runtime = app::VoxRuntime::new(&data_directory, settings).map_err(setup_error)?;
            app.manage(runtime.clone());
            tauri::async_runtime::block_on(runtime.initialize(app.handle()))
                .map_err(setup_error)?;
            tray::create_tray(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Vox");
}

fn setup_error(error: error::VoxError) -> tauri::Error {
    let error: Box<dyn std::error::Error> = Box::new(error);
    tauri::Error::Setup(error.into())
}
