# OpenAI Module

## Purpose

Minimal OpenAI-specific utilities. The primary OpenAI-compatible chat completion path lives in `summary/llm_client.rs` (used for OpenAI, Ollama, and Custom OpenAI providers). This module contains any OpenAI-specific helpers that don't fit the generic path.

## Ownership

This module owns:
- OpenAI-specific data structures (if any beyond the generic ones in `summary/llm_client.rs`)

## Local Contracts

### Provider Usage

OpenAI is accessed via the generic OpenAI-compatible chat completion endpoint in `summary/llm_client.rs`:
- URL: `https://api.openai.com/v1/chat/completions`
- Auth: Bearer token from `settings` table

### Deterministic Core

- API URL and auth header format

### Non-Deterministic Edges

- API response format variations

## Work Guidance

### Integration Points

- `summary/llm_client.rs` handles the actual HTTP requests for OpenAI provider
- API key stored in `settings.openaiApiKey` column via `database/repositories/setting.rs`

## Verification

```bash
# Build check
cargo check --manifest-path frontend/src-tauri/Cargo.toml
```

## Child DOX Index

This module has no child docs.
