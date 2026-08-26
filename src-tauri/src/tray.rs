use tauri::{App, Emitter, Manager, Wry, menu::MenuBuilder, tray::TrayIconBuilder};

use crate::{app::VoxRuntime, domain::DictationEvent};

pub fn create_tray(app: &App<Wry>) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .text("dictate", "Start dictating")
        .text("dashboard", "Open Vox")
        .separator()
        .text("settings", "Settings")
        .text("quit", "Quit Vox")
        .build()?;
    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))?;

    TrayIconBuilder::<Wry>::with_id("vox")
        .icon(icon)
        .icon_as_template(true)
        .tooltip("Vox — local voice-to-text")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "dictate" => {
                let runtime = app.state::<VoxRuntime>().inner().clone();
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = runtime.dispatch(&app, DictationEvent::Toggle).await {
                        tracing::error!(%error, "tray dictation action failed");
                    }
                });
            }
            "dashboard" => show_dashboard(app),
            "settings" => {
                show_dashboard(app);
                let _ = app.emit("vox://navigate", "settings");
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}

fn show_dashboard(app: &tauri::AppHandle<Wry>) {
    if let Some(window) = app.get_webview_window("dashboard") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
