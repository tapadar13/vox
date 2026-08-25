use std::{
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
};

use async_trait::async_trait;
use rusqlite::{Connection, Row, params};

use crate::{
    error::{VoxError, VoxResult},
    ports::{HistoryQuery, TranscriptRecord, TranscriptStore},
};

use super::migrate;

#[derive(Clone)]
pub struct SqliteStore {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteStore {
    pub fn open(path: &Path) -> VoxResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut connection = Connection::open(path).map_err(db_error)?;
        migrate(&mut connection)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    #[cfg(test)]
    pub fn in_memory() -> VoxResult<Self> {
        let mut connection = Connection::open_in_memory().map_err(db_error)?;
        migrate(&mut connection)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub(crate) fn connection(&self) -> VoxResult<MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|_| VoxError::Storage("database lock was poisoned".to_owned()))
    }

    fn insert_record(&self, record: TranscriptRecord) -> VoxResult<i64> {
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO transcriptions (
                   created_at, text, raw_text, language, duration_ms, latency_ms,
                   word_count, engine_id, delivered
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    record.created_at,
                    record.text,
                    record.raw_text,
                    record.language,
                    to_i64(record.duration_ms),
                    to_i64(record.latency_ms),
                    to_i64(record.word_count),
                    record.engine_id,
                    record.delivered,
                ],
            )
            .map_err(db_error)?;
        Ok(connection.last_insert_rowid())
    }

    fn query_history(&self, query: HistoryQuery) -> VoxResult<Vec<TranscriptRecord>> {
        let connection = self.connection()?;
        let limit = i64::from(query.limit.clamp(1, 200));
        let offset = i64::from(query.offset);

        if let Some(search) = query.search.filter(|value| !value.trim().is_empty()) {
            let mut statement = connection
                .prepare(
                    "SELECT id, created_at, text, raw_text, language, duration_ms,
                            latency_ms, word_count, engine_id, delivered
                     FROM transcriptions
                     WHERE text LIKE ?1 ESCAPE '\\'
                     ORDER BY created_at DESC, id DESC
                     LIMIT ?2 OFFSET ?3",
                )
                .map_err(db_error)?;
            let pattern = format!("%{}%", escape_like(search.trim()));
            let records = statement
                .query_map(params![pattern, limit, offset], row_to_record)
                .map_err(db_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(db_error)?;
            Ok(records)
        } else {
            let mut statement = connection
                .prepare(
                    "SELECT id, created_at, text, raw_text, language, duration_ms,
                            latency_ms, word_count, engine_id, delivered
                     FROM transcriptions
                     ORDER BY created_at DESC, id DESC
                     LIMIT ?1 OFFSET ?2",
                )
                .map_err(db_error)?;
            let records = statement
                .query_map(params![limit, offset], row_to_record)
                .map_err(db_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(db_error)?;
            Ok(records)
        }
    }
}

#[async_trait]
impl TranscriptStore for SqliteStore {
    async fn insert(&self, record: TranscriptRecord) -> VoxResult<i64> {
        self.insert_record(record)
    }

    async fn history(&self, query: HistoryQuery) -> VoxResult<Vec<TranscriptRecord>> {
        self.query_history(query)
    }

    async fn delete(&self, id: i64) -> VoxResult<()> {
        self.connection()?
            .execute("DELETE FROM transcriptions WHERE id = ?1", params![id])
            .map_err(db_error)?;
        Ok(())
    }

    async fn clear(&self) -> VoxResult<()> {
        self.connection()?
            .execute("DELETE FROM transcriptions", [])
            .map_err(db_error)?;
        Ok(())
    }
}

fn row_to_record(row: &Row<'_>) -> rusqlite::Result<TranscriptRecord> {
    Ok(TranscriptRecord {
        id: row.get(0)?,
        created_at: row.get(1)?,
        text: row.get(2)?,
        raw_text: row.get(3)?,
        language: row.get(4)?,
        duration_ms: from_i64(row.get(5)?),
        latency_ms: from_i64(row.get(6)?),
        word_count: from_i64(row.get(7)?),
        engine_id: row.get(8)?,
        delivered: row.get(9)?,
    })
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn to_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn from_i64(value: i64) -> u64 {
    u64::try_from(value).unwrap_or_default()
}

fn db_error(error: rusqlite::Error) -> VoxError {
    VoxError::Storage(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(text: &str) -> TranscriptRecord {
        TranscriptRecord {
            id: None,
            created_at: "2026-08-25T09:00:00Z".to_owned(),
            text: text.to_owned(),
            raw_text: text.to_lowercase(),
            language: "en".to_owned(),
            duration_ms: 2_000,
            latency_ms: 420,
            word_count: text.split_whitespace().count() as u64,
            engine_id: "whisper-turbo".to_owned(),
            delivered: true,
        }
    }

    #[tokio::test]
    async fn history_is_searchable_and_deletable() {
        let store = SqliteStore::in_memory().unwrap();
        store.insert(record("Hello from Vox")).await.unwrap();
        let second = store.insert(record("Private by design")).await.unwrap();

        let results = store
            .history(HistoryQuery {
                search: Some("Vox".to_owned()),
                ..HistoryQuery::default()
            })
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "Hello from Vox");

        store.delete(second).await.unwrap();
        assert_eq!(
            store.history(HistoryQuery::default()).await.unwrap().len(),
            1
        );
    }

    #[tokio::test]
    async fn percent_in_search_is_literal() {
        let store = SqliteStore::in_memory().unwrap();
        store.insert(record("100% local")).await.unwrap();
        store.insert(record("No cloud")).await.unwrap();

        let results = store
            .history(HistoryQuery {
                search: Some("%".to_owned()),
                ..HistoryQuery::default()
            })
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
    }
}
