use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    error::{VoxError, VoxResult},
    ports::{LanguageHint, SettingsStore},
};

pub const SETTINGS_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub schema_version: u32,
    pub hotkey: String,
    pub language: LanguageHint,
    pub engine_id: String,
    pub model_id: String,
    pub auto_paste: bool,
    pub launch_at_login: bool,
    pub trim_filler_words: bool,
    pub typing_wpm: u32,
    pub max_recording_seconds: u32,
    pub onboarding_complete: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            hotkey: "Alt+Space".to_owned(),
            language: LanguageHint::Auto,
            engine_id: "whisper-turbo".to_owned(),
            model_id: "whisper-large-v3-turbo-q5_0".to_owned(),
            auto_paste: true,
            launch_at_login: false,
            trim_filler_words: false,
            typing_wpm: 40,
            max_recording_seconds: 300,
            onboarding_complete: false,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> VoxResult<()> {
        if self.schema_version > SETTINGS_SCHEMA_VERSION {
            return Err(VoxError::Settings(format!(
                "settings schema {} is newer than supported schema {SETTINGS_SCHEMA_VERSION}",
                self.schema_version
            )));
        }
        if self.hotkey.trim().is_empty() {
            return Err(VoxError::Settings("hotkey cannot be empty".to_owned()));
        }
        if !(20..=200).contains(&self.typing_wpm) {
            return Err(VoxError::Settings(
                "typing speed must be between 20 and 200 WPM".to_owned(),
            ));
        }
        if !(10..=1_800).contains(&self.max_recording_seconds) {
            return Err(VoxError::Settings(
                "recording limit must be between 10 seconds and 30 minutes".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct JsonSettings {
    path: PathBuf,
}

impl JsonSettings {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub async fn load(&self) -> VoxResult<Settings> {
        let bytes = match fs::read(&self.path).await {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Settings::default());
            }
            Err(error) => return Err(error.into()),
        };
        let settings: Settings = serde_json::from_slice(&bytes)
            .map_err(|error| VoxError::Settings(error.to_string()))?;
        settings.validate()?;
        Ok(settings)
    }

    pub async fn save(&self, settings: &Settings) -> VoxResult<()> {
        settings.validate()?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let bytes = serde_json::to_vec_pretty(settings)
            .map_err(|error| VoxError::Settings(error.to_string()))?;
        let temporary = self.path.with_extension("json.tmp");
        fs::write(&temporary, bytes).await?;
        fs::rename(&temporary, &self.path).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl SettingsStore for JsonSettings {
    async fn load(&self) -> VoxResult<Settings> {
        JsonSettings::load(self).await
    }

    async fn save(&self, settings: &Settings) -> VoxResult<()> {
        JsonSettings::save(self, settings).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid_and_private() {
        let settings = Settings::default();
        assert!(settings.validate().is_ok());
        assert!(settings.auto_paste);
        assert_eq!(settings.language, LanguageHint::Auto);
    }

    #[test]
    fn unsafe_recording_limits_are_rejected() {
        let settings = Settings {
            max_recording_seconds: 3_600,
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
    }

    #[tokio::test]
    async fn settings_round_trip_atomically() {
        let directory = tempfile::tempdir().unwrap();
        let store = JsonSettings::new(directory.path().join("settings.json"));
        let expected = Settings {
            auto_paste: false,
            onboarding_complete: true,
            ..Settings::default()
        };

        store.save(&expected).await.unwrap();
        assert_eq!(store.load().await.unwrap(), expected);
        assert!(!store.path().with_extension("json.tmp").exists());
    }
}
