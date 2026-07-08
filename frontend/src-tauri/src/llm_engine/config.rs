use serde::{Deserialize, Serialize};

/// Metadata for a downloadable LLM model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMModelMetadata {
    /// Internal name / filename (e.g. "qwen2.5-1.5b-instruct-q4_k_m.gguf")
    pub name: String,
    /// Human-readable display name
    pub display_name: String,
    /// Download URL (HuggingFace)
    pub url: String,
    /// Approximate download size in MB
    pub size_mb: u64,
    /// Maximum context length for this model
    pub context_length: u32,
    /// Short description of the model's strengths
    pub description: String,
}

/// Pre-configured model catalog for local LLM inference
pub fn get_model_catalog() -> Vec<LLMModelMetadata> {
    vec![
        LLMModelMetadata {
            name: "qwen2.5-1.5b-instruct-q4_k_m.gguf".to_string(),
            display_name: "Qwen 2.5 1.5B (Recommended)".to_string(),
            url: "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/qwen2.5-1.5b-instruct-q4_k_m.gguf".to_string(),
            size_mb: 986,
            context_length: 32768,
            description: "Fast, good quality, low resource usage".to_string(),
        },
        LLMModelMetadata {
            name: "Llama-3.2-3B-Instruct-Q4_K_M.gguf".to_string(),
            display_name: "Llama 3.2 3B".to_string(),
            url: "https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF/resolve/main/Llama-3.2-3B-Instruct-Q4_K_M.gguf".to_string(),
            size_mb: 1930,
            context_length: 131072,
            description: "Balanced performance and quality".to_string(),
        },
        LLMModelMetadata {
            name: "gemma-2-2b-it-Q4_K_M.gguf".to_string(),
            display_name: "Gemma 2 2B".to_string(),
            url: "https://huggingface.co/bartowski/gemma-2-2b-it-GGUF/resolve/main/gemma-2-2b-it-Q4_K_M.gguf".to_string(),
            size_mb: 1460,
            context_length: 8192,
            description: "Google's efficient model".to_string(),
        },
    ]
}

/// GGUF magic number: "GGUF" in little-endian
pub const GGUF_MAGIC: [u8; 4] = [0x47, 0x47, 0x55, 0x46];

/// Get model metadata by name
pub fn get_model_by_name(name: &str) -> Option<LLMModelMetadata> {
    get_model_catalog().into_iter().find(|m| m.name == name)
}
