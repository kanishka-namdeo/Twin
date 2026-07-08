# App Lifecycle

## Purpose

Manages application lifecycle, resource cleanup, graceful shutdown, and signal handling for the Tauri desktop application.

## Ownership

This module owns:
- Application startup and initialization
- Resource allocation and deallocation
- Graceful shutdown procedures
- Signal handling (SIGTERM, SIGINT)
- Cleanup of audio, transcription, and LLM resources

## Local Contracts

### Shutdown State Model

States:
- Running: all systems active
- SignalReceived: shutdown signal detected
- DrainingAudio: stopping capture, flushing buffers
- DrainingTranscription: flushing pending segments
- ReleasingGPU: freeing model memory and GPU contexts
- ClosingDB: persisting final state, closing connections
- Exited: process terminated

Transitions:
- Running -> SignalReceived: on SIGTERM/SIGINT/window close
- SignalReceived -> DrainingAudio: immediately
- DrainingAudio -> DrainingTranscription: on audio stopped
- DrainingTranscription -> ReleasingGPU: on transcription flushed
- ReleasingGPU -> ClosingDB: on GPU resources freed
- ClosingDB -> Exited: on connections closed

Guards (illegal transitions):
- Cannot skip DrainingAudio (data loss risk)
- Cannot release GPU while transcription is draining
- Cannot close DB while audio is still capturing

### Deterministic Core

- Shutdown ordering (enforced by guards above)
- Resource deallocation sequence
- Signal handling registration
- RAII and Drop trait enforcement

### Non-Deterministic Edges

- Timeout durations for graceful drain
- Logging verbosity during shutdown
- Recovery strategy if a drain step hangs

### Resource Management

- Use RAII patterns for automatic cleanup
- Implement `Drop` trait for critical resources
- Log resource allocation/deallocation for debugging

## Work Guidance

### Signal Handling

```rust
// Handle SIGTERM/SIGINT for graceful shutdown
signal_handler::setup(app_handle)?;
```

### Graceful Shutdown

```rust
// Coordinate shutdown across modules
shutdown::graceful_shutdown(app_handle).await?;
```

### Integration Points

- Coordinates with `audio` module to stop capture
- Coordinates with `whisper_engine` to flush transcriptions
- Manages Tauri app handle lifecycle

## Verification

```bash
# Build check
cargo check --manifest-path frontend/src-tauri/Cargo.toml

# Test graceful shutdown
# Run app, then Ctrl+C or close window
```

## Child DOX Index

This module has no child docs.
