use crate::llm_engine::config::{get_model_by_name, get_model_catalog, GGUF_MAGIC};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

/// Model status for LLM models
#[derive(Debug, Clone)]
pub enum LLMModelStatus {
    Available,
    Missing,
    Downloading { progress: u8 },
    Error(String),
    Corrupted { file_size: u64, expected_size: u64 },
    Imported,
}

impl Serialize for LLMModelStatus {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            LLMModelStatus::Available => serializer.serialize_str("available"),
            LLMModelStatus::Missing => serializer.serialize_str("missing"),
            LLMModelStatus::Downloading { .. } => serializer.serialize_str("downloading"),
            LLMModelStatus::Error(_) => serializer.serialize_str("error"),
            LLMModelStatus::Corrupted { .. } => serializer.serialize_str("corrupted"),
            LLMModelStatus::Imported => serializer.serialize_str("imported"),
        }
    }
}

impl<'de> Deserialize<'de> for LLMModelStatus {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "available" => Ok(LLMModelStatus::Available),
            "missing" => Ok(LLMModelStatus::Missing),
            "downloading" => Ok(LLMModelStatus::Downloading { progress: 0 }),
            "error" => Ok(LLMModelStatus::Error(String::new())),
            "corrupted" => Ok(LLMModelStatus::Corrupted { file_size: 0, expected_size: 0 }),
            "imported" => Ok(LLMModelStatus::Imported),
            _ => Err(serde::de::Error::custom(format!("unknown status: {}", s))),
        }
    }
}

/// Download progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMDownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub downloaded_mb: f64,
    pub total_mb: f64,
    pub speed_mbps: f64,
    pub percent: u8,
}

impl LLMDownloadProgress {
    pub fn new(downloaded: u64, total: u64, speed_mbps: f64) -> Self {
        let percent = if total > 0 {
            ((downloaded as f64 / total as f64) * 100.0).min(100.0) as u8
        } else {
            0
        };
        Self {
            downloaded_bytes: downloaded,
            total_bytes: total,
            downloaded_mb: downloaded as f64 / (1024.0 * 1024.0),
            total_mb: total as f64 / (1024.0 * 1024.0),
            speed_mbps,
            percent,
        }
    }
}

/// Information about an LLM model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMModelInfo {
    pub name: String,
    pub display_name: String,
    pub size_mb: u64,
    pub context_length: u32,
    pub description: String,
    pub status: LLMModelStatus,
}

/// Model manager for LLM models
pub struct LLMModelManager {
    models_dir: PathBuf,
    active_downloads: Arc<RwLock<HashSet<String>>>,
}

impl LLMModelManager {
    /// Create a new model manager
    pub fn new(models_dir: PathBuf) -> Self {
        Self {
            models_dir,
            active_downloads: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Discover available models
    pub async fn discover_models(&self) -> Result<Vec<LLMModelInfo>> {
        let mut models = Vec::new();
        let active_downloads = self.active_downloads.read().await;

        let catalog_names: HashSet<String> = get_model_catalog()
            .iter()
            .map(|m| m.name.clone())
            .collect();

        for metadata in get_model_catalog() {
            let model_path = self.models_dir.join(&metadata.name);
            let status = if active_downloads.contains(&metadata.name) {
                LLMModelStatus::Downloading { progress: 0 }
            } else if model_path.exists() {
                match self.validate_model(&model_path).await {
                    Ok(_) => LLMModelStatus::Available,
                    Err(e) => {
                        log::warn!("Model {} validation failed: {}", metadata.name, e);
                        let file_size = std::fs::metadata(&model_path)
                            .map(|m| m.len())
                            .unwrap_or(0);
                        LLMModelStatus::Corrupted {
                            file_size,
                            expected_size: metadata.size_mb * 1024 * 1024,
                        }
                    }
                }
            } else {
                LLMModelStatus::Missing
            };

            models.push(LLMModelInfo {
                name: metadata.name.clone(),
                display_name: metadata.display_name.clone(),
                size_mb: metadata.size_mb,
                context_length: metadata.context_length,
                description: metadata.description.clone(),
                status,
            });
        }

        // Discover imported/custom models (GGUF files not in the catalog)
        if self.models_dir.exists() {
            let mut entries = tokio::fs::read_dir(&self.models_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let filename = match path.file_name().and_then(|n| n.to_str()) {
                    Some(name) => name.to_string(),
                    None => continue,
                };

                // Skip temp files and catalog models
                if filename.ends_with(".tmp") || catalog_names.contains(&filename) {
                    continue;
                }

                // Only include .gguf files
                if !filename.to_lowercase().ends_with(".gguf") {
                    continue;
                }

                // Validate the file
                match self.validate_model(&path).await {
                    Ok(_) => {
                        let file_metadata = std::fs::metadata(&path)?;
                        let size_mb = file_metadata.len() / (1024 * 1024);

                        models.push(LLMModelInfo {
                            name: filename.clone(),
                            display_name: format!("Custom: {}", filename),
                            size_mb,
                            context_length: 4096,
                            description: "Imported custom GGUF model".to_string(),
                            status: LLMModelStatus::Imported,
                        });
                    }
                    Err(e) => {
                        log::warn!("Skipping invalid file in models dir: {} ({})", filename, e);
                    }
                }
            }
        }

        Ok(models)
    }

    /// Validate a model file by checking GGUF magic number
    pub async fn validate_model(&self, model_path: &Path) -> Result<()> {
        if !model_path.exists() {
            return Err(anyhow!("Model file not found"));
        }

        let metadata = std::fs::metadata(model_path)?;
        if metadata.len() < 4 {
            return Err(anyhow!("File too small to be a valid GGUF model"));
        }

        // Read first 4 bytes to check magic number
        let mut file = std::fs::File::open(model_path)?;
        let mut magic = [0u8; 4];
        use std::io::Read;
        file.read_exact(&mut magic)?;

        if magic != GGUF_MAGIC {
            return Err(anyhow!("Invalid GGUF magic number"));
        }

        Ok(())
    }

    /// Download a model with progress tracking
    pub async fn download_model<F>(
        &self,
        model_name: &str,
        progress_callback: Option<F>,
    ) -> Result<()>
    where
        F: Fn(LLMDownloadProgress) + Send + 'static,
    {
        let metadata = get_model_by_name(model_name)
            .ok_or_else(|| anyhow!("Model not found in catalog: {}", model_name))?;

        // Check if already downloading
        {
            let active = self.active_downloads.read().await;
            if active.contains(&metadata.name) {
                return Err(anyhow!("Model is already being downloaded"));
            }
        }

        // Mark as active
        {
            let mut active = self.active_downloads.write().await;
            active.insert(metadata.name.clone());
        }

        let result = self
            .download_model_internal(&metadata, progress_callback)
            .await;

        // Remove from active downloads
        {
            let mut active = self.active_downloads.write().await;
            active.remove(&metadata.name);
        }

        result
    }

    /// Internal download implementation
    async fn download_model_internal<F>(
        &self,
        metadata: &crate::llm_engine::config::LLMModelMetadata,
        progress_callback: Option<F>,
    ) -> Result<()>
    where
        F: Fn(LLMDownloadProgress) + Send + 'static,
    {
        let model_path = self.models_dir.join(&metadata.name);
        let temp_path = self.models_dir.join(format!("{}.tmp", metadata.name));

        log::info!("Downloading model {} from {}", metadata.name, metadata.url);

        // Create models directory if needed
        if !self.models_dir.exists() {
            fs::create_dir_all(&self.models_dir).await?;
        }

        // Clean up any incomplete download
        if temp_path.exists() {
            fs::remove_file(&temp_path).await?;
        }

        let client = reqwest::Client::new();
        let response = client.get(&metadata.url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Download failed with status: {}",
                response.status()
            ));
        }

        let total_size = response
            .content_length()
            .unwrap_or(metadata.size_mb * 1024 * 1024);

        let mut file = fs::File::create(&temp_path).await?;
        let mut downloaded: u64 = 0;
        let start_time = Instant::now();
        let mut last_progress_time = Instant::now();

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            // Emit progress every 500ms
            let now = Instant::now();
            if now.duration_since(last_progress_time) >= Duration::from_millis(500) {
                let elapsed = start_time.elapsed().as_secs_f64();
                let speed_mbps = if elapsed > 0.0 {
                    (downloaded as f64 / (1024.0 * 1024.0)) / elapsed
                } else {
                    0.0
                };

                let progress = LLMDownloadProgress::new(downloaded, total_size, speed_mbps);

                if let Some(ref callback) = progress_callback {
                    callback(progress.clone());
                }

                log::debug!(
                    "Download progress: {:.1} MB / {:.1} MB ({:.1} MB/s) - {}%",
                    progress.downloaded_mb,
                    progress.total_mb,
                    progress.speed_mbps,
                    progress.percent
                );

                last_progress_time = now;
            }
        }

        file.flush().await?;
        drop(file);

        // Validate downloaded file
        self.validate_model(&temp_path).await?;

        // Move to final location
        fs::rename(&temp_path, &model_path).await?;

        log::info!("Model downloaded successfully: {}", metadata.name);

        Ok(())
    }

    /// Delete a model
    pub async fn delete_model(&self, model_name: &str) -> Result<()> {
        let model_path = self.models_dir.join(model_name);

        if !model_path.exists() {
            return Err(anyhow!("Model not found: {}", model_name));
        }

        // Check if currently downloading
        let active = self.active_downloads.read().await;
        if active.contains(model_name) {
            return Err(anyhow!("Cannot delete model while it's being downloaded"));
        }
        drop(active);

        fs::remove_file(&model_path).await?;
        log::info!("Model deleted: {}", model_name);

        Ok(())
    }

    /// Get the models directory path
    pub fn get_models_directory(&self) -> &PathBuf {
        &self.models_dir
    }

    /// Import a custom GGUF model from an external file path
    pub async fn import_model(&self, source_path: &Path) -> Result<LLMModelInfo> {
        // Validate source file exists
        if !source_path.exists() {
            return Err(anyhow!("Source file not found: {}", source_path.display()));
        }

        // Get file metadata
        let metadata = std::fs::metadata(source_path)?;
        let file_size = metadata.len();

        // Validate GGUF magic number
        self.validate_model(source_path).await?;

        // Extract filename from source path
        let filename = source_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid filename"))?;

        // Create destination path in models directory
        let dest_path = self.models_dir.join(filename);

        // Check if model with same name already exists
        if dest_path.exists() {
            return Err(anyhow!(
                "A model with the name '{}' already exists in the models directory",
                filename
            ));
        }

        // Create models directory if it doesn't exist
        if !self.models_dir.exists() {
            fs::create_dir_all(&self.models_dir).await?;
        }

        // Copy the file to models directory
        log::info!(
            "Importing model from {} to {}",
            source_path.display(),
            dest_path.display()
        );
        fs::copy(source_path, &dest_path).await?;

        // Calculate size in MB
        let size_mb = file_size / (1024 * 1024);

        // Create model info for imported model
        // Note: We use a default context length since we can't easily extract it from GGUF
        let model_info = LLMModelInfo {
            name: filename.to_string(),
            display_name: format!("Custom: {}", filename),
            size_mb,
            context_length: 4096, // Default context length for imported models
            description: "Imported custom GGUF model".to_string(),
            status: LLMModelStatus::Available,
        };

        log::info!(
            "Model imported successfully: {} ({} MB)",
            filename,
            size_mb
        );

        Ok(model_info)
    }
}
