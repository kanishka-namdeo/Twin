use crate::database::models::ActionItem;
use chrono::Utc;
use sqlx::{Error as SqlxError, SqlitePool};
use tracing::{error, info};
use uuid::Uuid;

pub struct ActionItemsRepository;

impl ActionItemsRepository {
    /// Create a new action item
    pub async fn create_action_item(
        pool: &SqlitePool,
        meeting_id: &str,
        text: &str,
    ) -> Result<ActionItem, SqlxError> {
        let action_item_id = format!("action-{}", Uuid::new_v4());
        let now = Utc::now();

        let action_item = sqlx::query_as::<_, ActionItem>(
            "INSERT INTO action_items (id, meeting_id, text, completed, created_at)
             VALUES (?, ?, ?, 0, ?)
             RETURNING *"
        )
        .bind(&action_item_id)
        .bind(meeting_id)
        .bind(text)
        .bind(now)
        .fetch_one(pool)
        .await?;

        info!("Created action item {} for meeting {}", action_item_id, meeting_id);
        Ok(action_item)
    }

    /// Get all action items for a meeting
    pub async fn get_action_items_by_meeting(
        pool: &SqlitePool,
        meeting_id: &str,
    ) -> Result<Vec<ActionItem>, SqlxError> {
        let action_items = sqlx::query_as::<_, ActionItem>(
            "SELECT * FROM action_items WHERE meeting_id = ? ORDER BY created_at ASC"
        )
        .bind(meeting_id)
        .fetch_all(pool)
        .await?;

        Ok(action_items)
    }

    /// Get all action items across all meetings
    pub async fn get_all_action_items(
        pool: &SqlitePool,
    ) -> Result<Vec<ActionItem>, SqlxError> {
        let action_items = sqlx::query_as::<_, ActionItem>(
            "SELECT * FROM action_items ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await?;

        Ok(action_items)
    }

    /// Update action item completion status
    pub async fn update_action_item_completed(
        pool: &SqlitePool,
        action_item_id: &str,
        completed: bool,
    ) -> Result<bool, SqlxError> {
        let result = sqlx::query(
            "UPDATE action_items SET completed = ? WHERE id = ?"
        )
        .bind(completed as i32)
        .bind(action_item_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update action item text
    pub async fn update_action_item_text(
        pool: &SqlitePool,
        action_item_id: &str,
        text: &str,
    ) -> Result<bool, SqlxError> {
        let result = sqlx::query(
            "UPDATE action_items SET text = ? WHERE id = ?"
        )
        .bind(text)
        .bind(action_item_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete an action item
    pub async fn delete_action_item(
        pool: &SqlitePool,
        action_item_id: &str,
    ) -> Result<bool, SqlxError> {
        let result = sqlx::query("DELETE FROM action_items WHERE id = ?")
            .bind(action_item_id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Extract action items from summary JSON and save them
    pub async fn extract_and_save_action_items(
        pool: &SqlitePool,
        meeting_id: &str,
        summary_json: &serde_json::Value,
    ) -> Result<Vec<ActionItem>, SqlxError> {
        let mut action_items = Vec::new();

        // Try to extract action items from various possible JSON structures
        if let Some(items) = summary_json.get("action_items").and_then(|v| v.as_array()) {
            for item in items {
                if let Some(text) = item.as_str() {
                    match Self::create_action_item(pool, meeting_id, text).await {
                        Ok(action_item) => action_items.push(action_item),
                        Err(e) => error!("Failed to create action item: {}", e),
                    }
                } else if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                    match Self::create_action_item(pool, meeting_id, text).await {
                        Ok(action_item) => action_items.push(action_item),
                        Err(e) => error!("Failed to create action item: {}", e),
                    }
                }
            }
        }

        info!("Extracted {} action items for meeting {}", action_items.len(), meeting_id);
        Ok(action_items)
    }
}
