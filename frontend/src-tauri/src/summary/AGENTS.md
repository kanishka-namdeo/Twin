# Summary Module

## Purpose

Meeting summary generation pipeline — chunks transcripts, calls LLM providers (OpenAI, Claude, Ollama, Custom OpenAI, Local LLM), processes responses into structured markdown, supports templates, language detection, caching, and cancellation.

## Ownership

This module owns:
- Summary generation orchestration (chunking, LLM calls, response parsing)
- LLM provider abstraction and HTTP client
- Summary templates (standard_meeting, daily_standup, retrospective, etc.)
- Summary language detection and translation
- Cancellation token management for in-flight generations
- English summary caching for language switching
- Tauri commands for summary lifecycle and template management

## Local Contracts

### LLM Providers

| Provider | Enum | Endpoint | Auth |
|----------|------|----------|------|
| OpenAI | `OpenAI` | `api.openai.com/v1` | Bearer token |
| Claude | `Claude` | `api.anthropic.com/v1` | x-api-key header |
| Ollama | `Ollama` | configurable (default `localhost:11434`) | None |
| Custom OpenAI | `CustomOpenAI` | user-configured endpoint | Bearer token (optional) |
| Local LLM | `LocalLLM` | N/A (in-process via llama-cpp-4) | None |

### Summary Generation State Model

States:
- Idle: no generation in progress
- Initializing: process row created, loading config
- Generating: LLM calls in progress (chunks being processed)
- Completed: markdown saved to database
- Failed: error occurred, backup restored
- Cancelled: user cancelled, backup restored

Transitions:
- Idle -> Initializing: on `api_process_transcript` command
- Initializing -> Generating: on background task spawn
- Generating -> Completed: on successful markdown save
- Generating -> Failed: on LLM or processing error
- Generating -> Cancelled: on cancellation token trigger
- Any -> Idle: after terminal state persisted

Guards:
- Cannot start new generation while one is in progress for same meeting
- Cancellation only works during Generating state
- On failure/cancel, previous summary backup is restored

### Template System

Templates define structured meeting summary formats:

| Template ID | Use Case |
|-------------|----------|
| `standard_meeting` | General meetings |
| `daily_standup` | Standup/scrum meetings |
| `project_sync` | Project status syncs |
| `retrospective` | Retrospective meetings |
| `sales_marketing_client_call` | Sales/client calls |
| `psychatric_session` | Therapy/clinical sessions |

Template files stored as JSON in `target/debug/templates/` (dev) or app data dir (prod).

### Caching

English summaries are cached alongside translated output. When the user switches output language but the source inputs are unchanged, the cached English markdown is reused instead of re-calling the LLM. Cache key includes: transcript fingerprint, custom prompt, template ID/content, token threshold, model provider/name, endpoints, and generation parameters.

### Tauri Commands

| Command | Purpose |
|---------|---------|
| `api_process_transcript` | Start summary generation (spawns background task) |
| `api_get_summary` | Get summary status and data |
| `api_save_meeting_summary` | Save summary manually |
| `api_cancel_summary` | Cancel in-progress generation |
| `api_get_meeting_summary_language` | Get per-meeting language override |
| `api_save_meeting_summary_language` | Set per-meeting language override |
| `api_get_meeting_detected_summary_language` | Get auto-detected language |
| `api_save_meeting_detected_summary_language` | Cache detected language |
| `api_detect_transcript_summary_language` | Detect language from text |
| `api_list_templates` | List available templates |
| `api_get_template_details` | Get template by ID |
| `api_validate_template` | Validate template JSON |

### Deterministic Core

- Provider URL construction and auth header format
- Chunk splitting by token threshold
- Cancellation token registry (global `HashMap<String, CancellationToken>`)
- Cache fingerprint computation (FNV-1a hash)
- Template loading and validation

### Non-Deterministic Edges

- LLM response quality and formatting
- Language detection heuristics
- Meeting name extraction from markdown
- Markdown cleanup (LLM output post-processing)

## Work Guidance

### Cancellation

Each summary generation registers a `CancellationToken` in a global registry keyed by meeting ID. The token is checked before HTTP requests and during chunk processing loops. Always clean up the token after completion.

### Background Processing

Summary generation runs in a spawned async task via `tauri::async_runtime::spawn`. The command returns immediately with a `process_id`. Progress is tracked via the `summary_processes` database table.

### Ollama Context Size

For Ollama, the context window is dynamically fetched from the model's metadata (with 5-minute TTL cache) and reduced by 300 tokens for prompt overhead. Other providers use 100k token threshold.

### Integration Points

- Uses `database/repositories/summary.rs` for process state persistence
- Uses `database/repositories/setting.rs` for API keys and config
- Uses `llm_engine/` for LocalLLM provider (in-process inference)
- Uses `ollama/metadata.rs` for context size lookup
- Frontend polls `api_get_summary` for status updates

## Verification

```bash
# Build check
cargo check --manifest-path frontend/src-tauri/Cargo.toml

# Run unit tests
cargo test --manifest-path frontend/src-tauri/Cargo.toml -- summary

# Run with debug logging
RUST_LOG=app_lib::summary=debug ./clean_run.sh
```

## Child DOX Index

This module has no child docs.
