use std::collections::HashMap;

use super::{ChunkRange, IncrementalConfig, merge_text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimedText {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkTranscript {
    pub range: ChunkRange,
    pub text: String,
    pub language: String,
    pub segments: Vec<TimedText>,
}

#[derive(Debug, Clone)]
pub struct IncrementalTranscript {
    config: IncrementalConfig,
    stable_text: String,
    provisional_text: String,
    committed_until_ms: u64,
    processed_until_ms: u64,
    language_votes: HashMap<String, usize>,
}

impl IncrementalTranscript {
    pub fn new(config: IncrementalConfig) -> Self {
        Self {
            config,
            stable_text: String::new(),
            provisional_text: String::new(),
            committed_until_ms: 0,
            processed_until_ms: 0,
            language_votes: HashMap::new(),
        }
    }

    pub fn ingest(&mut self, chunk: ChunkTranscript) {
        let chunk_start_ms = chunk.range.start_ms(self.config);
        let commit_before_ms = chunk
            .range
            .end_ms(self.config)
            .saturating_sub(self.config.overlap_ms());
        for segment in &chunk.segments {
            let global_end_ms = chunk_start_ms.saturating_add(segment.end_ms);
            if global_end_ms <= commit_before_ms && global_end_ms > self.committed_until_ms {
                self.stable_text = merge_text(&self.stable_text, &segment.text);
                self.committed_until_ms = self.committed_until_ms.max(global_end_ms);
            }
        }
        if chunk.segments.is_empty() {
            self.stable_text = merge_text(&self.stable_text, &chunk.text);
            self.committed_until_ms = self.committed_until_ms.max(commit_before_ms);
        }
        self.provisional_text = chunk.text.trim().to_owned();
        self.processed_until_ms = self.processed_until_ms.max(commit_before_ms);
        if !chunk.language.trim().is_empty() {
            *self.language_votes.entry(chunk.language).or_default() += 1;
        }
    }

    pub fn finish(&mut self, chunk: ChunkTranscript) -> String {
        let chunk_start_ms = chunk.range.start_ms(self.config);
        if chunk.segments.is_empty() {
            self.stable_text = merge_text(&self.stable_text, &chunk.text);
        } else {
            for segment in &chunk.segments {
                let global_end_ms = chunk_start_ms.saturating_add(segment.end_ms);
                if global_end_ms > self.committed_until_ms {
                    self.stable_text = merge_text(&self.stable_text, &segment.text);
                    self.committed_until_ms = self.committed_until_ms.max(global_end_ms);
                }
            }
        }
        if !chunk.language.trim().is_empty() {
            *self.language_votes.entry(chunk.language).or_default() += 1;
        }
        self.provisional_text.clear();
        self.stable_text.trim().to_owned()
    }

    pub fn stable_text(&self) -> &str {
        &self.stable_text
    }

    pub fn live_text(&self) -> String {
        merge_text(&self.stable_text, &self.provisional_text)
    }

    pub fn processed_until_ms(&self) -> u64 {
        self.processed_until_ms
    }

    pub fn final_tail_start_ms(&self) -> u64 {
        self.processed_until_ms
            .saturating_sub(self.config.overlap_ms())
    }

    pub fn language(&self) -> String {
        self.language_votes
            .iter()
            .max_by(
                |(left_language, left_count), (right_language, right_count)| {
                    left_count
                        .cmp(right_count)
                        .then_with(|| right_language.cmp(left_language))
                },
            )
            .map(|(language, _)| language.clone())
            .unwrap_or_else(|| "und".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(start_ms: u64, end_ms: u64) -> ChunkRange {
        let config = IncrementalConfig::default();
        ChunkRange {
            start_sample: config.samples_for_ms(start_ms),
            end_sample: config.samples_for_ms(end_ms),
        }
    }

    #[test]
    fn commits_only_segments_behind_the_overlap() {
        let mut transcript = IncrementalTranscript::new(IncrementalConfig::default());
        transcript.ingest(ChunkTranscript {
            range: range(0, 4_500),
            text: "one two three four".to_owned(),
            language: "en".to_owned(),
            segments: vec![
                TimedText {
                    text: "one two".to_owned(),
                    start_ms: 0,
                    end_ms: 2_000,
                },
                TimedText {
                    text: "three four".to_owned(),
                    start_ms: 2_000,
                    end_ms: 4_400,
                },
            ],
        });
        assert_eq!(transcript.stable_text(), "one two");
        assert_eq!(transcript.live_text(), "one two three four");
        assert_eq!(transcript.processed_until_ms(), 3_000);
    }

    #[test]
    fn final_tail_adds_only_new_timestamped_segments() {
        let mut transcript = IncrementalTranscript::new(IncrementalConfig::default());
        transcript.ingest(ChunkTranscript {
            range: range(0, 4_500),
            text: "one two three".to_owned(),
            language: "en".to_owned(),
            segments: vec![TimedText {
                text: "one two".to_owned(),
                start_ms: 0,
                end_ms: 2_000,
            }],
        });
        let final_text = transcript.finish(ChunkTranscript {
            range: range(1_500, 5_500),
            text: "two three four".to_owned(),
            language: "en".to_owned(),
            segments: vec![
                TimedText {
                    text: "two".to_owned(),
                    start_ms: 0,
                    end_ms: 1_000,
                },
                TimedText {
                    text: "three four".to_owned(),
                    start_ms: 1_000,
                    end_ms: 4_000,
                },
            ],
        });
        assert_eq!(final_text, "one two three four");
    }

    #[test]
    fn final_tail_is_bounded_by_the_overlap() {
        let mut transcript = IncrementalTranscript::new(IncrementalConfig::default());
        transcript.ingest(ChunkTranscript {
            range: range(3_000, 7_500),
            text: "later phrase".to_owned(),
            language: "en".to_owned(),
            segments: vec![],
        });
        assert_eq!(transcript.final_tail_start_ms(), 4_500);
    }

    #[test]
    fn final_pass_keeps_a_segment_crossing_the_stability_boundary() {
        let mut transcript = IncrementalTranscript::new(IncrementalConfig::default());
        transcript.ingest(ChunkTranscript {
            range: range(0, 4_500),
            text: "first crossing words".to_owned(),
            language: "en".to_owned(),
            segments: vec![
                TimedText {
                    text: "first".to_owned(),
                    start_ms: 0,
                    end_ms: 2_000,
                },
                TimedText {
                    text: "crossing words".to_owned(),
                    start_ms: 2_000,
                    end_ms: 4_000,
                },
            ],
        });
        let final_text = transcript.finish(ChunkTranscript {
            range: range(1_500, 5_000),
            text: "crossing words remain".to_owned(),
            language: "en".to_owned(),
            segments: vec![TimedText {
                text: "crossing words remain".to_owned(),
                start_ms: 500,
                end_ms: 3_500,
            }],
        });
        assert_eq!(final_text, "first crossing words remain");
    }
}
