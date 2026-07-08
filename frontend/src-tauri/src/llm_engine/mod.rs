//! Local LLM inference engine module using llama-cpp-4.
//!
//! This module provides embedded local LLM inference using GGUF models,
//! allowing users to run models (Qwen 2.5, Llama 3, Gemma 2) without
//! installing Ollama or any external software.
//!
//! # Architecture
//!
//! Follows the same pattern as `parakeet_engine` and `whisper_engine`:
//! - `config`: Model catalog and metadata
//! - `engine`: Core LlamaEngine wrapper for inference
//! - `model_manager`: Download, validate, list, delete models
//! - `commands`: Tauri command interface for frontend
//!
//! # Storage
//!
//! Models are stored in `app_data_dir/llm_models/`:
//! - macOS: `~/Library/Application Support/Twin/llm_models/`
//! - Windows: `%APPDATA%\Twin\llm_models\`
//! - Linux: `~/.config/Twin/llm_models/`

pub mod commands;
pub mod config;
pub mod engine;
pub mod model_manager;

pub use commands::*;
pub use config::{get_model_catalog, LLMModelMetadata};
pub use engine::{ChatMessage, GenerationConfig, LlamaEngine};
pub use model_manager::{LLMDownloadProgress, LLMModelInfo, LLMModelManager, LLMModelStatus};
