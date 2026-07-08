use crate::llm_engine::{
    LLMDownloadProgress, LLMModelInfo, LLMModelManager, LlamaEngine,
};
use crate::llm_engine::config::get_model_catalog;
use crate::llm_engine::engine::{detect_available_memory, recommend_model};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::Mutex as TokioMutex;
use tauri::{command, AppHandle, Emitter, Manager, Runtime};

// Global LLM engine - outer Mutex for Option access, inner TokioMutex for async-safe engine access
pub static LLM_ENGINE: Mutex<Option<Arc<TokioMutex<LlamaEngine>>>> = Mutex::new(None);

// Global models directory path
static MODELS_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Initialize the models directory path using app_data_dir
pub fn set_models_directory<R: Runtime>(app: &AppHandle<R>) {
    let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");

    let models_dir = app_data_dir.join("llm_models");

    if !models_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&models_dir) {
            log::error!("Failed to create LLM models directory: {}", e);
            return;
        }
    }

    log::info!("LLM models directory set to: {}", models_dir.display());

    let mut guard = MODELS_DIR.lock().unwrap();
    *guard = Some(models_dir);
}

/// Get the configured models directory
fn get_models_directory() -> Option<PathBuf> {
    MODELS_DIR.lock().unwrap().clone()
}

#[command]
pub async fn llm_init() -> Result<(), String> {
    let models_dir = get_models_directory().ok_or("LLM models directory not initialized")?;

    let mut guard = LLM_ENGINE.lock().unwrap();
    if guard.is_some() {
        return Ok(());
    }

    let engine = LlamaEngine::new(models_dir);
    *guard = Some(Arc::new(TokioMutex::new(engine)));

    log::info!("LLM engine initialized");
    Ok(())
}

#[command]
pub async fn llm_get_available_models() -> Result<Vec<LLMModelInfo>, String> {
    let models_dir = get_models_directory().ok_or("LLM models directory not initialized")?;

    let manager = LLMModelManager::new(models_dir);
    manager
        .discover_models()
        .await
        .map_err(|e| format!("Failed to discover LLM models: {}", e))
}

#[command]
pub async fn llm_download_model<R: Runtime>(
    app_handle: AppHandle<R>,
    model_name: String,
) -> Result<(), String> {
    let models_dir = get_models_directory().ok_or("LLM models directory not initialized")?;

    let manager = LLMModelManager::new(models_dir);

    let app_handle_clone = app_handle.clone();
    let model_name_clone = model_name.clone();

    let progress_callback = Box::new(move |progress: LLMDownloadProgress| {
        log::info!(
            "LLM download progress for {}: {:.1} MB / {:.1} MB ({:.1} MB/s) - {}%",
            model_name_clone,
            progress.downloaded_mb,
            progress.total_mb,
            progress.speed_mbps,
            progress.percent
        );

        if let Err(e) = app_handle_clone.emit(
            "llm-model-download-progress",
            serde_json::json!({
                "modelName": model_name_clone,
                "progress": progress.percent,
                "downloaded_bytes": progress.downloaded_bytes,
                "total_bytes": progress.total_bytes,
                "downloaded_mb": progress.downloaded_mb,
                "total_mb": progress.total_mb,
                "speed_mbps": progress.speed_mbps,
                "status": if progress.percent == 100 {
                    "completed"
                } else {
                    "downloading"
                }
            }),
        ) {
            log::error!("Failed to emit LLM download progress event: {}", e);
        }
    });

    let result = manager
        .download_model(&model_name, Some(progress_callback))
        .await;

    match result {
        Ok(()) => {
            if let Err(e) = app_handle.emit(
                "llm-model-download-complete",
                serde_json::json!({
                    "modelName": model_name
                }),
            ) {
                log::error!("Failed to emit LLM download complete event: {}", e);
            }

            log::info!("LLM model download complete: {}", model_name);
            Ok(())
        }
        Err(e) => {
            if let Err(emit_e) = app_handle.emit(
                "llm-model-download-error",
                serde_json::json!({
                    "modelName": model_name,
                    "error": e.to_string()
                }),
            ) {
                log::error!("Failed to emit LLM download error event: {}", emit_e);
            }
            Err(format!("Failed to download LLM model: {}", e))
        }
    }
}

#[command]
pub async fn llm_delete_model(model_name: String) -> Result<(), String> {
    let models_dir = get_models_directory().ok_or("LLM models directory not initialized")?;

    let manager = LLMModelManager::new(models_dir);
    manager
        .delete_model(&model_name)
        .await
        .map_err(|e| format!("Failed to delete LLM model: {}", e))
}

#[command]
pub async fn llm_load_model<R: Runtime>(
    app_handle: AppHandle<R>,
    model_name: String,
) -> Result<(), String> {
    let engine = {
        let guard = LLM_ENGINE.lock().unwrap();
        guard.as_ref().cloned()
    };

    if let Some(engine) = engine {
        if let Err(e) = app_handle.emit(
            "llm-model-loading-started",
            serde_json::json!({
                "modelName": model_name
            }),
        ) {
            log::error!("Failed to emit LLM model loading started event: {}", e);
        }

        let result = {
            let mut engine_guard = engine.lock().await;
            engine_guard.load_model(&model_name).await
        };

        match result {
            Ok(()) => {
                if let Err(e) = app_handle.emit(
                    "llm-model-loading-completed",
                    serde_json::json!({
                        "modelName": model_name
                    }),
                ) {
                    log::error!("Failed to emit LLM model loading completed event: {}", e);
                }
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                if let Err(e) = app_handle.emit(
                    "llm-model-loading-failed",
                    serde_json::json!({
                        "modelName": model_name,
                        "error": error_msg
                    }),
                ) {
                    log::error!("Failed to emit LLM model loading failed event: {}", e);
                }
                Err(format!("Failed to load LLM model: {}", error_msg))
            }
        }
    } else {
        Err("LLM engine not initialized".to_string())
    }
}

#[command]
pub async fn llm_unload_model() -> Result<(), String> {
    let engine = {
        let guard = LLM_ENGINE.lock().unwrap();
        guard.as_ref().cloned()
    };

    if let Some(engine) = engine {
        let mut engine_guard = engine.lock().await;
        engine_guard.unload_model().await;
        Ok(())
    } else {
        Err("LLM engine not initialized".to_string())
    }
}

#[command]
pub async fn llm_is_model_loaded() -> Result<bool, String> {
    let engine = {
        let guard = LLM_ENGINE.lock().unwrap();
        guard.as_ref().cloned()
    };

    if let Some(engine) = engine {
        let engine_guard = engine.lock().await;
        Ok(engine_guard.is_model_loaded())
    } else {
        Ok(false)
    }
}

#[command]
pub async fn llm_get_current_model() -> Result<Option<String>, String> {
    let engine = {
        let guard = LLM_ENGINE.lock().unwrap();
        guard.as_ref().cloned()
    };

    if let Some(engine) = engine {
        let engine_guard = engine.lock().await;
        Ok(engine_guard.get_current_model_name())
    } else {
        Ok(None)
    }
}

#[command]
pub async fn llm_get_models_directory() -> Result<String, String> {
    let models_dir = get_models_directory().ok_or("LLM models directory not initialized")?;
    Ok(models_dir.to_string_lossy().to_string())
}

#[command]
pub async fn llm_get_gpu_info() -> Result<crate::llm_engine::engine::GPUAccelerationInfo, String> {
    Ok(LlamaEngine::get_gpu_info())
}

/// Open the LLM models folder in the system file explorer
#[command]
pub async fn open_llm_models_folder() -> Result<(), String> {
    let models_dir = get_models_directory()
        .ok_or_else(|| "LLM models directory not initialized".to_string())?;

    if !models_dir.exists() {
        std::fs::create_dir_all(&models_dir)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let folder_path = models_dir.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&folder_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&folder_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&folder_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    log::info!("Opened LLM models folder: {}", folder_path);
    Ok(())
}

/// Recommend the best model for the user's hardware
#[command]
pub async fn llm_recommend_model() -> Result<Option<String>, String> {
    // Detect available memory
    let available_memory = detect_available_memory()
        .map_err(|e| format!("Failed to detect available memory: {}", e))?;

    log::info!("Detected available memory: {} MB", available_memory / (1024 * 1024));

    // Get model catalog with sizes
    let models: Vec<(String, u64)> = get_model_catalog()
        .into_iter()
        .map(|m| (m.name, m.size_mb))
        .collect();

    // Recommend best model
    let recommendation = recommend_model(available_memory, &models);

    if let Some(ref model_name) = recommendation {
        log::info!("Recommended model: {}", model_name);
    } else {
        log::warn!("No model recommended - insufficient memory for any model");
    }

    Ok(recommendation)
}
