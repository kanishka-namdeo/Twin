# Twin Project Manual

**Repository:** [test_tauri/twin](D:/test_tauri/twin)
**Branch:** main

## Core Contract

- AGENTS.md files are binding work contracts for their subtrees
- Work products, source materials, instructions, records, assets, and durable docs must stay understandable from the nearest applicable AGENTS.md plus every parent AGENTS.md above it
- This file is the source of truth for project context

## Read Before Editing

1. Read this root AGENTS.md
2. Identify every file or folder you expect to touch
3. Walk from the repository root to each target path
4. Read every AGENTS.md found along each route
5. If a parent AGENTS.md lists a child AGENTS.md whose scope contains the path, read that child and continue from there
6. Use the nearest AGENTS.md as the local contract and parent docs for repo-wide rules
7. If docs conflict, the closer doc controls local work details, but no child doc may weaken DOX

Do not rely on memory. Re-read the applicable DOX chain in the current session before editing.

## Update After Editing

Every meaningful change requires a DOX pass before the task is done.

Update the closest owning AGENTS.md when a change affects:

- purpose, scope, ownership, or responsibilities
- durable structure, contracts, workflows, or operating rules
- required inputs, outputs, permissions, constraints, side effects, or artifacts
- user preferences about behavior, communication, process, organization, or quality
- AGENTS.md creation, deletion, move, rename, or index contents

Update parent docs when parent-level structure, ownership, workflow, or child index changes. Update child docs when parent changes alter local rules. Remove stale or contradictory text immediately. Small edits that do not change behavior or contracts may leave docs unchanged, but the DOX pass still must happen.

## Project Overview

**Twin** is a privacy-first AI meeting assistant that captures, transcribes, and summarizes meetings entirely on local infrastructure. The supported application is the Tauri desktop app with a Rust core.

**Architecture**: Tauri 2.x (Rust) + Next.js 16 + React 19
**Key Feature**: No separate backend — all features handled by the Tauri app (Rust core)

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop App | Tauri 2.x (Rust) |
| Frontend UI | Next.js 16 + React 19 + TypeScript |
| Audio Processing | Rust (cpal, whisper-rs) |
| Transcription | Whisper.cpp / whisper-rs, Parakeet |
| LLM Integration | Ollama (local), Claude, Groq, OpenRouter |

## Development Commands

### macOS (run from `frontend/` directory)
```bash
./clean_run.sh              # Clean build + run (info logging)
./clean_run.sh debug        # Run with debug logging
./clean_build.sh            # Production build
```

### Windows (run from `frontend/` directory)
```bash
clean_run_windows.bat       # Clean build + run
clean_build_windows.bat     # Production build
```

### Manual
```bash
pnpm install                # Install dependencies
pnpm run dev                # Next.js dev server (port 3118)
pnpm run tauri:dev          # Full Tauri dev mode
pnpm run tauri:build        # Production build
```

### GPU-Specific
```bash
pnpm run tauri:dev:metal    # macOS Metal
pnpm run tauri:dev:cuda     # NVIDIA CUDA
pnpm run tauri:dev:vulkan   # AMD/Intel Vulkan
pnpm run tauri:dev:cpu      # CPU-only
```

## Critical Constraints

1. **No Separate Backend**: Meeting persistence, transcription, and LLM features are handled by the Tauri app. Do NOT reintroduce the archived FastAPI backend (`backend/`) as a dependency.

2. **Legacy Backend Archive**: The Python/FastAPI backend under `backend/` is archived and unsupported. Do not use for current development, installs, or production.

3. **Cross-Platform Paths**: Use Tauri's path APIs (`downloadDir`, etc.) for cross-platform compatibility. Never hardcode paths.

4. **Audio Permissions**: Request permissions early. macOS requires both microphone AND screen recording for system audio.

## Hierarchy

- Root AGENTS.md is the DOX rail: project-wide instructions, global preferences, durable workflow rules, and the top-level Child DOX Index
- Child AGENTS.md files own domain-specific instructions and their own Child DOX Index
- Each parent explains what its direct children cover and what stays owned by the parent
- The closer a doc is to the work, the more specific and practical it must be

## Child Doc Shape

- Create a child AGENTS.md when a folder becomes a durable boundary with its own purpose, rules, responsibilities, workflow, materials, or quality standards
- Work Guidance must reflect the current standards of the project or user instructions; if there are no specific standards or instructions yet, leave it empty
- Verification must reflect an existing check; if no verification framework exists yet, leave it empty and update it when one exists

Default section order:
- Purpose
- Ownership
- Local Contracts
- Work Guidance
- Verification
- Child DOX Index

## Style

- Keep docs concise, current, and operational
- Document stable contracts, not diary entries
- Put broad rules in parent docs and concrete details in child docs
- Prefer direct bullets with explicit names
- Do not duplicate rules across many files unless each scope needs a local version
- Delete stale notes instead of explaining history
- Trim obvious statements, repeated rules, misplaced detail, and warnings for risks that no longer exist

## Closeout

1. Re-check changed paths against the DOX chain
2. Update nearest owning docs and any affected parents or children
3. Refresh every affected Child DOX Index
4. Remove stale or contradictory text
5. Run existing verification when relevant
6. Report any docs intentionally left unchanged and why

## User Preferences

When the user requests a durable behavior change, record it here or in the relevant child AGENTS.md

## Child DOX Index

| Path | Scope | Purpose |
|------|-------|---------|
| `frontend/src-tauri/src/audio/` | Audio pipeline | Device detection, capture, mixing, VAD, recording orchestration |
| `frontend/src-tauri/src/whisper_engine/` | Transcription | Whisper model management, parallel processing, transcription workflows |
| `frontend/src-tauri/src/lifecycle/` | App lifecycle | Resource management, shutdown handling, graceful termination |
| `frontend/src/` | Frontend UI | React components, state management, Tauri IPC patterns |

## When Stuck

1. Check `.cursor/rules/` for scoped guidance on specific file types
2. Read the key files listed in child AGENTS.md files
3. Enable debug logging: `RUST_LOG=debug ./clean_run.sh`
4. Open DevTools: `Cmd+Shift+I` (macOS) or `Ctrl+Shift+I` (Windows)
