# Database Module

## Purpose

SQLite persistence layer for all meeting data — meetings, transcripts, summaries, settings, speakers, action items, notes, and full-text search. Provides connection management, schema migrations, repository pattern, and Tauri commands for database lifecycle.

## Ownership

This module owns:
- SQLite database creation, migration, and WAL recovery
- Data models (Rust structs mapped to tables)
- Repository pattern for all CRUD operations
- Tauri commands for database initialization, import, and folder access
- Speaker diarization persistence
- FTS5 full-text search index

## Local Contracts

### Database Location

| Environment | Path |
|-------------|------|
| macOS Dev | `~/Library/Application Support/Twin/meeting_minutes.sqlite` |
| Windows Dev | `%APPDATA%\Twin\meeting_minutes.sqlite` |
| Linux Dev | `~/.config/Twin/meeting_minutes.sqlite` |

Legacy `.db` files are auto-migrated to `.sqlite` on first launch.

### Schema Management

Migrations live in `frontend/src-tauri/migrations/` and run via `sqlx::migrate!()` at startup.

### Key Tables

| Table | Purpose |
|-------|---------|
| `meetings` | Meeting metadata (id, title, timestamps, folder_path, meeting_context) |
| `transcripts` | Transcript segments with audio timing and speaker_id |
| `transcript_chunks` | Raw transcript text and processing parameters |
| `summary_processes` | Summary generation state machine (status, result, error, backup) |
| `settings` | Summary model config (provider, model, API keys, custom OpenAI JSON) |
| `transcript_settings` | Transcription model config (provider, model, API keys) |
| `speakers` | Speaker diarization labels per meeting |
| `action_items` | Extracted action items with completion status |
| `meeting_notes` | User-authored notes (markdown + JSON) per meeting |
| `transcripts_fts` | FTS5 virtual table for full-text search |

### Repository Pattern

All repositories are stateless structs with static async methods taking `&SqlitePool`:

| Repository | File |
|------------|------|
| `MeetingsRepository` | `repositories/meeting.rs` |
| `TranscriptsRepository` | `repositories/transcript.rs` |
| `TranscriptChunksRepository` | `repositories/transcript_chunk.rs` |
| `SummaryProcessesRepository` | `repositories/summary.rs` |
| `SettingsRepository` | `repositories/setting.rs` |
| `SpeakersRepository` | `repositories/speaker.rs` |
| `ActionItemsRepository` | `repositories/action_item.rs` |
| `FtsSearchRepository` | `repositories/fts_search.rs` |
| `MeetingNotesRepository` | `repositories/meeting_notes.rs` |

### Summary Process State Model

States:
- PENDING: summary generation started, awaiting result
- completed: summary generated and saved
- failed: generation failed, backup restored
- cancelled: user cancelled, backup restored

Transitions:
- (created) -> PENDING: on `create_or_reset_process`
- PENDING -> completed: on `update_process_completed`
- PENDING -> failed: on `update_process_failed`
- PENDING -> cancelled: on `update_process_cancelled`

Guards:
- Cannot complete/failed/cancel a non-existent process
- On reset, existing result is backed up before status returns to PENDING
- On failure/cancel, backup is restored if available

### Deterministic Core

- Database path resolution (cross-platform via Tauri path APIs)
- Migration execution order
- Repository method signatures and SQL queries
- Transaction boundaries (meeting delete cascades, summary save atomicity)
- WAL checkpoint on cleanup

### Non-Deterministic Edges

- WAL corruption recovery strategy
- Default model configuration for fresh installs
- FTS5 query construction and ranking

### Tauri Commands

| Command | Purpose |
|---------|---------|
| `check_first_launch` | Detect if database exists |
| `import_and_initialize_database` | Import legacy `.db` file |
| `initialize_fresh_database` | Create fresh DB with defaults |
| `get_database_directory` | Get app data directory path |
| `open_database_folder` | Open folder in file explorer |
| `get_speakers` | Get speakers for a meeting |
| `rename_speaker` | Update speaker label |

### Tauri Events

| Event | When |
|-------|------|
| `first-launch-detected` | First launch, no DB exists |
| `database-initialized` | DB ready after import or fresh init |

## Work Guidance

### Transaction Safety

All multi-table operations must use transactions. Meeting deletion cascades through `transcript_chunks`, `summary_processes`, `transcripts`, then `meetings`.

### WAL Management

The database uses WAL mode. On startup, orphaned WAL/SHM files are detected and cleaned if they cause corruption. On shutdown, `PRAGMA wal_checkpoint(TRUNCATE)` flushes and removes the WAL file.

### Settings Singleton

Both `settings` and `transcript_settings` use `id = '1'` as a singleton row with `INSERT ... ON CONFLICT DO UPDATE` upserts.

### Integration Points

- `api/` module calls repositories for Tauri commands exposed to frontend
- `summary/` module uses `SummaryProcessesRepository` and `TranscriptChunksRepository`
- `audio/` module saves transcripts via `TranscriptsRepository`
- `lifecycle/` module calls `DatabaseManager::cleanup()` on shutdown

## Verification

```bash
# Build check
cargo check --manifest-path frontend/src-tauri/Cargo.toml

# Run with debug logging
RUST_LOG=app_lib::database=debug ./clean_run.sh
```

## Child DOX Index

This module has no child docs.
