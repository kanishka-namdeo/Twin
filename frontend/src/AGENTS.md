# Frontend UI

## Purpose

React/Next.js frontend interface for meeting recording, transcription display, and user interaction with the Tauri backend.

## Ownership

This module owns:
- React components for recording interface
- State management for meetings and transcription
- Tauri IPC (invoke commands, listen to events)
- UI/UX for meeting workflows
- Audio level monitoring and visualization

## Local Contracts

### Calling Tauri Commands

```typescript
import { invoke } from '@tauri-apps/api/core';

await invoke('start_recording', {
  mic_device_name: "Built-in Microphone",
  system_device_name: "BlackHole 2ch",
  meeting_name: "Team Standup"
});
```

### Listening to Tauri Events

```typescript
import { listen } from '@tauri-apps/api/event';

await listen<TranscriptUpdate>('transcript-update', (event) => {
  setTranscripts(prev => [...prev, event.payload]);
});
```

### State Management

- `SidebarProvider.tsx`: Global state (meetings, recording status)
- Pattern: Rust state → emit event → React listener → context update

### Meeting Recording State Model

States:
- NoMeeting: idle, no recording active
- Configuring: user selecting devices and meeting name
- Recording: audio capture in progress, transcript streaming
- Reviewing: recording stopped, viewing transcript
- Saved: meeting persisted

Transitions:
- NoMeeting -> Configuring: on "New Meeting" action
- Configuring -> Recording: on start_recording invoke success
- Recording -> Reviewing: on stop_recording invoke
- Reviewing -> Saved: on save action
- Reviewing -> NoMeeting: on discard
- Configuring -> NoMeeting: on cancel

Guards:
- Cannot start recording without device confirmation
- Cannot save without a completed recording
- Cannot start new recording while one is active

### Deterministic Core

- State transitions (enforced by guards above)
- Tauri IPC call signatures and event listener contracts
- Component ownership boundaries
- Global state shape in SidebarProvider

### Non-Deterministic Edges

- UI layout and component decomposition
- Transcript rendering strategy
- Error message wording and UX feedback

### Error Handling

```typescript
try {
  await invoke('command');
} catch (error) {
  toast.error(`Failed: ${error}`);  // User-friendly message
}
```

## Work Guidance

### Design System

All UI work must follow the design system rules. See `docs/design-system/` for full specs.

| Doc | Governs |
|:----|:--------|
| `docs/design-system/tokens.md` | Colors, typography, spacing, shadows |
| `docs/design-system/components.md` | Button, card, input, badge patterns |
| `docs/design-system/layout.md` | Section structure, grids, animation, responsive, **desktop layout** |

Cursor rules enforce these automatically:
- `design-tokens.mdc` — always loaded for frontend files
- `design-components.mdc` — loaded when editing `components/ui/`
- `design-layout.mdc` — loaded when editing pages and feature components

### Desktop App Constraints

This is a Tauri desktop app, not a website. Key constraints:

- **Resizable panels**: Sidebar + content use `react-resizable-panels`, not fixed margins
- **Window**: Min size `900×600`, default `1100×700`, never open maximized
- **State persistence**: Save window size/position and sidebar width across sessions
- **Keyboard shortcuts**: `Cmd/Ctrl+N` (new), `Cmd/Ctrl+K` (search), `Cmd/Ctrl+B` (toggle sidebar), `Escape` (cancel)
- **No hardcoded widths**: Use flex/grid/panel constraints, not pixel widths
- **Content scaling**: Show more content as window grows, don't just stretch

### Performance

- Transcript rendering: virtualized for large meetings
- Audio level monitoring: throttled to 60fps
- Use React.memo for expensive components

### Component Structure

| Component Type | Location |
|----------------|----------|
| Recording UI | `components/Recording/` |
| Meeting List | `components/Sidebar/` |
| Transcript Display | `components/Transcript/` |
| Settings | `components/Settings/` |

### Integration Points

- Receives events from `audio` and `whisper_engine` modules
- Sends commands to Rust backend via Tauri invoke
- Manages local UI state with React context

## Verification

```bash
# Install dependencies
pnpm install

# Run dev server
pnpm run dev

# Build for production
pnpm run build
```

## Child DOX Index

| Path | Scope | Purpose |
|------|-------|---------|
| `docs/design-system/` | Design system | Tokens, component patterns, layout & animation rules |

### Frontend Sub-modules (no separate AGENTS.md — documented here)

| Path | Scope | Purpose |
|------|-------|---------|
| `frontend/src/components/` | UI components | React components for recording, transcript, settings, sidebar, onboarding |
| `frontend/src/services/` | Service layer | Typed wrappers for Tauri IPC (configService, recordingService) |
| `frontend/src/contexts/` | React context | Global state management (ConfigContext for model/device/language prefs) |
| `frontend/src/types/` | TypeScript types | Shared type definitions (llm.ts, betaFeatures.ts) |
| `frontend/src/app/` | Next.js pages | App router pages (home, meeting-details, settings, notes, action-items) |
