use tauri::{App, Manager, PhysicalPosition, WindowEvent, Wry};

#[cfg(target_os = "macos")]
use tauri_nspanel::{CollectionBehavior, PanelLevel, StyleMask, WebviewWindowExt, tauri_panel};

#[cfg(target_os = "macos")]
tauri_panel! {
    panel!(VoxPillPanel {
        config: {
            can_become_key_window: false,
            can_become_main_window: false,
            is_floating_panel: true
        }
    })
}

pub fn configure_windows(app: &mut App<Wry>) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    if let Some(dashboard) = app.get_webview_window("dashboard") {
        let dashboard_for_close = dashboard.clone();
        dashboard.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = dashboard_for_close.hide();
            }
        });
    }

    if let Some(pill) = app.get_webview_window("pill") {
        position_pill(&pill)?;
        #[cfg(target_os = "macos")]
        {
            let panel = pill.to_panel::<VoxPillPanel<Wry>>()?;
            panel.set_level(PanelLevel::Floating.value());
            panel.set_style_mask(StyleMask::empty().nonactivating_panel().borderless().into());
            panel.set_collection_behavior(
                CollectionBehavior::new()
                    .full_screen_auxiliary()
                    .can_join_all_spaces()
                    .into(),
            );
        }
    }
    Ok(())
}

fn position_pill(pill: &tauri::WebviewWindow<Wry>) -> tauri::Result<()> {
    let Some(monitor) = pill.current_monitor()?.or(pill.primary_monitor()?) else {
        return Ok(());
    };
    let monitor_position = monitor.position();
    let monitor_size = monitor.size();
    let pill_size = pill.outer_size()?;
    let bottom_margin = (72.0 * monitor.scale_factor()).round() as i32;
    let x = monitor_position.x + (monitor_size.width as i32 - pill_size.width as i32) / 2;
    let y =
        monitor_position.y + monitor_size.height as i32 - pill_size.height as i32 - bottom_margin;
    pill.set_position(PhysicalPosition::new(x, y))
}
