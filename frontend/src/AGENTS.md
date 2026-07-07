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

### Error Handling

```typescript
try {
  await invoke('command');
} catch (error) {
  toast.error(`Failed: ${error}`);  // User-friendly message
}
```

## Work Guidance

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

This module has no child docs.
