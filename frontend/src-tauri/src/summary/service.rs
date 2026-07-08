use crate::database::repositories::{
    meeting::MeetingsRepository, setting::SettingsRepository, summary::SummaryProcessesRepository,
    transcript::TranscriptsRepository,
};
use crate::summary::llm_client::LLMProvider;
use crate::summary::language_detection::detect_summary_language;
use crate::summary::metadata::read_detected_summary_language_from_metadata;
use crate::summary::processor::{
    extract_meeting_name_from_markdown, generate_meeting_summary, language_name_from_code,
    rough_token_count,
};
use crate::summary::templates::{self, Template};
use crate::ollama::metadata::ModelMetadataCache;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use once_cell::sync::Lazy;

// Global cache for model metadata (5 minute TTL)
static METADATA_CACHE: Lazy<ModelMetadataCache> = Lazy::new(|| {
    ModelMetadataCache::new(Duration::from_secs(300))
});

// Global registry for cancellation tokens (thread-safe)
static CANCELLATION_REGISTRY: Lazy<Arc<Mutex<HashMap<String, CancellationToken>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));
// Global HTTP client singleton to prevent connection pool exhaustion
static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);

/// Tracks inference progress metrics for emitting periodic events
struct InferenceProgressState {
    tokens_generated: u32,
    start_time: Instant,
    last_emit_time: Option<Instant>,
}

impl InferenceProgressState {
    fn new() -> Self {
        Self {
            tokens_generated: 0,
            start_time: Instant::now(),
            last_emit_time: None,
        }
    }

    fn record_token(&mut self) {
        self.tokens_generated += 1;
    }

    /// Returns true if 500ms have elapsed since last emit (or if never emitted)
    fn should_emit_progress(&self) -> bool {
        match self.last_emit_time {
            None => true,
            Some(last) => last.elapsed() >= Duration::from_millis(500),
        }
    }
}

/// Strips the first `#` heading line; returns "" if no `#` is found.
fn strip_leading_title(markdown: &str) -> String {
    if let Some(hash_pos) = markdown.find('#') {
        let body_start = markdown[hash_pos..]
            .find('\n')
            .map_or(markdown.len(), |line_end| hash_pos + line_end);
        markdown[body_start..].trim_start().to_string()
    } else {
        String::new()
    }
}

/// Strips the leading H1 (`# Title\n...`) only when the markdown starts with one.
/// No-op on already-stripped values, values starting with `## Subheading`, or values
/// without any heading. Avoids the silent-empty-return case where `strip_leading_title`
/// returns "" for input lacking a leading `#`.
fn strip_title_if_present(markdown: &str) -> String {
    if markdown.trim_start().starts_with("# ") {
        strip_leading_title(markdown)
    } else {
        markdown.to_string()
    }
}

const ENGLISH_CACHE_FIELD: &str = "english_cache";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SummaryCacheSource {
    transcript_fingerprint: String,
    custom_prompt_fingerprint: String,
    template_id: String,
    template_fingerprint: String,
    token_threshold: usize,
    model_provider: String,
    model_name: String,
    ollama_endpoint: Option<String>,
    custom_openai_endpoint: Option<String>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct EnglishSummaryCache {
    markdown: String,
    source: SummaryCacheSource,
    output_language: Option<String>,
}

fn stable_text_fingerprint(text: &str) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in text.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{:016x}:{}", hash, text.len())
}

#[allow(clippy::too_many_arguments)]
fn build_summary_cache_source(
    text: &str,
    custom_prompt: &str,
    template_id: &str,
    template_fingerprint: &str,
    token_threshold: usize,
    model_provider: &str,
    model_name: &str,
    ollama_endpoint: Option<&str>,
    custom_openai_endpoint: Option<&str>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
) -> SummaryCacheSource {
    SummaryCacheSource {
        transcript_fingerprint: stable_text_fingerprint(text),
        custom_prompt_fingerprint: stable_text_fingerprint(custom_prompt),
        template_id: template_id.to_string(),
        template_fingerprint: template_fingerprint.to_string(),
        token_threshold,
        model_provider: model_provider.to_string(),
        model_name: model_name.to_string(),
        ollama_endpoint: ollama_endpoint.map(str::to_string),
        custom_openai_endpoint: custom_openai_endpoint.map(str::to_string),
        max_tokens,
        temperature,
        top_p,
    }
}

fn template_cache_fingerprint(template: &Template) -> String {
    let rendered_template = format!(
        "{}\n---SECTION-INSTRUCTIONS---\n{}",
        template.to_markdown_structure(),
        template.to_section_instructions()
    );
    stable_text_fingerprint(&rendered_template)
}

fn normalise_summary_language_for_cache(summary_language: Option<&str>) -> Option<String> {
    language_name_from_code(summary_language?.trim()).map(str::to_string)
}

fn build_summary_result_json(
    final_markdown: &str,
    english_markdown: &str,
    source: SummaryCacheSource,
    output_language: Option<&str>,
    quality_score: u8,
) -> serde_json::Value {
    serde_json::json!({
        "markdown": strip_title_if_present(final_markdown),
        "quality_score": quality_score,
        ENGLISH_CACHE_FIELD: EnglishSummaryCache {
            markdown: english_markdown.to_string(),
            source,
            output_language: normalise_summary_language_for_cache(output_language),
        },
    })
}

/// Parses a `summary_processes.result` JSON blob and extracts a cached English
/// summary only when it was produced from exactly the same source inputs and
/// the user is switching to a different non-English target language.
fn extract_cached_english_markdown(
    raw: &str,
    expected_source: &SummaryCacheSource,
    requested_language: Option<&str>,
) -> Result<Option<String>, serde_json::Error> {
    let requested_language = match normalise_summary_language_for_cache(requested_language) {
        Some(language) if language != "English" => language,
        _ => return Ok(None),
    };

    let value: serde_json::Value = serde_json::from_str(raw)?;
    let Some(cache_value) = value.get(ENGLISH_CACHE_FIELD) else {
        return Ok(None);
    };

    let cache: EnglishSummaryCache = match serde_json::from_value(cache_value.clone()) {
        Ok(cache) => cache,
        Err(_) => return Ok(None),
    };

    if cache.source != *expected_source {
        return Ok(None);
    }

    if cache.output_language.as_deref() == Some(requested_language.as_str()) {
        return Ok(None);
    }

    let markdown = cache.markdown.trim();
    if markdown.is_empty() {
        Ok(None)
    } else {
        Ok(Some(cache.markdown))
    }
}

/// Summary service - handles all summary generation logic
pub struct SummaryService;

impl SummaryService {
    /// Registers a new cancellation token for a meeting
    fn register_cancellation_token(meeting_id: &str) -> CancellationToken {
        let token = CancellationToken::new();
        if let Ok(mut registry) = CANCELLATION_REGISTRY.lock() {
            registry.insert(meeting_id.to_string(), token.clone());
            info!("Registered cancellation token for meeting: {}", meeting_id);
        }
        token
    }

    /// Cancels the summary generation for a meeting
    pub fn cancel_summary(meeting_id: &str) -> bool {
        if let Ok(registry) = CANCELLATION_REGISTRY.lock() {
            if let Some(token) = registry.get(meeting_id) {
                info!("Cancelling summary generation for meeting: {}", meeting_id);
                token.cancel();
                return true;
            }
        }
        warn!("No active summary generation found for meeting: {}", meeting_id);
        false
    }

    /// Cleans up the cancellation token after processing completes
    fn cleanup_cancellation_token(meeting_id: &str) {
        if let Ok(mut registry) = CANCELLATION_REGISTRY.lock() {
            if registry.remove(meeting_id).is_some() {
                info!("Cleaned up cancellation token for meeting: {}", meeting_id);
            }
        }
    }

    /// Auto-generates a summary for a meeting after recording stops
    ///
    /// This method is called automatically when a recording ends if auto-summarize is enabled.
    /// It loads the transcript, uses the last-used template and parameters, and triggers
    /// background summary generation.
    pub async fn auto_summarize_meeting<R: tauri::Runtime>(
        app: AppHandle<R>,
        pool: SqlitePool,
        meeting_id: &str,
    ) -> Result<(), String> {
        info!("Auto-summarizing meeting: {}", meeting_id);

        // Load transcript for this meeting
        let transcript = match TranscriptsRepository::get_transcripts_for_meeting(&pool, meeting_id).await {
            Ok(transcripts) => {
                if transcripts.is_empty() {
                    warn!("No transcripts found for meeting {}, skipping auto-summarize", meeting_id);
                    return Err("No transcripts available".to_string());
                }
                // Concatenate all transcript segments
                transcripts.into_iter()
                    .map(|t| t.transcript)
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            Err(e) => {
                error!("Failed to load transcripts for auto-summarize: {}", e);
                return Err(format!("Failed to load transcripts: {}", e));
            }
        };

        // Get current model config to use for summarization
        let config = match SettingsRepository::get_model_config(&pool).await {
            Ok(Some(config)) => config,
            Ok(None) => {
                warn!("No model config found, using defaults");
                // Return default config
                crate::database::models::Setting {
                    id: "1".to_string(),
                    provider: "openai".to_string(),
                    model: "gpt-4o-mini".to_string(),
                    whisper_model: "large-v3".to_string(),
                    openai_api_key: None,
                    anthropic_api_key: None,
                    ollama_api_key: None,
                    ollama_endpoint: None,
                    custom_openai_config: None,
                    auto_summarize_enabled: 1,
                }
            }
            Err(e) => {
                error!("Failed to load model config: {}", e);
                return Err(format!("Failed to load model config: {}", e));
            }
        };

        // Use default template and parameters
        let template_id = "standard_meeting".to_string();
        let custom_prompt = "".to_string();
        let summary_language = None;
        let max_tokens = None;
        let temperature = None;
        let top_p = None;
        let top_k = None;

        info!("Auto-summarize: using provider={}, model={}, template={}", 
              config.provider, config.model, template_id);

        // Create or reset the process entry
        if let Err(e) = SummaryProcessesRepository::create_or_reset_process(&pool, meeting_id).await {
            error!("Failed to create summary process: {}", e);
            return Err(format!("Failed to create summary process: {}", e));
        }

        // Spawn background task for processing
        let meeting_id_clone = meeting_id.to_string();
        tauri::async_runtime::spawn(async move {
            Self::process_transcript_background(
                app,
                pool,
                meeting_id_clone,
                transcript,
                config.provider,
                config.model,
                custom_prompt,
                template_id,
                summary_language,
                max_tokens,
                temperature,
                top_p,
                top_k,
            )
            .await;
        });

        info!("Auto-summarize task spawned for meeting: {}", meeting_id);
        Ok(())
    }

    /// Scores the quality of a generated summary
    ///
    /// Uses heuristic analysis to evaluate summary quality:
    /// - Section presence (Key Points, Action Items, Decisions, etc.)
    /// - Length appropriateness (not too short, not too long)
    /// - Keyword coverage from transcript
    ///
    /// Returns a score from 0-5 where:
    /// - 5: Excellent (all sections present, good length, high keyword coverage)
    /// - 4: Good (most sections present, appropriate length)
    /// - 3: Fair (some sections present, acceptable length)
    /// - 2: Poor (missing sections or poor length)
    /// - 1: Very poor (minimal content)
    /// - 0: Failed (empty or invalid)
    pub fn score_summary_quality(summary_markdown: &str, transcript_text: &str) -> u8 {
        if summary_markdown.trim().is_empty() {
            return 0;
        }

        let mut score = 0u8;

        // 1. Check for key sections (max 2 points)
        let sections = [
            "key points", "action items", "decisions", "summary",
            "attendees", "next steps", "discussion", "agenda"
        ];
        let summary_lower = summary_markdown.to_lowercase();
        let section_count = sections.iter()
            .filter(|&&section| summary_lower.contains(section))
            .count();

        match section_count {
            4..=8 => score += 2,
            2..=3 => score += 1,
            _ => {}
        }

        // 2. Check length appropriateness (max 2 points)
        let summary_words = summary_markdown.split_whitespace().count();
        let transcript_words = transcript_text.split_whitespace().count();

        // Summary should be 5-20% of transcript length, with reasonable absolute bounds
        let ratio = if transcript_words > 0 {
            summary_words as f32 / transcript_words as f32
        } else {
            0.0
        };

        if summary_words >= 100 && summary_words <= 2000 {
            if ratio >= 0.05 && ratio <= 0.20 {
                score += 2; // Ideal length
            } else if ratio >= 0.03 && ratio <= 0.30 {
                score += 1; // Acceptable length
            }
        } else if summary_words >= 50 {
            score += 1; // At least has some content
        }

        // 3. Check keyword coverage (max 1 point)
        // Extract important words from transcript (simple heuristic: words > 5 chars)
        let transcript_keywords: std::collections::HashSet<String> = transcript_text
            .split_whitespace()
            .filter(|w| w.len() > 5)
            .map(|w| w.to_lowercase())
            .take(100) // Sample first 100 keywords
            .collect();

        let summary_keywords: std::collections::HashSet<String> = summary_markdown
            .split_whitespace()
            .filter(|w| w.len() > 5)
            .map(|w| w.to_lowercase())
            .collect();

        let overlap = transcript_keywords.intersection(&summary_keywords).count();
        let coverage = if !transcript_keywords.is_empty() {
            overlap as f32 / transcript_keywords.len() as f32
        } else {
            0.0
        };

        if coverage >= 0.10 {
            score += 1; // Good keyword coverage
        }

        info!("Summary quality score: {}/5 (sections: {}, length: {} words, keyword coverage: {:.1}%)",
              score, section_count, summary_words, coverage * 100.0);

        score
    }

    /// Checks if the transcript is appropriate for the selected model's context window
    ///
    /// Analyzes transcript length and compares it to the model's context window.
    /// Emits a warning event if the transcript exceeds 80% of the context window.
    ///
    /// Returns a warning message if the transcript is too long, None otherwise.
    pub async fn check_transcript_capacity<R: tauri::Runtime>(
        app: &AppHandle<R>,
        pool: &SqlitePool,
        meeting_id: &str,
        model_provider: &str,
        model_name: &str,
    ) -> Option<String> {
        // Load transcript for this meeting
        let transcript = match TranscriptsRepository::get_transcripts_for_meeting(pool, meeting_id).await {
            Ok(transcripts) => {
                transcripts.into_iter()
                    .map(|t| t.transcript)
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            Err(e) => {
                warn!("Failed to load transcripts for capacity check: {}", e);
                return None;
            }
        };

        let transcript_tokens = rough_token_count(&transcript);

        // Get model context window
        let context_window = if model_provider == "ollama" {
            match METADATA_CACHE.get_or_fetch(model_name, None).await {
                Ok(metadata) => metadata.context_size,
                Err(e) => {
                    warn!("Failed to fetch context for {}: {}, using default 4096", model_name, e);
                    4096
                }
            }
        } else if model_provider == "local-llm" {
            // Local LLM models typically have 4096 context
            4096
        } else {
            // Cloud providers have large context windows
            128000
        };

        // Check if transcript exceeds 80% of context window
        let threshold = (context_window as f32 * 0.8) as usize;

        if transcript_tokens > threshold {
            let warning = format!(
                "This meeting has {} tokens. {} may struggle with this length (context: {} tokens). Consider using a cloud provider or a model with larger context.",
                transcript_tokens, model_name, context_window
            );

            warn!("Transcript capacity warning: {}", warning);

            // Emit warning event to frontend
            let _ = app.emit("model-capability-warning", serde_json::json!({
                "meeting_id": meeting_id,
                "transcript_tokens": transcript_tokens,
                "model_context_window": context_window,
                "model_name": model_name,
                "warning": warning,
            }));

            return Some(warning);
        }

        info!("Transcript capacity check passed: {} tokens vs {} context window", 
              transcript_tokens, context_window);
        None
    }

    async fn read_detected_summary_language(
        pool: &SqlitePool,
        meeting_id: &str,
    ) -> Option<String> {
        let meeting = match MeetingsRepository::get_meeting_metadata(pool, meeting_id).await {
            Ok(Some(meeting)) => meeting,
            Ok(None) => {
                warn!("Meeting not found while reading detected summary language: {}", meeting_id);
                return None;
            }
            Err(e) => {
                warn!(
                    "Failed to read meeting metadata for detected summary language (meeting_id={}): {}",
                    meeting_id, e
                );
                return None;
            }
        };

        let Some(folder_path) = meeting.folder_path.filter(|p| !p.trim().is_empty()) else {
            return None;
        };

        match read_detected_summary_language_from_metadata(Path::new(&folder_path)) {
            Ok(language) => language,
            Err(e) => {
                warn!(
                    "Failed to read detected summary language metadata for meeting_id={}: {}",
                    meeting_id, e
                );
                None
            }
        }
    }

    fn detect_summary_language_from_text(text: &str) -> Option<String> {
        let transcript_texts = [text.to_string()];
        let detection = detect_summary_language(&transcript_texts);
        match &detection.language {
            Some(language) => {
                info!("Detected transcript summary language for normalization: {}", language);
            }
            None => {
                info!(
                    "Transcript summary language unknown for normalization: {:?}",
                    detection.reason
                );
            }
        }
        detection.language
    }

    /// Processes transcript in the background and generates summary
    ///
    /// This function is designed to be spawned as an async task and does not block
    /// the main thread. It updates the database with progress and results.
    ///
    /// # Arguments
    /// * `_app` - Tauri app handle (for future use)
    /// * `pool` - SQLx connection pool
    /// * `meeting_id` - Unique identifier for the meeting
    /// * `text` - Full transcript text
    /// * `model_provider` - LLM provider name (e.g., "ollama", "openai")
    /// * `model_name` - Specific model (e.g., "gpt-4", "llama3.2:latest")
    /// * `custom_prompt` - Optional user-provided context
    /// * `template_id` - Template identifier (e.g., "daily_standup", "standard_meeting")
    pub async fn process_transcript_background<R: tauri::Runtime>(
        _app: AppHandle<R>,
        pool: SqlitePool,
        meeting_id: String,
        text: String,
        model_provider: String,
        model_name: String,
        custom_prompt: String,
        template_id: String,
        summary_language: Option<String>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
        top_p: Option<f32>,
        top_k: Option<i32>,
    ) {
        let start_time = Instant::now();
        info!(
            "Starting background processing for meeting_id: {}",
            meeting_id
        );

        // Register cancellation token for this meeting
        let cancellation_token = Self::register_cancellation_token(&meeting_id);

        // Parse provider
        let provider = match LLMProvider::from_str(&model_provider) {
            Ok(p) => p,
            Err(e) => {
                Self::update_process_failed(&pool, &meeting_id, &e).await;
                return;
            }
        };

        // Validate and setup api_key, Flexible for Ollama, CustomOpenAI, and LocalLLM
        let api_key = if provider == LLMProvider::Ollama || provider == LLMProvider::CustomOpenAI || provider == LLMProvider::LocalLLM {
            // These providers don't require API keys from the standard database column
            String::new()
        } else {
            match SettingsRepository::get_api_key(&pool, &model_provider).await {
                Ok(Some(key)) if !key.is_empty() => key,
                Ok(None) | Ok(Some(_)) => {
                    let err_msg = format!("API key not found for {}", &model_provider);
                    Self::update_process_failed(&pool, &meeting_id, &err_msg).await;
                    return;
                }
                Err(e) => {
                    let err_msg = format!("Failed to retrieve API key for {}: {}", &model_provider, e);
                    Self::update_process_failed(&pool, &meeting_id, &err_msg).await;
                    return;
                }
            }
        };

        // Get Ollama endpoint if provider is Ollama
        let ollama_endpoint = if provider == LLMProvider::Ollama {
            match SettingsRepository::get_model_config(&pool).await {
                Ok(Some(config)) => config.ollama_endpoint,
                Ok(None) => None,
                Err(e) => {
                    info!("Failed to retrieve Ollama endpoint: {}, using default", e);
                    None
                }
            }
        } else {
            None
        };

        // Get CustomOpenAI config if provider is CustomOpenAI
        let (custom_openai_endpoint, custom_openai_api_key, custom_openai_max_tokens, custom_openai_temperature, custom_openai_top_p) =
            if provider == LLMProvider::CustomOpenAI {
                match SettingsRepository::get_custom_openai_config(&pool).await {
                    Ok(Some(config)) => {
                        info!("✓ Using custom OpenAI endpoint: {}", config.endpoint);
                        (
                            Some(config.endpoint),
                            config.api_key,
                            config.max_tokens.map(|t| t as u32),
                            config.temperature,
                            config.top_p,
                        )
                    }
                    Ok(None) => {
                        let err_msg = "Custom OpenAI provider selected but no configuration found";
                        Self::update_process_failed(&pool, &meeting_id, err_msg).await;
                        return;
                    }
                    Err(e) => {
                        let err_msg = format!("Failed to retrieve custom OpenAI config: {}", e);
                        Self::update_process_failed(&pool, &meeting_id, &err_msg).await;
                        return;
                    }
                }
            } else {
                (None, None, None, None, None)
            };

        // For CustomOpenAI, use its API key (if any) instead of the empty string
        let final_api_key = if provider == LLMProvider::CustomOpenAI {
            custom_openai_api_key.unwrap_or_default()
        } else {
            api_key
        };

        // Dynamically fetch context size based on provider and model
        let token_threshold = if provider == LLMProvider::Ollama {
            match METADATA_CACHE.get_or_fetch(&model_name, ollama_endpoint.as_deref()).await {
                Ok(metadata) => {
                    // Reserve 300 tokens for prompt overhead
                    let optimal = metadata.context_size.saturating_sub(300);
                    info!(
                        "✓ Using dynamic context for {}: {} tokens (chunk size: {})",
                        model_name, metadata.context_size, optimal
                    );
                    optimal
                }
                Err(e) => {
                    warn!(
                        "Failed to fetch context for {}: {}. Using default 4000",
                        model_name, e
                    );
                    4000  // Fallback to safe default
                }
            }
        } else {
            // Cloud providers (OpenAI, Claude, CustomOpenAI) handle large contexts automatically
            100000  // Effectively unlimited for single-pass processing
        };

        // Get app data directory
        let app_data_dir = _app.path().app_data_dir().ok();

        if let Some(code) = &summary_language {
            info!("📝 Summary language preference: {}", code);
        }

        let detected_summary_language =
            Self::read_detected_summary_language(&pool, &meeting_id)
                .await
                .or_else(|| Self::detect_summary_language_from_text(&text));

        if let Some(code) = &detected_summary_language {
            info!("📝 Detected transcript summary language: {}", code);
        }

        let template = match templates::get_template(&template_id) {
            Ok(template) => template,
            Err(e) => {
                let err_msg = format!("Failed to load template '{}': {}", template_id, e);
                Self::update_process_failed(&pool, &meeting_id, &err_msg).await;
                return;
            }
        };
        let template_fingerprint = template_cache_fingerprint(&template);

        // Merge command params with CustomOpenAI config (command params take precedence)
        let final_max_tokens = max_tokens.or(custom_openai_max_tokens);
        let final_temperature = temperature.or(custom_openai_temperature);
        let final_top_p = top_p.or(custom_openai_top_p);
        let final_top_k = top_k;

        let cache_source = build_summary_cache_source(
            &text,
            &custom_prompt,
            &template_id,
            &template_fingerprint,
            token_threshold,
            &model_provider,
            &model_name,
            ollama_endpoint.as_deref(),
            custom_openai_endpoint.as_deref(),
            final_max_tokens,
            final_temperature,
            final_top_p,
        );

        let cached_english = match SummaryProcessesRepository::get_summary_data(&pool, &meeting_id).await {
            Err(e) => {
                warn!(
                    "Failed to load prior summary row for cache lookup (meeting_id={}): {}. Falling back to full pass-1 generation.",
                    meeting_id, e
                );
                None
            }
            Ok(None) => None,
            Ok(Some(process)) => process.result.and_then(|raw| {
                match extract_cached_english_markdown(
                    &raw,
                    &cache_source,
                    summary_language.as_deref(),
                ) {
                    Ok(opt) => opt,
                    Err(e) => {
                        warn!(
                            "Cached summary result for meeting_id={} is not valid JSON ({}); ignoring cache.",
                            meeting_id, e
                        );
                        None
                    }
                }
            }),
        };

        // Create streaming callback for LocalLLM provider
        let progress_state = Arc::new(Mutex::new(InferenceProgressState::new()));
        let streaming_callback = if provider == LLMProvider::LocalLLM {
            let app_clone = _app.clone();
            let meeting_id_clone = meeting_id.clone();
            let progress_clone = progress_state.clone();
            Some(std::sync::Arc::new(move |token: &str| {
                let meeting_id_str = meeting_id_clone.as_str();
                // Emit streaming token event
                let _ = app_clone.emit("summary-token-stream", serde_json::json!({
                    "token": token,
                    "meeting_id": meeting_id_clone,
                }));
                // Track and emit inference progress metrics
                let mut state = progress_clone.lock().unwrap();
                state.record_token();
                if state.should_emit_progress() {
                    let elapsed = state.start_time.elapsed().as_secs_f32();
                    let tokens_per_second = state.tokens_generated as f32 / elapsed.max(0.001);
                    let estimated_remaining = if tokens_per_second > 0.0 {
                        // Estimate based on typical summary length of 2000 tokens
                        let remaining_tokens = (2000.0_f32 - state.tokens_generated as f32).max(0.0);
                        Some(remaining_tokens / tokens_per_second)
                    } else {
                        None
                    };
                    let _ = app_clone.emit("llm-inference-progress", serde_json::json!({
                        "tokens_generated": state.tokens_generated,
                        "tokens_per_second": (tokens_per_second * 10.0).round() / 10.0,
                        "estimated_remaining_seconds": estimated_remaining.map(|v: f32| (v * 10.0).round() / 10.0),
                        "meeting_id": meeting_id_str,
                    }));
                    state.last_emit_time = Some(Instant::now());
                }
            }) as std::sync::Arc<dyn Fn(&str) + Send + Sync>)
        } else {
            None
        };
        let client = &*HTTP_CLIENT;
        let result = generate_meeting_summary(
            &client,
            &provider,
            &model_name,
            &final_api_key,
            &text,
            &custom_prompt,
            &template_id,
            &template,
            token_threshold,
            ollama_endpoint.as_deref(),
            custom_openai_endpoint.as_deref(),
            final_max_tokens,
            final_temperature,
            final_top_p,
            final_top_k,
            app_data_dir.as_ref(),
            Some(&cancellation_token),
            summary_language.as_deref(),
            detected_summary_language.as_deref(),
            cached_english.as_deref(),
            streaming_callback,
        )
        .await;

        let duration = start_time.elapsed().as_secs_f64();

        // Emit inference complete event for LocalLLM with final metrics
        if provider == LLMProvider::LocalLLM {
            let state = progress_state.lock().unwrap();
            let elapsed = state.start_time.elapsed().as_secs_f32();
            let tokens_per_second = state.tokens_generated as f32 / elapsed.max(0.001);
            let _ = _app.emit("llm-inference-complete", serde_json::json!({
                "tokens_generated": state.tokens_generated,
                "tokens_per_second": (tokens_per_second * 10.0).round() / 10.0,
                "total_duration_seconds": (duration * 10.0).round() / 10.0,
                "meeting_id": meeting_id,
            }));
        }

        // Clean up cancellation token regardless of outcome
        Self::cleanup_cancellation_token(&meeting_id);

        match result {
            Ok((final_markdown, english_markdown, num_chunks)) => {
                info!(
                    "✓ Successfully processed {} chunks for meeting_id: {}. Duration: {:.2}s",
                    num_chunks, meeting_id, duration
                );
                info!("Final markdown generated ({} chars)", final_markdown.len());

                if let Some(name) = extract_meeting_name_from_markdown(&final_markdown)
                    .filter(|n| !n.is_empty())
                {
                    info!("Extracted meeting name from summary: '{}'", name);
                    if let Err(e) =
                        MeetingsRepository::update_meeting_name(&pool, &meeting_id, &name).await
                    {
                        error!("Failed to update meeting name for {}: {}", meeting_id, e);
                    } else {
                        info!("Successfully updated meeting name for {}", meeting_id);
                    }
                }

                // Score summary quality
                let quality_score = Self::score_summary_quality(&final_markdown, &text);
                info!("Summary quality score for meeting {}: {}/5", meeting_id, quality_score);

                let result_json = build_summary_result_json(
                    &final_markdown,
                    &english_markdown,
                    cache_source,
                    summary_language.as_deref(),
                    quality_score,
                );

                // Update database with completed status
                if let Err(e) = SummaryProcessesRepository::update_process_completed(
                    &pool,
                    &meeting_id,
                    result_json,
                    num_chunks,
                    duration,
                )
                .await
                {
                    error!(
                        "Failed to save completed process for {}: {}",
                        meeting_id, e
                    );
                } else {
                    info!(
                        "Summary saved successfully for meeting_id: {}",
                        meeting_id
                    );
                }
            }
            Err(e) => {
                // Check if error is due to cancellation
                if e.contains("cancelled") {
                    info!("Summary generation was cancelled for meeting_id: {}", meeting_id);
                    if let Err(db_err) = SummaryProcessesRepository::update_process_cancelled(&pool, &meeting_id).await {
                        error!("Failed to update DB status to cancelled for {}: {}", meeting_id, db_err);
                    }
                } else {
                    Self::update_process_failed(&pool, &meeting_id, &e).await;
                }
            }
        }
    }

    /// Updates the summary process status to failed with error message
    ///
    /// # Arguments
    /// * `pool` - SQLx connection pool
    /// * `meeting_id` - Meeting identifier
    /// * `error_msg` - Error message to store
    async fn update_process_failed(pool: &SqlitePool, meeting_id: &str, error_msg: &str) {
        error!(
            "Processing failed for meeting_id {}: {}",
            meeting_id, error_msg
        );
        if let Err(e) =
            SummaryProcessesRepository::update_process_failed(pool, meeting_id, error_msg).await
        {
            error!(
                "Failed to update DB status to failed for {}: {}",
                meeting_id, e
            );
        }
    }

    /// Regenerates summary using cached transcript chunks with new generation parameters
    pub async fn regenerate_summary<R: tauri::Runtime>(
        _app: AppHandle<R>,
        pool: SqlitePool,
        meeting_id: String,
        model_provider: String,
        model_name: String,
        custom_prompt: String,
        template_id: String,
        summary_language: Option<String>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
        top_p: Option<f32>,
        top_k: Option<i32>,
    ) -> Result<(), String> {
        info!(
            "Regenerating summary for meeting_id: {} with new params",
            meeting_id
        );

        // Load cached transcript text
        let text = crate::database::repositories::transcript_chunk::TranscriptChunksRepository::get_transcript_text(&pool, &meeting_id)
            .await
            .map_err(|e| format!("Failed to load transcript chunks: {}", e))?
            .ok_or_else(|| "No transcript chunks found for this meeting. Please generate a summary first.".to_string())?;

        // Reset process state
        SummaryProcessesRepository::create_or_reset_process(&pool, &meeting_id)
            .await
            .map_err(|e| format!("Failed to reset process: {}", e))?;

        // Spawn background task for regeneration
        let app_clone = _app.clone();
        let meeting_id_clone = meeting_id.clone();
        tauri::async_runtime::spawn(async move {
            Self::process_transcript_background(
                app_clone,
                pool,
                meeting_id_clone,
                text,
                model_provider,
                model_name,
                custom_prompt,
                template_id,
                summary_language,
                max_tokens,
                temperature,
                top_p,
                top_k,
            )
            .await;
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_leading_title_with_body() {
        let input = "# Meeting Title\nThis is the body.\nMore content.";
        let result = strip_leading_title(input);
        assert_eq!(result, "This is the body.\nMore content.");
    }

    #[test]
    fn test_strip_leading_title_only() {
        let input = "# Meeting Title";
        let result = strip_leading_title(input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_strip_leading_title_no_heading() {
        let input = "No heading here.\nJust body.";
        let result = strip_leading_title(input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_strip_leading_title_multiline_body() {
        let input = "# Title\n## Subheading\nParagraph 1\n\nParagraph 2";
        let result = strip_leading_title(input);
        assert_eq!(result, "## Subheading\nParagraph 1\n\nParagraph 2");
    }

    #[test]
    fn test_strip_leading_title_empty_after_heading() {
        let input = "# Title\n";
        let result = strip_leading_title(input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_strip_leading_title_whitespace_after_heading() {
        let input = "# Title\n   \n Body with leading spaces";
        let result = strip_leading_title(input);
        assert_eq!(result, "Body with leading spaces");
    }

    #[test]
    fn test_strip_title_if_present_preserves_already_stripped() {
        assert_eq!(strip_title_if_present("## Action Items\nfoo"), "## Action Items\nfoo");
    }

    #[test]
    fn test_strip_title_if_present_strips_leading_h1() {
        assert_eq!(strip_title_if_present("# Meeting Title\n## Action Items\nfoo"), "## Action Items\nfoo");
    }

    #[test]
    fn test_strip_title_if_present_no_heading_preserved() {
        // Distinct from strip_leading_title which returns "" — this preserves input.
        assert_eq!(strip_title_if_present("Just body text"), "Just body text");
    }

    #[test]
    fn test_strip_title_if_present_hash_no_space_preserved() {
        // `#NoSpace` is not a markdown H1 — preserve.
        assert_eq!(strip_title_if_present("#NoSpace\nbody"), "#NoSpace\nbody");
    }

    #[test]
    fn test_strip_title_if_present_mid_document_h1_preserved() {
        // H1 after body content must NOT be stripped — guards the asymmetry where
        // extract_meeting_name_from_markdown scans every line for "# ".
        let input = "Some paragraph\n\n# H1 on line 3\n## Section\nbody";
        assert_eq!(strip_title_if_present(input), input);
    }

    #[test]
    fn test_strip_title_if_present_leading_whitespace_h1_stripped() {
        assert_eq!(
            strip_title_if_present("  # Title\n## Section\nbody"),
            "## Section\nbody"
        );
    }

    fn sample_cache_source() -> SummaryCacheSource {
        let template_fingerprint = stable_text_fingerprint("standard template prompt");
        build_summary_cache_source(
            "transcript body",
            "custom prompt",
            "standard_meeting",
            &template_fingerprint,
            3700,
            "ollama",
            "gemma3:1b",
            Some("http://localhost:11434"),
            None,
            None,
            None,
            None,
        )
    }

    fn test_template(section_title: &str) -> Template {
        Template {
            name: "Test".to_string(),
            description: "Test template".to_string(),
            sections: vec![crate::summary::templates::TemplateSection {
                title: section_title.to_string(),
                instruction: "Summarize this section".to_string(),
                format: "paragraph".to_string(),
                item_format: None,
                example_item_format: None,
            }],
        }
    }

    #[test]
    fn test_template_cache_fingerprint_changes_with_rendered_template() {
        assert_ne!(
            template_cache_fingerprint(&test_template("Summary")),
            template_cache_fingerprint(&test_template("Decisions"))
        );
    }

    #[test]
    fn test_legacy_english_markdown_field_is_cache_miss() {
        let raw = serde_json::json!({
            "markdown": "translated",
            "english_markdown": "# Old English\nBody"
        })
        .to_string();

        assert_eq!(
            extract_cached_english_markdown(&raw, &sample_cache_source(), Some("de")).unwrap(),
            None
        );
    }

    #[test]
    fn test_matching_source_changed_translation_target_reuses_cache() {
        let source = sample_cache_source();
        let raw = build_summary_result_json(
            "# Reunion\n## Points\nBonjour",
            "# Meeting\n## Points\nHello",
            source.clone(),
            Some("fr"),
            4,
        )
        .to_string();

        assert_eq!(
            extract_cached_english_markdown(&raw, &source, Some("de")).unwrap(),
            Some("# Meeting\n## Points\nHello".to_string())
        );
    }

    #[test]
    fn test_same_language_regeneration_rejects_cache() {
        let source = sample_cache_source();
        let raw = build_summary_result_json(
            "# Reunion\n## Points\nBonjour",
            "# Meeting\n## Points\nHello",
            source.clone(),
            Some("fr"),
            4,
        )
        .to_string();

        assert_eq!(
            extract_cached_english_markdown(&raw, &source, Some("fr")).unwrap(),
            None
        );
    }

    #[test]
    fn test_changed_summary_inputs_reject_cache() {
        let source = sample_cache_source();
        let template_fingerprint = source.template_fingerprint.clone();
        let raw = build_summary_result_json(
            "# Reunion\n## Points\nBonjour",
            "# Meeting\n## Points\nHello",
            source,
            Some("fr"),
            4,
        )
        .to_string();

        let changed_sources = [
            build_summary_cache_source(
                "changed transcript",
                "custom prompt",
                "standard_meeting",
                &template_fingerprint,
                3700,
                "ollama",
                "gemma3:1b",
                Some("http://localhost:11434"),
                None,
                None,
                None,
                None,
            ),
            build_summary_cache_source(
                "transcript body",
                "changed prompt",
                "standard_meeting",
                &template_fingerprint,
                3700,
                "ollama",
                "gemma3:1b",
                Some("http://localhost:11434"),
                None,
                None,
                None,
                None,
            ),
            build_summary_cache_source(
                "transcript body",
                "custom prompt",
                "daily_standup",
                &template_fingerprint,
                3700,
                "ollama",
                "gemma3:1b",
                Some("http://localhost:11434"),
                None,
                None,
                None,
                None,
            ),
            build_summary_cache_source(
                "transcript body",
                "custom prompt",
                "standard_meeting",
                &template_fingerprint,
                3700,
                "openai",
                "gemma3:1b",
                Some("http://localhost:11434"),
                None,
                None,
                None,
                None,
            ),
            build_summary_cache_source(
                "transcript body",
                "custom prompt",
                "standard_meeting",
                &template_fingerprint,
                3700,
                "ollama",
                "qwen2.5:3b",
                Some("http://localhost:11434"),
                None,
                None,
                None,
                None,
            ),
            build_summary_cache_source(
                "transcript body",
                "custom prompt",
                "standard_meeting",
                &template_fingerprint,
                3700,
                "ollama",
                "gemma3:1b",
                Some("http://localhost:11500"),
                None,
                None,
                None,
                None,
            ),
            build_summary_cache_source(
                "transcript body",
                "custom prompt",
                "standard_meeting",
                &template_fingerprint,
                3700,
                "ollama",
                "gemma3:1b",
                Some("http://localhost:11434"),
                Some("https://custom.example/v1"),
                Some(2048),
                Some(0.2),
                Some(0.9),
            ),
        ];

        for changed_source in changed_sources {
            assert_eq!(
                extract_cached_english_markdown(&raw, &changed_source, Some("de")).unwrap(),
                None
            );
        }
    }

    #[test]
    fn test_changed_template_content_rejects_cache() {
        let source = sample_cache_source();
        let raw = build_summary_result_json(
            "# Reunion\n## Points\nBonjour",
            "# Meeting\n## Points\nHello",
            source.clone(),
            Some("fr"),
            4,
        )
        .to_string();

        let changed_template = SummaryCacheSource {
            template_fingerprint: stable_text_fingerprint("changed template prompt"),
            ..source
        };

        assert_eq!(
            extract_cached_english_markdown(&raw, &changed_template, Some("de")).unwrap(),
            None
        );
    }

    #[test]
    fn test_changed_token_threshold_rejects_cache() {
        let source = sample_cache_source();
        let raw = build_summary_result_json(
            "# Reunion\n## Points\nBonjour",
            "# Meeting\n## Points\nHello",
            source.clone(),
            Some("fr"),
            4,
        )
        .to_string();

        let changed_threshold = SummaryCacheSource {
            token_threshold: 8192,
            ..source
        };

        assert_eq!(
            extract_cached_english_markdown(&raw, &changed_threshold, Some("de")).unwrap(),
            None
        );
    }

    #[test]
    fn test_result_json_strips_display_markdown_but_keeps_cache_title() {
        let result = build_summary_result_json(
            "# Translated Title\n## Decisions\nDone",
            "# English Title\n## Decisions\nDone",
            sample_cache_source(),
            Some("fr"),
            4,
        );

        assert_eq!(result["markdown"], "## Decisions\nDone");
        assert_eq!(
            result["english_cache"]["markdown"],
            "# English Title\n## Decisions\nDone"
        );
    }

    #[test]
    fn test_extract_cached_english_from_malformed_json_errors() {
        let raw = r#"{ not valid json"#;
        assert!(extract_cached_english_markdown(raw, &sample_cache_source(), Some("de")).is_err());
    }
}
