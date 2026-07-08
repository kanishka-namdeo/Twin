# Console Utils Module

## Purpose

Developer console utilities for debugging and runtime inspection. Provides Tauri commands for toggling console output and viewing internal state.

## Ownership

This module owns:
- Console/logging utility functions
- Tauri commands for console control

## Local Contracts

### Tauri Commands

| Command | Purpose |
|---------|---------|
| Console toggle/state commands (see `commands.rs`) | Control debug console visibility |

### Deterministic Core

- Console attach/detach on Windows (AllocConsole/FreeConsole)

### Non-Deterministic Edges

- Platform-specific console behavior

## Work Guidance

### Integration Points

- Registered in `lib.rs` as Tauri commands
- Used for debugging in development builds

## Verification

```bash
# Build check
cargo check --manifest-path frontend/src-tauri/Cargo.toml
```

## Child DOX Index

This module has no child docs.
