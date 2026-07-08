use crate::database::models::Speaker;
use chrono::Utc;
use sqlx::{Error as SqlxError, SqlitePool};
use tracing::{error, info};

pub struct SpeakersRepository;

impl SpeakersRepository {
    /// Get all speakers for a meeting
    pub async fn get_speakers(
        pool: &SqlitePool,
        meeting_id: &str,
    ) -> Result<Vec<Speaker>, SqlxError> {
        let speakers = sqlx::query_as::<_, Speaker>(
            "SELECT id, meeting_id, speaker_index, label, created_at, updated_at 
             FROM speakers 
             WHERE meeting_id = ? 
             ORDER BY speaker_index"
        )
        .bind(meeting_id)
        .fetch_all(pool)
        .await?;

        Ok(speakers)
    }

    /// Get a specific speaker by ID
    pub async fn get_speaker(
        pool: &SqlitePool,
        speaker_id: i32,
    ) -> Result<Option<Speaker>, SqlxError> {
        let speaker = sqlx::query_as::<_, Speaker>(
            "SELECT id, meeting_id, speaker_index, label, created_at, updated_at 
             FROM speakers 
             WHERE id = ?"
        )
        .bind(speaker_id)
        .fetch_optional(pool)
        .await?;

        Ok(speaker)
    }

    /// Create or update speakers for a meeting
    /// This will create speakers if they don't exist, or update existing ones
    pub async fn upsert_speakers(
        pool: &SqlitePool,
        meeting_id: &str,
        speaker_count: u32,
    ) -> Result<Vec<Speaker>, SqlxError> {
        let now = Utc::now();
        let mut speakers = Vec::new();

        for i in 0..speaker_count {
            let label = format!("Speaker {}", i + 1);
            
            // Try to insert, ignore if already exists
            let result = sqlx::query(
                "INSERT INTO speakers (meeting_id, speaker_index, label, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?)
                 ON CONFLICT(meeting_id, speaker_index) DO NOTHING"
            )
            .bind(meeting_id)
            .bind(i as i32)
            .bind(&label)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await;

            if let Err(e) = result {
                error!("Failed to upsert speaker {} for meeting {}: {}", i, meeting_id, e);
                continue;
            }

            // Fetch the speaker to return it
            if let Ok(Some(speaker)) = sqlx::query_as::<_, Speaker>(
                "SELECT id, meeting_id, speaker_index, label, created_at, updated_at 
                 FROM speakers 
                 WHERE meeting_id = ? AND speaker_index = ?"
            )
            .bind(meeting_id)
            .bind(i as i32)
            .fetch_optional(pool)
            .await
            {
                speakers.push(speaker);
            }
        }

        info!("Upserted {} speakers for meeting {}", speakers.len(), meeting_id);
        Ok(speakers)
    }

    /// Rename a speaker
    pub async fn rename_speaker(
        pool: &SqlitePool,
        speaker_id: i32,
        new_label: &str,
    ) -> Result<(), SqlxError> {
        let now = Utc::now();
        
        sqlx::query(
            "UPDATE speakers 
             SET label = ?, updated_at = ? 
             WHERE id = ?"
        )
        .bind(new_label)
        .bind(now)
        .bind(speaker_id)
        .execute(pool)
        .await?;

        info!("Renamed speaker {} to '{}'", speaker_id, new_label);
        Ok(())
    }

    /// Delete all speakers for a meeting
    pub async fn delete_speakers(
        pool: &SqlitePool,
        meeting_id: &str,
    ) -> Result<u64, SqlxError> {
        let result = sqlx::query("DELETE FROM speakers WHERE meeting_id = ?")
            .bind(meeting_id)
            .execute(pool)
            .await?;

        info!("Deleted {} speakers for meeting {}", result.rows_affected(), meeting_id);
        Ok(result.rows_affected())
    }
}
