# Twin Project Manual

**Repository:** [test_tauri/twin](D:/test_tauri/twin)
**Branch:** main

## Core Contract

- AGENTS.md files are binding work contracts for their subtrees
- Work products, source materials, instructions, records, assets, and durable docs must stay understandable from the nearest applicable AGENTS.md plus every parent AGENTS.md above it
- This file is the source of truth for project context

## Read Before Editing

1. Read the root AGENTS.md
2. Identify every file or folder you expect to touch
3. Walk from the repository root to each target path
4. Read every AGENTS.md found along each route
5. If a parent AGENTS.md lists a child AGENTS.md whose scope contains the path, read that child and continue from there
6. Use the nearest AGENTS.md as the local contract and parent docs for repo-wide rules
7. If docs conflict, the closer doc controls local work details, but no child doc may weaken DOX

Do not rely on memory. Re-read the applicable DOX chain in the current session before editing.

## Edit Lifecycle

### States

- PLANNING: identifying target paths and scope
- READING: traversing DOX chain, reading all AGENTS.md along each route
- EDITING: making code changes
- DOX_PASS: updating owning docs and affected parents/children
- VERIFIED: docs refreshed, verification passed, stale text removed

### Transitions

- PLANNING -> READING: when target paths identified
- READING -> EDITING: when all AGENTS.md in path have been read
- EDITING -> DOX_PASS: when code changes are complete
- DOX_PASS -> VERIFIED: when owning docs updated + verification passes

### Guards (illegal transitions)

- EDITING -> VERIFIED: BLOCKED (DOX pass is mandatory)
- PLANNING -> EDITING: BLOCKED (reading required first)
- DOX_PASS -> EDITING: BLOCKED (must proceed to verification)

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
| LLM Integration | llama-cpp-4 (local), Ollama, Claude, Groq, OpenRouter |

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

## Deterministic Core vs Non-Deterministic Edges

### Deterministic Core (no agent judgment — enforce strictly)

- File hierarchy and DOX chain structure
- Edit lifecycle invariants (above)
- Naming conventions defined in child AGENTS.md
- Module ownership boundaries
- Critical constraints (no separate backend, cross-platform paths, etc.)
- AGENTS.md section structure and Child DOX Index contents

### Non-Deterministic Edges (agent judgment welcome)

- How to implement a feature within the established model
- Code style within documented conventions
- Performance optimization approaches
- Error message wording
- Component decomposition within owned modules

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

## Modeling Rule

For any workflow with 3+ states or conditional branches:
1. Model it explicitly (states, transitions, guards) before implementing
2. Keep the model in the nearest owning AGENTS.md
3. Reference the model in code — do not scatter logic across files
4. If code diverges from model, update the model or fix the code
5. Never encode control flow as numbered prose steps — use invariants or state diagrams

## Closeout

Before claiming a task is complete, verify the Edit Lifecycle has reached VERIFIED:
- All changed paths re-checked against DOX chain
- Nearest owning docs and affected parents/children updated
- Child DOX Index refreshed
- Stale or contradictory text removed
- Verification run when applicable
- Report any docs intentionally left unchanged and why

## User Preferences

When the user requests a durable behavior change, record it here or in the relevant child AGENTS.md

## Child DOX Index

| Path | Scope | Purpose |
|------|-------|---------|
| `frontend/src-tauri/src/audio/` | Audio pipeline | Device detection, capture, mixing, VAD, recording orchestration |
| `frontend/src-tauri/src/whisper_engine/` | Transcription | Whisper model management, parallel processing, transcription workflows |
| `frontend/src-tauri/src/llm_engine/` | LLM inference | Local LLM model management, GGUF inference via llama-cpp-4, GPU acceleration |
| `frontend/src-tauri/src/parakeet_engine/` | Parakeet STT | NVIDIA NeMo Parakeet ONNX transcription engine |
| `frontend/src-tauri/src/lifecycle/` | App lifecycle | Resource management, shutdown handling, graceful termination |
| `frontend/src-tauri/src/database/` | Database | SQLite persistence, migrations, repositories, data models |
| `frontend/src-tauri/src/summary/` | Summary generation | LLM orchestration, templates, caching, language detection |
| `frontend/src-tauri/src/api/` | API commands | Tauri command layer bridging frontend to backend repositories |
| `frontend/src-tauri/src/ollama/` | Ollama integration | Model discovery, metadata caching, context size lookup |
| `frontend/src-tauri/src/openai/` | OpenAI utilities | OpenAI-specific helpers (generic path in summary/llm_client.rs) |
| `frontend/src-tauri/src/console_utils/` | Console utilities | Developer console toggle and debug utilities |
| `frontend/src/` | Frontend UI | React components, state management, Tauri IPC patterns |
| `docs/design-system/` | Design system | Tokens, component patterns, layout & animation rules |

## When Stuck

Debugging techniques (try in any order):
- Check `.cursor/rules/` for scoped guidance on specific file types
- Read the key files listed in child AGENTS.md files
- Enable debug logging: `RUST_LOG=debug ./clean_run.sh`
- Open DevTools: `Cmd+Shift+I` (macOS) or `Ctrl+Shift+I` (Windows)
## Cursor Rules

Code patterns are enforced via `.cursor/rules/` — these auto-apply in Cursor when editing matching files:

| Rule | Governs |
|:-----|:--------|
| `design-tokens.mdc` | Colors, typography, spacing, shadows (all frontend files) |
| `design-components.mdc` | Button, card, input, badge patterns (`components/ui/`) |
| `design-layout.mdc` | Section structure, grids, animation, desktop layout (pages and feature components) |
| `architecture.mdc` | Tauri command/event patterns, thread safety (Rust backend root) |
| `testing-debugging.mdc` | Logging, DevTools, platform quirks (manual attach) |

These rules complement AGENTS.md files: rules enforce code patterns, AGENTS.md files govern process, workflows, and module contracts. When both apply, follow both — rules for how to write code, AGENTS.md for what to do before/after editing.
