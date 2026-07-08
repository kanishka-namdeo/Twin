use crate::database::models::FtsSearchResult;
use sqlx::{Error as SqlxError, SqlitePool};
use tracing::info;

pub struct FtsSearchRepository;

impl FtsSearchRepository {
    /// Full-text search using FTS5
    pub async fn search_meetings_fts(
        pool: &SqlitePool,
        query: &str,
        date_from: Option<String>,
        date_to: Option<String>,
        min_duration: Option<f64>,
        has_summary: Option<bool>,
    ) -> Result<Vec<FtsSearchResult>, SqlxError> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Build the FTS5 query with proper escaping
        let fts_query = query.replace("'", "''");

        // Build dynamic WHERE clause
        let mut conditions = Vec::new();
        let mut bindings: Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send>> = Vec::new();

        // FTS5 search condition
        conditions.push("m.id IN (SELECT DISTINCT t.meeting_id FROM transcripts t JOIN transcripts_fts fts ON t.rowid = fts.rowid WHERE transcripts_fts MATCH ?)".to_string());
        bindings.push(Box::new(fts_query.clone()));

        // Date range filters
        if let Some(ref from) = date_from {
            conditions.push("m.created_at >= ?".to_string());
            bindings.push(Box::new(from.clone()));
        }
        if let Some(ref to) = date_to {
            conditions.push("m.created_at <= ?".to_string());
            bindings.push(Box::new(to.clone()));
        }

        // Duration filter
        if let Some(duration) = min_duration {
            conditions.push("(SELECT COALESCE(SUM(t.duration), 0) FROM transcripts t WHERE t.meeting_id = m.id) >= ?".to_string());
            bindings.push(Box::new(duration));
        }

        // Has summary filter
        if let Some(has_sum) = has_summary {
            if has_sum {
                conditions.push("EXISTS (SELECT 1 FROM summary_processes sp WHERE sp.meeting_id = m.id AND sp.status = 'completed')".to_string());
            }
        }

        let where_clause = conditions.join(" AND ");
        let sql = format!(
            "SELECT DISTINCT m.id, m.title, snippet(transcripts_fts, 0, '<mark>', '</mark>', '...', 32) as snippet, rank
             FROM meetings m
             JOIN transcripts t ON m.id = t.meeting_id
             JOIN transcripts_fts fts ON t.rowid = fts.rowid
             WHERE transcripts_fts MATCH ?
             AND {}
             ORDER BY rank
             LIMIT 50",
            where_clause
        );

        // Execute query with bindings
        let mut query_builder = sqlx::query_as::<_, (String, String, String, f64)>(&sql);
        query_builder = query_builder.bind(fts_query);

        // Add bindings for filters
        if let Some(ref from) = date_from {
            query_builder = query_builder.bind(from.clone());
        }
        if let Some(ref to) = date_to {
            query_builder = query_builder.bind(to.clone());
        }
        if let Some(duration) = min_duration {
            query_builder = query_builder.bind(duration);
        }

        let rows = query_builder.fetch_all(pool).await?;

        let results: Vec<FtsSearchResult> = rows
            .into_iter()
            .map(|(meeting_id, meeting_title, snippet, rank)| FtsSearchResult {
                meeting_id,
                meeting_title,
                snippet,
                rank,
            })
            .collect();

        info!("FTS search for '{}' returned {} results", query, results.len());
        Ok(results)
    }

    /// Simple FTS search without filters
    pub async fn search_simple(
        pool: &SqlitePool,
        query: &str,
    ) -> Result<Vec<FtsSearchResult>, SqlxError> {
        Self::search_meetings_fts(pool, query, None, None, None, None).await
    }
}
