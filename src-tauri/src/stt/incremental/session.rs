use crate::ports::{AudioClip, Transcript};

use super::{
    ChunkPlanner, ChunkRange, ChunkTranscript, IncrementalConfig, IncrementalTranscript, TimedText,
};

#[derive(Debug, Clone, PartialEq)]
pub struct PlannedChunk {
    pub range: ChunkRange,
    pub audio: AudioClip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveTranscript {
    pub text: String,
    pub stable_text: String,
    pub stable_words: usize,
    pub processed_until_ms: u64,
}

pub struct IncrementalSession {
    config: IncrementalConfig,
    planner: ChunkPlanner,
    transcript: IncrementalTranscript,
    failed: bool,
}

impl IncrementalSession {
    pub fn new(config: IncrementalConfig) -> Self {
        Self {
            config,
            planner: ChunkPlanner::new(config),
            transcript: IncrementalTranscript::new(config),
            failed: false,
        }
    }

    pub fn next_chunk(&mut self, audio: &AudioClip) -> Option<PlannedChunk> {
        if self.failed || audio.sample_rate != self.config.sample_rate {
            return None;
        }
        let range = self.planner.next_ready(audio.samples.len())?;
        Some(self.clip(range, audio))
    }

    pub fn ingest(&mut self, range: ChunkRange, result: Transcript) -> LiveTranscript {
        self.transcript.ingest(to_chunk(range, result));
        self.live()
    }

    pub fn mark_failed(&mut self) {
        self.failed = true;
    }

    pub fn final_chunk(&self, audio: &AudioClip) -> PlannedChunk {
        let start_sample = self
            .config
            .samples_for_ms(self.transcript.final_tail_start_ms())
            .min(audio.samples.len());
        self.clip(
            ChunkRange {
                start_sample,
                end_sample: audio.samples.len(),
            },
            audio,
        )
    }

    pub fn finish(&mut self, range: ChunkRange, result: Transcript) -> Transcript {
        let segments = result.segments.clone();
        let final_text = self.transcript.finish(to_chunk(range, result));
        Transcript {
            text: final_text,
            language: self.transcript.language(),
            segments,
        }
    }

    pub fn live(&self) -> LiveTranscript {
        let stable_text = self.transcript.stable_text().to_owned();
        LiveTranscript {
            stable_words: stable_text.split_whitespace().count(),
            text: self.transcript.live_text(),
            stable_text,
            processed_until_ms: self.transcript.processed_until_ms(),
        }
    }

    pub fn failed(&self) -> bool {
        self.failed
    }

    fn clip(&self, range: ChunkRange, audio: &AudioClip) -> PlannedChunk {
        let samples = audio.samples[range.samples()].to_vec();
        PlannedChunk {
            range,
            audio: AudioClip {
                duration_ms: self.config.ms_for_samples(samples.len()),
                samples,
                sample_rate: audio.sample_rate,
            },
        }
    }
}

fn to_chunk(range: ChunkRange, transcript: Transcript) -> ChunkTranscript {
    ChunkTranscript {
        range,
        text: transcript.text,
        language: transcript.language,
        segments: transcript
            .segments
            .into_iter()
            .map(|segment| TimedText {
                text: segment.text,
                start_ms: segment.start_ms,
                end_ms: segment.end_ms,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::TranscriptSegment;

    fn audio(milliseconds: u64) -> AudioClip {
        AudioClip {
            samples: vec![0.0; (milliseconds * 16) as usize],
            sample_rate: 16_000,
            duration_ms: milliseconds,
        }
    }

    fn transcript(text: &str, end_ms: u64) -> Transcript {
        Transcript {
            text: text.to_owned(),
            language: "en".to_owned(),
            segments: vec![TranscriptSegment {
                text: text.to_owned(),
                start_ms: 0,
                end_ms,
            }],
        }
    }

    #[test]
    fn yields_complete_overlapping_chunks_only() {
        let mut session = IncrementalSession::new(IncrementalConfig::default());
        assert!(session.next_chunk(&audio(4_499)).is_none());
        let first = session.next_chunk(&audio(4_500)).unwrap();
        assert_eq!(first.range.start_sample, 0);
        assert_eq!(first.audio.duration_ms, 4_500);
    }

    #[test]
    fn bounds_final_work_to_the_unstable_tail() {
        let mut session = IncrementalSession::new(IncrementalConfig::default());
        let chunk = session.next_chunk(&audio(4_500)).unwrap();
        session.ingest(chunk.range, transcript("stable words", 2_500));
        let final_chunk = session.final_chunk(&audio(6_000));
        assert_eq!(final_chunk.range.start_sample, 24_000);
        assert_eq!(final_chunk.audio.duration_ms, 4_500);
    }

    #[test]
    fn failure_stops_background_chunks_but_preserves_final_recovery() {
        let mut session = IncrementalSession::new(IncrementalConfig::default());
        session.mark_failed();
        assert!(session.failed());
        assert!(session.next_chunk(&audio(10_000)).is_none());
        assert_eq!(session.final_chunk(&audio(10_000)).range.start_sample, 0);
    }
}
