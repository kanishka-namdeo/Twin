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

### Shutdown Sequence

1. Receive shutdown signal (user action or system signal)
2. Stop audio capture and recording
3. Flush pending transcriptions
4. Release GPU resources
5. Close database connections
6. Exit application

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
