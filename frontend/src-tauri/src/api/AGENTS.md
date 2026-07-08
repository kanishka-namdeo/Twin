# API Module

## Purpose

Tauri command layer that bridges frontend requests to backend repositories and services. Exposes meeting management, transcript operations, configuration, search, export, and utility commands to the Next.js frontend via Tauri IPC.

## Ownership

This module owns:
- All Tauri commands prefixed with `api_` for meeting/transcript operations
- Request/response data structures for frontend communication
- Meeting CRUD operations (list, get, delete, update title)
- Transcript save and search operations
- Configuration management (model config, API keys, transcript config)
- Export functionality (SRT, VTT, TXT, bundle formats)
- Action items and meeting notes commands
- Full-text search (FTS5) commands
- Custom OpenAI configuration commands
- Meeting audio file path resolution

## Local Contracts

### Data Structures

| Structure | Purpose |
|-----------|---------|
| `Meeting` | Meeting id + title |
| `MeetingDetails` | Full meeting with transcripts |
| `MeetingMetadata` | Meeting without transcripts (pagination) |
| `MeetingTranscript` | Transcript segment with audio timing |
| `TranscriptSegment` | Incoming transcript from recording |
| `TranscriptSearchResult` | Search match with context snippet |
| `PaginatedTranscriptsResponse` | Paginated transcript list |
| `ModelConfig` | Summary model configuration |
| `TranscriptConfig` | Transcription model configuration |
| `ActionItemResponse` | Action item with meeting title |
| `FtsSearchResult` | Full-text search result |

### Tauri Commands

| Command | Purpose |
|---------|---------|
| `api_get_meetings` | List all meetings |
| `api_get_meeting` | Get meeting with transcripts |
| `api_get_meeting_metadata` | Get meeting without transcripts |
| `api_get_meeting_transcripts` | Paginated transcripts |
| `api_delete_meeting` | Delete meeting and cascade |
| `api_save_meeting_title` | Update meeting title |
| `api_save_transcript` | Save recording transcripts |
| `api_search_transcripts` | LIKE-based search |
| `api_search_meetings_fts` | FTS5 full-text search |
| `api_get_model_config` | Get summary model config |
| `api_save_model_config` | Save summary model config |
| `api_get_transcript_config` | Get transcription config |
| `api_save_transcript_config` | Save transcription config |
| `api_get_api_key` | Get API key for provider |
| `api_delete_api_key` | Delete API key |
| `api_save_custom_openai_config` | Save custom OpenAI config |
| `api_get_custom_openai_config` | Get custom OpenAI config |
| `api_test_custom_openai_connection` | Test custom endpoint |
| `api_get_all_action_items` | List all action items |
| `api_update_action_item` | Update action item completion |
| `api_get_meeting_notes` | Get meeting notes |
| `api_save_meeting_notes` | Save meeting notes |
| `api_get_meetings_with_notes` | List meetings with notes |
| `api_export_meeting_transcript` | Export transcript (SRT/VTT/TXT) |
| `api_export_meeting_bundle` | Export full meeting bundle |
| `api_get_meeting_audio_path` | Find audio file in meeting folder |
| `api_save_meeting_context` | Save meeting context |
| `api_get_meeting_context` | Get meeting context |
| `open_meeting_folder` | Open folder in file explorer |
| `open_external_url` | Open URL in browser |

### Deterministic Core

- Request/response JSON serialization
- Repository delegation pattern
- Export format generation (SRT/VTT/TXT)
- Audio file discovery logic

### Non-Deterministic Edges

- Error message wording
- File system operations (folder existence, permissions)

## Work Guidance

### Repository Delegation

All database operations delegate to repositories in `database/repositories/`. This module handles request parsing and response formatting only.

### Export Formats

- **SRT**: SubRip format with `HH:MM:SS,mmm` timestamps
- **VTT**: WebVTT format with `HH:MM:SS.mmm` timestamps
- **TXT**: Plain text with `[timestamp] text` per line
- **Bundle**: Markdown with meeting metadata + full transcript

### Integration Points

- Calls `database/repositories/*` for all CRUD operations
- Frontend invokes commands via `@tauri-apps/api/core` `invoke()`
- Uses `database/models.rs` for data structures

## Verification

```bash
# Build check
cargo check --manifest-path frontend/src-tauri/Cargo.toml

# Run with debug logging
RUST_LOG=app_lib::api=debug ./clean_run.sh
```

## Child DOX Index

This module has no child docs.
