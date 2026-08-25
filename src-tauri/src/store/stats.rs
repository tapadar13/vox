use std::collections::BTreeMap;

use async_trait::async_trait;
use chrono::{Duration, NaiveDate};
use rusqlite::params;

use crate::{
    domain::{StatsInput, StatsService, StatsSnapshot},
    error::{VoxError, VoxResult},
    ports::StatsStore,
};

use super::SqliteStore;

impl SqliteStore {
    fn calculate_stats(&self, typing_wpm: u32, today: NaiveDate) -> VoxResult<StatsSnapshot> {
        let connection = self.connection()?;
        let totals = connection
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(word_count), 0),
                        COALESCE(SUM(duration_ms), 0), COALESCE(SUM(latency_ms), 0)
                 FROM transcriptions",
                [],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                },
            )
            .map_err(db_error)?;

        let first_day = today - Duration::days(83);
        let mut words_by_day = (0..84)
            .map(|offset| (first_day + Duration::days(offset), 0))
            .collect::<BTreeMap<_, _>>();
        let mut statement = connection
            .prepare(
                "SELECT substr(created_at, 1, 10) AS day, SUM(word_count)
                 FROM transcriptions
                 WHERE created_at >= ?1
                 GROUP BY day
                 ORDER BY day",
            )
            .map_err(db_error)?;
        let start = format!("{}T00:00:00Z", first_day.format("%Y-%m-%d"));
        let rows = statement
            .query_map(params![start], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(db_error)?;

        for row in rows {
            let (date, words) = row.map_err(db_error)?;
            if let Ok(date) = NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
                if let Some(slot) = words_by_day.get_mut(&date) {
                    *slot = u64::try_from(words).unwrap_or_default();
                }
            }
        }

        Ok(StatsService::calculate(
            StatsInput {
                transcription_count: to_u64(totals.0),
                total_words: to_u64(totals.1),
                speaking_ms: to_u64(totals.2),
                total_latency_ms: to_u64(totals.3),
                words_by_day,
            },
            typing_wpm,
            today,
        ))
    }
}

#[async_trait]
impl StatsStore for SqliteStore {
    async fn stats(&self, typing_wpm: u32, today: NaiveDate) -> VoxResult<StatsSnapshot> {
        self.calculate_stats(typing_wpm, today)
    }
}

fn to_u64(value: i64) -> u64 {
    u64::try_from(value).unwrap_or_default()
}

fn db_error(error: rusqlite::Error) -> VoxError {
    VoxError::Storage(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{TranscriptRecord, TranscriptStore};

    #[tokio::test]
    async fn stats_are_derived_from_transcript_rows() {
        let store = SqliteStore::in_memory().unwrap();
        store
            .insert(TranscriptRecord {
                id: None,
                created_at: "2026-08-25T08:30:00Z".to_owned(),
                text: "hello local world".to_owned(),
                raw_text: "hello local world".to_owned(),
                language: "en".to_owned(),
                duration_ms: 1_500,
                latency_ms: 300,
                word_count: 3,
                engine_id: "whisper-turbo".to_owned(),
                delivered: true,
            })
            .await
            .unwrap();

        let snapshot = store
            .stats(
                40,
                NaiveDate::from_ymd_opt(2026, 8, 25).expect("valid date"),
            )
            .await
            .unwrap();
        assert_eq!(snapshot.transcription_count, 1);
        assert_eq!(snapshot.total_words, 3);
        assert_eq!(snapshot.activity.last().unwrap().words, 3);
    }
}
