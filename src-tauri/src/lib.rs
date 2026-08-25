use tauri_plugin_autostart::MacosLauncher;

pub mod audio;
pub mod domain;
pub mod error;
pub mod ports;
pub mod settings;
pub mod store;
pub mod stt;
pub mod text;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
        .run(tauri::generate_context!())
        .expect("error while running Vox");
}
