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

### Transcription Pipeline State Model

States:
- Unloaded: no model in memory
- Loading: model file read, initializing context
- Ready: model loaded, awaiting audio
- Transcribing: processing audio chunk
- ParallelProcessing: multiple chunks in flight
- Flushing: draining pending segments
- Error: transcription failed

Transitions:
- Unloaded -> Loading: on model load request
- Loading -> Ready: on context initialized
- Ready -> Transcribing: on audio chunk received
- Transcribing -> Ready: on chunk complete
- Transcribing -> ParallelProcessing: on multi-chunk batch
- ParallelProcessing -> Ready: on all chunks complete
- Ready -> Flushing: on shutdown/stop signal
- Flushing -> Ready: on pending segments emitted
- Any -> Error: on processing failure
- Error -> Unloaded: on model reset

Guards:
- Cannot transcribe without model in Ready state
- Cannot unload model while Transcribing or ParallelProcessing
- Cannot enter ParallelProcessing from Unloaded or Loading

### Deterministic Core

- Model loading and caching (path resolution, memory management)
- Chunk scheduling and parallel processing orchestration
- State transitions (enforced by guards above)
- GPU capability detection and feature selection

### Non-Deterministic Edges

- Transcription accuracy and language detection
- Chunk sizing and batching heuristics
- Error recovery strategy

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
