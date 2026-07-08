use crate::database::models::DateTimeUtc;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::SqlitePool;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MeetingNote {
    pub meeting_id: String,
    pub notes_markdown: Option<String>,
    pub notes_json: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MeetingNoteWithDetails {
    pub meeting_id: String,
    pub meeting_title: String,
    pub notes_markdown: Option<String>,
    pub notes_json: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

pub struct MeetingNotesRepository;

impl MeetingNotesRepository {
    /// Get meeting notes by meeting ID
    pub async fn get_meeting_notes(
        pool: &SqlitePool,
        meeting_id: &str,
    ) -> Result<Option<MeetingNote>, sqlx::Error> {
        let note = sqlx::query_as::<_, MeetingNote>(
            "SELECT * FROM meeting_notes WHERE meeting_id = $1",
        )
        .bind(meeting_id)
        .fetch_optional(pool)
        .await?;

        Ok(note)
    }

    /// Save or update meeting notes
    pub async fn save_meeting_notes(
        pool: &SqlitePool,
        meeting_id: &str,
        notes_markdown: Option<&str>,
        notes_json: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let now_str = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO meeting_notes (meeting_id, notes_markdown, notes_json, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT(meeting_id) DO UPDATE SET
                notes_markdown = excluded.notes_markdown,
                notes_json = excluded.notes_json,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(meeting_id)
        .bind(notes_markdown)
        .bind(notes_json)
        .bind(&now_str)
        .bind(&now_str)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete meeting notes
    pub async fn delete_meeting_notes(
        pool: &SqlitePool,
        meeting_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM meeting_notes WHERE meeting_id = $1")
            .bind(meeting_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Get all meetings that have notes
    pub async fn get_meetings_with_notes(
        pool: &SqlitePool,
    ) -> Result<Vec<MeetingNoteWithDetails>, sqlx::Error> {
        let notes = sqlx::query_as::<_, MeetingNoteWithDetails>(
            r#"
            SELECT
                mn.meeting_id,
                m.title as meeting_title,
                mn.notes_markdown,
                mn.notes_json,
                mn.created_at,
                mn.updated_at
            FROM meeting_notes mn
            INNER JOIN meetings m ON mn.meeting_id = m.id
            ORDER BY mn.updated_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(notes)
    }

    /// Check if a meeting has notes
    pub async fn has_notes(
        pool: &SqlitePool,
        meeting_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM meeting_notes WHERE meeting_id = $1"
        )
        .bind(meeting_id)
        .fetch_one(pool)
        .await?;

        Ok(count > 0)
    }
}
