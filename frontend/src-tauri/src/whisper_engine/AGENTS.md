# Whisper Engine

## Purpose

Manages Whisper model loading, transcription processing, parallel processing, and integration with the audio pipeline for meeting transcription.

## Ownership

This module owns:
- Whisper model loading and caching
- Transcription request handling
- Parallel processing of audio chunks
- Model selection and switching
- GPU acceleration (Metal, CUDA, Vulkan)

## Local Contracts

### Model Locations

| Environment | Path |
|-------------|------|
| Development | `frontend/models/` |
| macOS Prod | `~/Library/Application Support/Twin/models/` |
| Windows Prod | `%APPDATA%\Twin\models\` |

### Model Selection

| Use Case | Model |
|----------|-------|
| Development | `base` or `small` |
| Production | `medium` or `large-v3` |

### Key Constraints

- Models loaded once and cached in memory
- Model change requires restart or manual unload/reload
- No separate backend — transcription handled entirely in Tauri app
- GPU acceleration is platform-specific and auto-detected

## Work Guidance

### GPU Acceleration

- macOS: Metal + CoreML (auto-enabled)
- Windows/Linux: CUDA/Vulkan via Cargo features

### Parallel Processing

- `parallel_processor.rs`: Handles concurrent transcription of audio chunks
- Uses thread pools for efficient processing
- Respects system resource constraints

### Integration Points

- Receives VAD-filtered audio from `audio` module
- Emits transcription events to frontend via Tauri events
- Coordinates with `lifecycle` module for graceful shutdown

## Verification

```bash
# Build check
cargo check --manifest-path frontend/src-tauri/Cargo.toml

# Run with debug logging
RUST_LOG=app_lib::whisper_engine=debug ./clean_run.sh
```

## Child DOX Index

This module has no child docs.
