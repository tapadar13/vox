use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use futures_util::StreamExt;
use reqwest::{Client, StatusCode, header::RANGE};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};

use crate::error::{VoxError, VoxResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSpec {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub url: String,
    pub speed: String,
    pub accuracy: String,
    pub multilingual: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub fraction: f64,
}

#[derive(Clone)]
pub struct ModelManager {
    directory: PathBuf,
    client: Client,
}

impl ModelManager {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
            client: Client::new(),
        }
    }

    pub fn registry() -> Vec<ModelSpec> {
        vec![
            ModelSpec {
                id: "whisper-large-v3-turbo-q5_0".to_owned(),
                name: "Turbo · Best quality".to_owned(),
                filename: "ggml-large-v3-turbo-q5_0.bin".to_owned(),
                size_bytes: 574_041_195,
                sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2"
                    .to_owned(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin"
                    .to_owned(),
                speed: "Fast on Apple Silicon".to_owned(),
                accuracy: "Best".to_owned(),
                multilingual: true,
            },
            ModelSpec {
                id: "whisper-small-q5_1".to_owned(),
                name: "Small · Balanced".to_owned(),
                filename: "ggml-small-q5_1.bin".to_owned(),
                size_bytes: 190_085_487,
                sha256: "ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb"
                    .to_owned(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small-q5_1.bin"
                    .to_owned(),
                speed: "Very fast".to_owned(),
                accuracy: "Good".to_owned(),
                multilingual: true,
            },
            ModelSpec {
                id: "whisper-base-q5_1".to_owned(),
                name: "Base · Lightest".to_owned(),
                filename: "ggml-base-q5_1.bin".to_owned(),
                size_bytes: 59_707_625,
                sha256: "422f1ae452ade6f30a004d7e5c6a43195e4433bc370bf23fac9cc591f01a8898"
                    .to_owned(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base-q5_1.bin"
                    .to_owned(),
                speed: "Fastest".to_owned(),
                accuracy: "Basic".to_owned(),
                multilingual: true,
            },
        ]
    }

    pub fn spec(&self, id: &str) -> VoxResult<ModelSpec> {
        Self::registry()
            .into_iter()
            .find(|model| model.id == id)
            .ok_or_else(|| VoxError::Model(format!("unknown model '{id}'")))
    }

    pub fn model_path(&self, id: &str) -> VoxResult<PathBuf> {
        Ok(self.directory.join(self.spec(id)?.filename))
    }

    pub async fn installed(&self, id: &str) -> VoxResult<bool> {
        let spec = self.spec(id)?;
        let path = self.directory.join(spec.filename);
        if !path.is_file() {
            return Ok(false);
        }
        verify_checksum(path, spec.sha256).await
    }

    pub async fn download(
        &self,
        id: &str,
        on_progress: Arc<dyn Fn(ModelDownloadProgress) + Send + Sync>,
    ) -> VoxResult<PathBuf> {
        let spec = self.spec(id)?;
        fs::create_dir_all(&self.directory).await?;
        let destination = self.directory.join(&spec.filename);
        if destination.is_file()
            && verify_checksum(destination.clone(), spec.sha256.clone()).await?
        {
            return Ok(destination);
        }

        let partial = destination.with_extension("bin.part");
        let existing = fs::metadata(&partial)
            .await
            .map(|metadata| metadata.len())
            .unwrap_or_default();
        let mut request = self.client.get(&spec.url);
        if existing > 0 {
            request = request.header(RANGE, format!("bytes={existing}-"));
        }
        let response = request
            .send()
            .await
            .map_err(|error| VoxError::Model(error.to_string()))?
            .error_for_status()
            .map_err(|error| VoxError::Model(error.to_string()))?;
        let resumed = existing > 0 && response.status() == StatusCode::PARTIAL_CONTENT;
        let offset = if resumed { existing } else { 0 };
        let total = response
            .content_length()
            .map(|length| length + offset)
            .unwrap_or(spec.size_bytes);
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(resumed)
            .truncate(!resumed)
            .open(&partial)
            .await?;
        let mut downloaded = offset;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|error| VoxError::Model(error.to_string()))?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            on_progress(ModelDownloadProgress {
                downloaded_bytes: downloaded,
                total_bytes: total,
                fraction: if total == 0 {
                    0.0
                } else {
                    (downloaded as f64 / total as f64).clamp(0.0, 1.0)
                },
            });
        }
        file.flush().await?;
        drop(file);

        if !verify_checksum(partial.clone(), spec.sha256).await? {
            let _ = fs::remove_file(&partial).await;
            return Err(VoxError::Model(
                "model checksum did not match the curated registry".to_owned(),
            ));
        }
        fs::rename(&partial, &destination).await?;
        Ok(destination)
    }
}

async fn verify_checksum(path: PathBuf, expected: String) -> VoxResult<bool> {
    tokio::task::spawn_blocking(move || checksum_matches(&path, &expected))
        .await
        .map_err(|error| VoxError::Model(error.to_string()))?
}

fn checksum_matches(path: &Path, expected: &str) -> VoxResult<bool> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()).eq_ignore_ascii_case(expected))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curated_models_have_complete_integrity_metadata() {
        for model in ModelManager::registry() {
            assert_eq!(model.sha256.len(), 64);
            assert!(
                model
                    .sha256
                    .chars()
                    .all(|character| character.is_ascii_hexdigit())
            );
            assert!(model.url.starts_with("https://huggingface.co/"));
            assert!(model.size_bytes > 50_000_000);
        }
    }

    #[test]
    fn model_paths_can_only_come_from_the_registry() {
        let manager = ModelManager::new("/tmp/vox-model-test");
        assert!(manager.model_path("../../secret").is_err());
        assert!(
            manager
                .model_path("whisper-base-q5_1")
                .unwrap()
                .ends_with("ggml-base-q5_1.bin")
        );
    }
}
