use std::sync::Mutex;

use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::error::{VoxError, VoxResult};

#[derive(Default)]
pub struct HotkeyManager {
    main_shortcut: Mutex<Option<String>>,
    escape_registered: Mutex<bool>,
}

impl HotkeyManager {
    pub fn register_main<R, F>(
        &self,
        app: &AppHandle<R>,
        shortcut: &str,
        handler: F,
    ) -> VoxResult<()>
    where
        R: Runtime,
        F: Fn(AppHandle<R>) + Send + Sync + 'static,
    {
        validate(shortcut)?;
        let mut active = self
            .main_shortcut
            .lock()
            .map_err(|_| VoxError::Other("hotkey state lock was poisoned".to_owned()))?;
        if let Some(previous) = active.take() {
            app.global_shortcut()
                .unregister(previous.as_str())
                .map_err(shortcut_error)?;
        }
        app.global_shortcut()
            .on_shortcut(shortcut, move |app, _, event| {
                if event.state == ShortcutState::Pressed {
                    handler(app.clone());
                }
            })
            .map_err(shortcut_error)?;
        *active = Some(shortcut.to_owned());
        Ok(())
    }

    pub fn register_escape<R, F>(&self, app: &AppHandle<R>, handler: F) -> VoxResult<()>
    where
        R: Runtime,
        F: Fn(AppHandle<R>) + Send + Sync + 'static,
    {
        let mut registered = self
            .escape_registered
            .lock()
            .map_err(|_| VoxError::Other("hotkey state lock was poisoned".to_owned()))?;
        if *registered {
            return Ok(());
        }
        app.global_shortcut()
            .on_shortcut("Escape", move |app, _, event| {
                if event.state == ShortcutState::Pressed {
                    handler(app.clone());
                }
            })
            .map_err(shortcut_error)?;
        *registered = true;
        Ok(())
    }

    pub fn unregister_escape<R: Runtime>(&self, app: &AppHandle<R>) -> VoxResult<()> {
        let mut registered = self
            .escape_registered
            .lock()
            .map_err(|_| VoxError::Other("hotkey state lock was poisoned".to_owned()))?;
        if *registered {
            app.global_shortcut()
                .unregister("Escape")
                .map_err(shortcut_error)?;
            *registered = false;
        }
        Ok(())
    }
}

pub fn validate(shortcut: &str) -> VoxResult<()> {
    shortcut
        .parse::<Shortcut>()
        .map(|_| ())
        .map_err(|error| VoxError::Settings(format!("invalid hotkey: {error}")))
}

fn shortcut_error(error: tauri_plugin_global_shortcut::Error) -> VoxError {
    VoxError::Other(format!("global shortcut error: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_user_configurable_shortcuts() {
        assert!(validate("CommandOrControl+Shift+Space").is_ok());
        assert!(validate("not-a-shortcut").is_err());
    }
}
