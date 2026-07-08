use log::{error, info};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::database::repositories::speaker::SpeakersRepository;
use crate::state::AppState;

/// Get all speakers for a meeting
#[tauri::command]
pub async fn get_speakers(
    app: AppHandle,
    meeting_id: String,
) -> Result<Vec<SpeakerResponse>, String> {
    info!("Getting speakers for meeting: {}", meeting_id);

    let state = app.state::<AppState>();
    let pool = state.db_manager.pool();

    let speakers = SpeakersRepository::get_speakers(pool, &meeting_id)
        .await
        .map_err(|e| {
            error!("Failed to get speakers: {}", e);
            format!("Failed to get speakers: {}", e)
        })?;

    let response: Vec<SpeakerResponse> = speakers
        .into_iter()
        .map(|s| SpeakerResponse {
            id: s.id,
            meeting_id: s.meeting_id,
            speaker_index: s.speaker_index,
            label: if s.label.is_empty() {
                format!("Speaker {}", s.speaker_index + 1)
            } else {
                s.label
            },
        })
        .collect();

    Ok(response)
}

/// Rename a speaker
#[tauri::command]
pub async fn rename_speaker(
    app: AppHandle,
    speaker_id: i32,
    new_label: String,
) -> Result<(), String> {
    info!("Renaming speaker {} to '{}'", speaker_id, new_label);

    let state = app.state::<AppState>();
    let pool = state.db_manager.pool();

    SpeakersRepository::rename_speaker(pool, speaker_id, &new_label)
        .await
        .map_err(|e| {
            error!("Failed to rename speaker: {}", e);
            format!("Failed to rename speaker: {}", e)
        })?;

    Ok(())
}

/// Response type for speaker data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerResponse {
    pub id: i32,
    pub meeting_id: String,
    pub speaker_index: i32,
    pub label: String,
}
