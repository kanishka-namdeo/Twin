# LLM Engine Module

## Purpose
Provides embedded local LLM inference using llama-cpp-4, allowing users to run GGUF models (Qwen 2.5, Llama 3, Gemma 2) without installing Ollama or any external software.

## Ownership
This module owns:
- Local LLM model management (download, validate, delete)
- Model loading and inference via llama-cpp-4
- GGUF model catalog and metadata
- GPU acceleration detection and configuration
- Integration with the summary service for local inference

## Local Contracts

### Model Storage
Models are stored in `app_data_dir/llm_models/`:
- macOS: `~/Library/Application Support/Twin/llm_models/`
- Windows: `%APPDATA%\Twin\llm_models\`
- Linux: `~/.config/Twin/llm_models/`

### Model Catalog
Pre-configured models available for download:
- **Qwen 2.5 1.5B** (Recommended): Fast, good quality, low resource usage (~986 MB)
- **Llama 3.2 3B**: Balanced performance and quality (~1.9 GB)
- **Gemma 2 2B**: Google's efficient model (~1.4 GB)

### Tauri Commands
| Command | Purpose |
|---------|---------|
| `llm_init` | Initialize LLM engine |
| `llm_get_available_models` | List all models with status |
| `llm_download_model` | Download a model with progress |
| `llm_delete_model` | Delete a downloaded model |
| `llm_load_model` | Load a model into memory |
| `llm_unload_model` | Unload model and free resources |
| `llm_is_model_loaded` | Check if model is loaded |
| `llm_get_current_model` | Get currently loaded model name |
| `llm_get_models_directory` | Get models directory path |
| `llm_get_gpu_info` | Get GPU acceleration info |
| `open_llm_models_folder` | Open models folder in file explorer |

### Tauri Events
| Event | Payload | When |
|-------|---------|------|
| `llm-model-download-progress` | `{ modelName, progress, downloaded_bytes, total_bytes, downloaded_mb, total_mb, speed_mbps, status }` | During download (every 500ms) |
| `llm-model-download-complete` | `{ modelName }` | Download finished |
| `llm-model-download-error` | `{ modelName, error }` | Download failed |
| `llm-model-loading-started` | `{ modelName }` | Before model load |
| `llm-model-loading-completed` | `{ modelName }` | After successful load |
| `llm-model-loading-failed` | `{ modelName, error }` | Load failure |

### Model Validation
Models are validated by checking:
1. GGUF magic number (first 4 bytes: `0x47, 0x47, 0x55, 0x46`)
2. File size matches expected size (within tolerance)

### GPU Acceleration
GPU acceleration is automatically detected and enabled based on platform:
- **macOS**: Metal (auto-enabled via Cargo feature)
- **Windows/Linux**: CUDA or Vulkan (user-selectable via Cargo features)
- **Fallback**: CPU-only mode

GPU info available via `llm_get_gpu_info` command.

### Integration with Summary Service
The summary service (`summary/llm_client.rs`) supports `LocalLLM` provider:
- Provider string: `"local-llm"`
- Model name: filename from catalog (e.g., `"qwen2.5-1.5b-instruct-q4_k_m.gguf"`)
- No API key required
- Inference handled directly by llama-cpp-4 (no HTTP calls)

## Work Guidance

### Model Download
Downloads use streaming with progress tracking:
- Progress emitted every 500ms via Tauri events
- Temporary file (`.tmp` extension) during download
- Atomic rename after validation
- Concurrent download protection via `active_downloads` set

### Memory Management
- Models loaded into memory on demand via `llm_load_model`
- Only one model can be loaded at a time
- Previous model automatically unloaded when loading new model
- Explicit unload via `llm_unload_model` to free resources

### Inference Configuration
Default generation parameters:
- `max_tokens`: 2048
- `temperature`: 0.7
- `top_p`: 0.9
- `top_k`: 40
- `repeat_penalty`: 1.1

### Error Handling
- Model not found: clear error message with model name
- Download failures: automatic retry via UI, error event emitted
- Validation failures: model marked as corrupted, can be deleted and re-downloaded
- Inference errors: returned to summary service, displayed to user

### Testing
```bash
# Build check
cargo check --manifest-path frontend/src-tauri/Cargo.toml

# Run with debug logging
RUST_LOG=app_lib::llm_engine=debug ./clean_run.sh

# Test model download
# Use UI to download a model, verify progress events

# Test inference
# Select LocalLLM provider in settings, generate summary
```

## Verification
```bash
# Build check
cargo check --manifest-path frontend/src-tauri/Cargo.toml

# Verify model catalog
# Check that 3 models are listed in config.rs

# Verify GPU detection
# Call llm_get_gpu_info command, check backend matches platform

# Verify download flow
# Download model via UI, check file exists in llm_models/

# Verify inference
# Load model, select LocalLLM provider, generate summary
```

## Child DOX Index
This module has no child docs.
