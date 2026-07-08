# Ollama Module

## Purpose

Integration with locally-running Ollama instances for LLM-based meeting summarization. Handles model discovery, metadata fetching (context size), and provides the Ollama-specific LLM client path used by the summary module.

## Ownership

This module owns:
- Querying Ollama for available models (via HTTP API)
- Fetching and caching Ollama model metadata (context window size)
- Ollama-specific Tauri commands for model listing and endpoint configuration

## Local Contracts

### Default Endpoint

`http://localhost:11434` (configurable via settings table in database)

### Key Types

| Type | Purpose |
|------|---------|
| `ModelMetadataCache` | TTL-based cache (5 min) for model context sizes |

### Tauri Commands

| Command | Purpose |
|---------|---------|
| `get_ollama_models` | List available models from Ollama instance |

### Deterministic Core

- HTTP request format to Ollama API (`/api/tags`, `/api/show`)
- Cache TTL and key structure
- Endpoint URL construction

### Non-Deterministic Edges

- Network availability and Ollama server status
- Model metadata response format variations

## Work Guidance

### Context Size

The summary module uses `ModelMetadataCache` to dynamically determine chunk sizes based on the Ollama model's context window, reserving 300 tokens for prompt overhead.

### Integration Points

- `summary/llm_client.rs` sends chat completion requests to Ollama's OpenAI-compatible `/v1/chat/completions` endpoint
- `summary/service.rs` uses metadata cache for token threshold calculation
- Frontend calls `get_ollama_models` to populate model selection dropdown

## Verification

```bash
# Build check
cargo check --manifest-path frontend/src-tauri/Cargo.toml

# Verify Ollama connectivity (requires running Ollama)
curl http://localhost:11434/api/tags
```

## Child DOX Index

This module has no child docs.
