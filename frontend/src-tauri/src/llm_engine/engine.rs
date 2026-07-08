use anyhow::{anyhow, Result};
use llama_cpp_4::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{params::LlamaModelParams, AddBos, LlamaModel, Special},
    sampling::LlamaSampler,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

/// Chat message for LLM inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Configuration for text generation
#[derive(Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub repeat_penalty: f32,
    /// Context window size in tokens. Determines how many tokens the model can
    /// consider at once (prompt + generation). Defaults to 4096 for backward
    /// compatibility; callers should set this from the model catalog.
    pub context_size: u32,
    /// Optional callback invoked with each decoded token during inference.
    /// Used to stream tokens to the frontend in real-time.
    #[serde(skip)]
    pub streaming_callback: Option<Arc<dyn Fn(&str) + Send + Sync>>,
}

impl std::fmt::Debug for GenerationConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenerationConfig")
            .field("max_tokens", &self.max_tokens)
            .field("temperature", &self.temperature)
            .field("top_p", &self.top_p)
            .field("top_k", &self.top_k)
            .field("repeat_penalty", &self.repeat_penalty)
            .field("context_size", &self.context_size)
            .field("streaming_callback", &self.streaming_callback.as_ref().map(|_| "callback_set"))
            .finish()
    }
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_tokens: 2048,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            context_size: 4096,
            streaming_callback: None,
        }
    }
}

/// GPU acceleration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUAccelerationInfo {
    pub backend: String,
    pub enabled: bool,
    pub description: String,
}

/// Core LLM engine wrapping llama-cpp-4
pub struct LlamaEngine {
    backend: Arc<LlamaBackend>,
    model: Option<Arc<LlamaModel>>,
    model_path: Option<PathBuf>,
    models_dir: PathBuf,
}

impl LlamaEngine {
    /// Create a new LlamaEngine with the specified models directory
    pub fn new(models_dir: PathBuf) -> Self {
        let backend = LlamaBackend::init().expect("Failed to initialize LlamaBackend");
        Self {
            backend: Arc::new(backend),
            model: None,
            model_path: None,
            models_dir,
        }
    }

    /// Load a GGUF model from the models directory
    pub async fn load_model(&mut self, model_name: &str) -> Result<()> {
        let model_path = self.models_dir.join(model_name);

        if !model_path.exists() {
            return Err(anyhow!("Model file not found: {}", model_path.display()));
        }

        info!("Loading LLM model from: {}", model_path.display());

        // Load model - create params inside closure to avoid Send issues
        let model = tokio::task::spawn_blocking({
            let backend = self.backend.clone();
            let model_path = model_path.clone();
            move || {
                let mut model_params = LlamaModelParams::default();
                model_params = model_params.with_n_gpu_layers(99);
                
                LlamaModel::load_from_file(&backend, &model_path, &model_params)
                    .map_err(|e| anyhow!("Failed to load model: {:?}", e))
            }
        })
        .await??;

        self.model = Some(Arc::new(model));
        self.model_path = Some(model_path);

        Ok(())
    }

    /// Unload the current model and free resources
    pub async fn unload_model(&mut self) {
        info!("Unloading LLM model...");
        self.model = None;
        self.model_path = None;
        info!("LLM model unloaded");
    }

    /// Check if a model is currently loaded
    pub fn is_model_loaded(&self) -> bool {
        self.model.is_some()
    }

    /// Get the name of the currently loaded model
    pub fn get_current_model_name(&self) -> Option<String> {
        self.model_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    }

    /// Perform chat completion using the loaded model
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        config: GenerationConfig,
    ) -> Result<String> {
        let model = self
            .model
            .as_ref()
            .ok_or_else(|| anyhow!("No model loaded"))?
            .clone();

        // Format messages into a prompt
        let prompt = self.format_chat_prompt(messages);

        info!("Generating completion with prompt length: {}", prompt.len());

        // Run inference in blocking task
        let result = tokio::task::spawn_blocking({
            let backend = self.backend.clone();
            let model = model.clone();
            let context_size = config.context_size;
            let streaming_callback = config.streaming_callback.clone();
            let config = config.clone();
            move || {
                // Create context params inside the closure to avoid Send issues
                let ctx_params = LlamaContextParams::default()
                    .with_n_ctx(std::num::NonZeroU32::new(context_size))
                    .with_n_batch(512);

                let mut context = model
                    .new_context(&backend, ctx_params)
                    .map_err(|e| anyhow!("Failed to create context: {:?}", e))?;

                Self::run_inference(&model, &mut context, &prompt, config, streaming_callback)
            }
        })
        .await??;

        Ok(result)
    }

    /// Format chat messages into a prompt string
    fn format_chat_prompt(&self, messages: Vec<ChatMessage>) -> String {
        let mut prompt = String::new();

        for msg in messages {
            match msg.role.to_lowercase().as_str() {
                "system" => {
                    prompt.push_str(&format!("System: {}\n\n", msg.content));
                }
                "user" => {
                    prompt.push_str(&format!("User: {}\n\n", msg.content));
                }
                "assistant" => {
                    prompt.push_str(&format!("Assistant: {}\n\n", msg.content));
                }
                _ => {
                    prompt.push_str(&format!("{}: {}\n\n", msg.role, msg.content));
                }
            }
        }

        prompt.push_str("Assistant:");
        prompt
    }

    /// Run inference on the model (blocking)
    fn run_inference(
        model: &LlamaModel,
        context: &mut llama_cpp_4::context::LlamaContext<'_>,
        prompt: &str,
        config: GenerationConfig,
        streaming_callback: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    ) -> Result<String> {
        // Tokenize the prompt
        let tokens = model
            .str_to_token(prompt, AddBos::Always)
            .map_err(|e| anyhow!("Failed to tokenize prompt: {:?}", e))?;

        if tokens.is_empty() {
            return Err(anyhow!("Prompt tokenization resulted in empty token list"));
        }

        // Create sampler chain (no_perf=false to include perf tracking)
        let sampler = LlamaSampler::chain(
            vec![
                LlamaSampler::top_k(config.top_k),
                LlamaSampler::top_p(config.top_p, 1),
                LlamaSampler::temp(config.temperature),
                LlamaSampler::dist(1234),
            ],
            false,
        );

        // Create batch for processing
        let mut batch = LlamaBatch::new(512, 1);

        // Process prompt tokens
        for (i, token) in tokens.iter().enumerate() {
            batch.add(*token, i as i32, &[0], false)?;
        }

        // Decode prompt
        context.decode(&mut batch)?;

        // Generate tokens
        let mut output = String::new();
        let mut n_cur = tokens.len() as i32;
        let max_tokens = config.max_tokens as usize;

        for _ in 0..max_tokens {
            // Sample next token (sampler takes context and position index)
            let next_token = sampler.sample(context, -1);

            // Check for EOS token
            if model.is_eog_token(next_token) {
                break;
            }

            // Decode token to string
            let token_str = model
                .token_to_str(next_token, Special::Tokenize)
                .map_err(|e| anyhow!("Failed to decode token: {:?}", e))?;

            output.push_str(&token_str);

            // Stream token to frontend if callback is present
            if let Some(ref callback) = streaming_callback {
                callback(&token_str);
            }

            // Create batch for single token
            let mut batch = LlamaBatch::new(512, 1);
            batch.add(next_token, n_cur, &[0], false)?;

            // Decode next token
            context.decode(&mut batch)?;

            n_cur += 1;
        }

        Ok(output.trim().to_string())
    }

    /// Get GPU acceleration information
    pub fn get_gpu_info() -> GPUAccelerationInfo {
        #[cfg(feature = "metal")]
        {
            GPUAccelerationInfo {
                backend: "Metal".to_string(),
                enabled: true,
                description: "Apple Metal GPU acceleration enabled".to_string(),
            }
        }
        #[cfg(feature = "cuda")]
        {
            GPUAccelerationInfo {
                backend: "CUDA".to_string(),
                enabled: true,
                description: "NVIDIA CUDA GPU acceleration enabled".to_string(),
            }
        }
        #[cfg(feature = "vulkan")]
        {
            GPUAccelerationInfo {
                backend: "Vulkan".to_string(),
                enabled: true,
                description: "Vulkan GPU acceleration enabled".to_string(),
            }
        }
        #[cfg(not(any(feature = "metal", feature = "cuda", feature = "vulkan")))]
        {
            GPUAccelerationInfo {
                backend: "CPU".to_string(),
                enabled: false,
                description: "CPU-only mode (no GPU acceleration)".to_string(),
            }
        }
    }

    /// Get the models directory path
    pub fn get_models_directory(&self) -> &PathBuf {
        &self.models_dir
    }
}

/// Detect available system memory in bytes.
/// Uses available system RAM as the primary metric.
/// On macOS with Metal, this accurately reflects GPU-available memory
/// since Apple Silicon uses unified memory. For discrete GPUs (CUDA/Vulkan),
/// this is a conservative estimate that prevents OOM in most cases.
pub fn detect_available_memory() -> Result<u64> {
    use sysinfo::System;

    let mut sys = System::new_all();
    sys.refresh_memory();
    let available = sys.available_memory();
    
    let gpu_info = LlamaEngine::get_gpu_info();
    info!(
        "{} mode - available system RAM: {} MB",
        gpu_info.backend,
        available / (1024 * 1024)
    );
    
    Ok(available)
}

/// Recommend the best model for available hardware.
/// Returns the largest model that fits in available memory with 20% headroom,
/// accounting for ~1.5x inference overhead (model must fit in memory at runtime).
///
/// `available_memory` is in bytes.
/// `models` is a slice of (model_name, size_mb) tuples.
pub fn recommend_model(available_memory: u64, models: &[(String, u64)]) -> Option<String> {
    // Runtime memory estimate: file_size_mb * 1.5 (for inference overhead)
    // Available memory with 20% headroom: available_memory * 0.8
    let usable_bytes = (available_memory as f64 * 0.8) as u64;

    // Convert to MB for comparison
    let usable_mb = usable_bytes / (1024 * 1024);

    // Find models that fit, sorted by size descending (prefer largest that fits)
    let mut fitting_models: Vec<(&String, u64)> = models
        .iter()
        .filter(|(_, size_mb)| {
            let runtime_mb = (*size_mb as f64 * 1.5) as u64;
            runtime_mb <= usable_mb
        })
        .map(|(name, size)| (name, *size))
        .collect();

    // Sort by size descending - recommend the largest model that fits
    fitting_models.sort_by(|a, b| b.1.cmp(&a.1));

    fitting_models.first().map(|(name, _)| (*name).clone())
}

impl Drop for LlamaEngine {
    fn drop(&mut self) {
        info!("Dropping LlamaEngine, releasing GPU memory");
        // Context and model will be dropped automatically
    }
}
