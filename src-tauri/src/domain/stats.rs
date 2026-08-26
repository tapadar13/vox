use std::collections::BTreeMap;

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatsInput {
    pub transcription_count: u64,
    pub total_words: u64,
    pub speaking_ms: u64,
    pub total_latency_ms: u64,
    pub words_by_day: BTreeMap<NaiveDate, u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDay {
    pub date: String,
    pub words: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsSnapshot {
    pub transcription_count: u64,
    pub total_words: u64,
    pub speaking_ms: u64,
    pub time_saved_ms: u64,
    pub speaking_wpm: f64,
    pub average_latency_ms: u64,
    pub streak_days: u32,
    pub activity: Vec<ActivityDay>,
}

pub struct StatsService;

impl StatsService {
    pub fn calculate(input: StatsInput, typing_wpm: u32, today: NaiveDate) -> StatsSnapshot {
        let estimated_typing_ms = if typing_wpm == 0 {
            0
        } else {
            input.total_words.saturating_mul(60_000) / u64::from(typing_wpm)
        };
        let time_saved_ms = estimated_typing_ms.saturating_sub(input.speaking_ms);
        let speaking_wpm = if input.speaking_ms == 0 {
            0.0
        } else {
            input.total_words as f64 * 60_000.0 / input.speaking_ms as f64
        };
        let average_latency_ms = input
            .total_latency_ms
            .checked_div(input.transcription_count)
            .unwrap_or_default();

        let activity = input
            .words_by_day
            .iter()
            .map(|(date, words)| ActivityDay {
                date: date.format("%Y-%m-%d").to_string(),
                words: *words,
            })
            .collect();

        StatsSnapshot {
            transcription_count: input.transcription_count,
            total_words: input.total_words,
            speaking_ms: input.speaking_ms,
            time_saved_ms,
            speaking_wpm,
            average_latency_ms,
            streak_days: active_streak(&input.words_by_day, today),
            activity,
        }
    }
}

fn active_streak(activity: &BTreeMap<NaiveDate, u64>, today: NaiveDate) -> u32 {
    let most_recent = activity
        .iter()
        .rev()
        .find_map(|(date, words)| (*words > 0).then_some(*date));
    let Some(most_recent) = most_recent else {
        return 0;
    };

    if most_recent < today - Duration::days(1) {
        return 0;
    }

    let mut cursor = most_recent;
    let mut streak = 0;
    while activity.get(&cursor).copied().unwrap_or_default() > 0 {
        streak += 1;
        cursor -= Duration::days(1);
    }
    streak
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn calculates_time_saved_and_speaking_speed() {
        let snapshot = StatsService::calculate(
            StatsInput {
                transcription_count: 2,
                total_words: 120,
                speaking_ms: 60_000,
                total_latency_ms: 1_200,
                words_by_day: BTreeMap::new(),
            },
            40,
            date("2026-08-25"),
        );

        assert_eq!(snapshot.time_saved_ms, 120_000);
        assert_eq!(snapshot.speaking_wpm, 120.0);
        assert_eq!(snapshot.average_latency_ms, 600);
    }

    #[test]
    fn counts_a_streak_that_ended_yesterday() {
        let words_by_day = [
            (date("2026-08-22"), 4),
            (date("2026-08-23"), 20),
            (date("2026-08-24"), 18),
        ]
        .into_iter()
        .collect();

        let snapshot = StatsService::calculate(
            StatsInput {
                transcription_count: 3,
                total_words: 42,
                speaking_ms: 20_000,
                total_latency_ms: 1_000,
                words_by_day,
            },
            40,
            date("2026-08-25"),
        );
        assert_eq!(snapshot.streak_days, 3);
    }

    #[test]
    fn empty_stats_do_not_divide_by_zero() {
        let snapshot = StatsService::calculate(
            StatsInput {
                transcription_count: 0,
                total_words: 0,
                speaking_ms: 0,
                total_latency_ms: 0,
                words_by_day: BTreeMap::new(),
            },
            40,
            date("2026-08-25"),
        );
        assert_eq!(snapshot.speaking_wpm, 0.0);
        assert_eq!(snapshot.average_latency_ms, 0);
    }
}
