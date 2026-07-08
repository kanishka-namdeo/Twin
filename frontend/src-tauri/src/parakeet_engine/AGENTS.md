# Parakeet Engine

## Purpose

High-performance speech-to-text transcription using NVIDIA NeMo Parakeet models via ONNX Runtime. Provides a faster alternative to Whisper for real-time transcription.

## Ownership

This module owns:
- Parakeet ONNX model loading and inference
- Int8 quantization support for reduced memory footprint
- Model download and management
- Tauri commands for model lifecycle and transcription

## Local Contracts

### Model Storage

Models stored in `app_data_dir/parakeet_models/` (platform-specific via Tauri path APIs).

### Key Types

| Type | Purpose |
|------|---------|
| `ParakeetEngine` | Main engine: model loading, transcription, download |
| `ParakeetModel` | ONNX model wrapper and inference |
| `QuantizationType` | Int8 or Float32 model variant |
| `TimestampedResult` | Transcription output with word-level timestamps |
| `ModelInfo` | Model metadata (name, size, quantization) |
| `ModelStatus` | Download/load state |
| `DownloadProgress` | Progress events during model download |

### Transcription Pipeline State Model

States:
- Unloaded: no model in memory
- Loading: ONNX model being loaded
- Ready: model loaded, awaiting audio
- Transcribing: processing audio
- Error: transcription failed

Transitions:
- Unloaded -> Loading: on model load request
- Loading -> Ready: on ONNX session created
- Ready -> Transcribing: on audio input
- Transcribing -> Ready: on transcription complete
- Any -> Error: on processing failure
- Error -> Unloaded: on model reset

Guards:
- Cannot transcribe without model in Ready state
- Cannot unload model while Transcribing

### Deterministic Core

- ONNX model path resolution and validation
- Quantization type selection
- Timestamp extraction from model output

### Non-Deterministic Edges

- Transcription accuracy
- Model download speed
- Memory usage during inference

## Work Guidance

### Performance

- Int8 quantized models recommended for production (lower memory, comparable accuracy)
- ONNX Runtime auto-selects execution provider (CUDA, DirectML, CPU)

### Integration Points

- Called by `audio/transcription/parakeet_provider.rs` for transcription
- Tauri commands registered in `lib.rs`
- Complementary to `whisper_engine/` module (alternative STT backend)

## Verification

```bash
# Build check
cargo check --manifest-path frontend/src-tauri/Cargo.toml

# Run with debug logging
RUST_LOG=app_lib::parakeet_engine=debug ./clean_run.sh
```

## Child DOX Index

This module has no child docs.
