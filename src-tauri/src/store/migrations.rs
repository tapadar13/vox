use rusqlite::{Connection, OptionalExtension, params};

use crate::error::{VoxError, VoxResult};

const LATEST_SCHEMA_VERSION: i64 = 1;

const MIGRATION_1: &str = r#"
CREATE TABLE transcriptions (
  id          INTEGER PRIMARY KEY,
  created_at  TEXT NOT NULL,
  text        TEXT NOT NULL,
  raw_text    TEXT NOT NULL,
  language    TEXT NOT NULL,
  duration_ms INTEGER NOT NULL CHECK (duration_ms >= 0),
  latency_ms  INTEGER NOT NULL CHECK (latency_ms >= 0),
  word_count  INTEGER NOT NULL CHECK (word_count >= 0),
  engine_id   TEXT NOT NULL,
  delivered   INTEGER NOT NULL CHECK (delivered IN (0, 1))
);
CREATE INDEX idx_t_created ON transcriptions(created_at DESC);
"#;

pub fn migrate(connection: &mut Connection) -> VoxResult<()> {
    connection
        .execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS schema_version (
               version INTEGER NOT NULL
             );",
        )
        .map_err(db_error)?;

    let current = connection
        .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
            row.get::<_, i64>(0)
        })
        .optional()
        .map_err(db_error)?
        .unwrap_or(0);

    if current > LATEST_SCHEMA_VERSION {
        return Err(VoxError::Storage(format!(
            "database schema {current} is newer than supported schema {LATEST_SCHEMA_VERSION}"
        )));
    }

    if current < 1 {
        let transaction = connection.transaction().map_err(db_error)?;
        transaction.execute_batch(MIGRATION_1).map_err(db_error)?;
        transaction
            .execute("DELETE FROM schema_version", [])
            .map_err(db_error)?;
        transaction
            .execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                params![LATEST_SCHEMA_VERSION],
            )
            .map_err(db_error)?;
        transaction.commit().map_err(db_error)?;
    }

    Ok(())
}

fn db_error(error: rusqlite::Error) -> VoxError {
    VoxError::Storage(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_is_forward_only_and_idempotent() {
        let mut connection = Connection::open_in_memory().unwrap();
        migrate(&mut connection).unwrap();
        migrate(&mut connection).unwrap();

        let version: i64 = connection
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, LATEST_SCHEMA_VERSION);

        let table_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'transcriptions'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_count, 1);
    }
}
