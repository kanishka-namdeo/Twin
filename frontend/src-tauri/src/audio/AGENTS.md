# Audio Pipeline

## Purpose

Manages audio device detection, capture, mixing, voice activity detection (VAD), and recording orchestration for meeting transcription and recording.

## Ownership

This module owns:
- Device discovery and selection (microphone/system)
- Audio capture from input devices
- Audio mixing for recording output
- VAD filtering for transcription input
- Recording workflow orchestration

## Local Contracts

### Naming Conventions
- Audio devices: use "microphone" and "system" (not "input"/"output")
- Errors: use `anyhow::Result` with context
- Logging: include module context (`app_lib::audio::*`)

### Architecture (Two Parallel Paths)

```
Raw Audio (Mic + System)
         ↓
    Audio Pipeline Manager
         ↓
    ┌─────────────┬─────────────┐
    │ Recording   │ Transcription│
    │ (Pre-mixed) │ (VAD-filtered)│
    └─────────────┴─────────────┘
```

**Key Insight**: Professional mixing for recording, VAD for transcription (only speech segments).

### Module Structure

| Issue Type | Location |
|------------|----------|
| Device detection | `devices/discovery.rs` or `devices/platform/{platform}.rs` |
| Microphone/speaker | `devices/microphone.rs` or `devices/speakers.rs` |
| Audio capture | `capture/microphone.rs` or `capture/system.rs` |
| Mixing/processing | `pipeline.rs` |
| Recording workflow | `recording_manager.rs` |

### Key Components

- `AudioMixerRingBuffer`: Mic + system sync
- `ProfessionalAudioMixer`: RMS-based ducking
- `AudioPipelineManager`: VAD, mixing, distribution

## Work Guidance

### Thread Safety

```rust
Arc<RwLock<T>>      // Shared state across async tasks
Arc<AtomicBool>     // Simple flags
```

### Performance-Aware Logging

```rust
perf_debug!()   // Hot-path logging, zero cost in release
perf_trace!()   // Even finer-grained, eliminated in release
```

### Testing Audio

```bash
RUST_LOG=app_lib::audio=debug ./clean_run.sh
```

### Platform-Specific

| Platform | Audio Capture | Special Requirements |
|----------|--------------|----------------------|
| macOS | ScreenCaptureKit (macOS 13+) | Mic + screen recording permissions; virtual audio device (BlackHole) for system capture |
| Windows | WASAPI loopback | Visual Studio Build Tools; WASAPI can conflict with other apps |
| Linux | ALSA/PulseAudio | cmake, llvm, libomp |

## Verification

```bash
# Build check
cargo check --manifest-path frontend/src-tauri/Cargo.toml

# Run with debug logging
RUST_LOG=app_lib::audio=debug ./clean_run.sh
```

## Child DOX Index

This module has no child docs.
