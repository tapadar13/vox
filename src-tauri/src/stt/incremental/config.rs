use crate::audio::TRANSCRIPTION_SAMPLE_RATE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IncrementalConfig {
    pub sample_rate: u32,
    pub chunk_ms: u64,
    pub step_ms: u64,
    pub poll_ms: u64,
}

impl Default for IncrementalConfig {
    fn default() -> Self {
        Self {
            sample_rate: TRANSCRIPTION_SAMPLE_RATE,
            chunk_ms: 4_500,
            step_ms: 3_000,
            poll_ms: 120,
        }
    }
}

impl IncrementalConfig {
    pub fn overlap_ms(self) -> u64 {
        self.chunk_ms.saturating_sub(self.step_ms)
    }

    pub fn samples_for_ms(self, milliseconds: u64) -> usize {
        ((milliseconds * u64::from(self.sample_rate)) / 1_000) as usize
    }

    pub fn ms_for_samples(self, samples: usize) -> u64 {
        samples as u64 * 1_000 / u64::from(self.sample_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_keep_a_context_overlap() {
        let config = IncrementalConfig::default();
        assert_eq!(config.overlap_ms(), 1_500);
        assert!(config.step_ms < config.chunk_ms);
    }

    #[test]
    fn converts_time_to_whisper_samples() {
        let config = IncrementalConfig::default();
        assert_eq!(config.samples_for_ms(1_000), 16_000);
        assert_eq!(config.ms_for_samples(72_000), 4_500);
    }
}
