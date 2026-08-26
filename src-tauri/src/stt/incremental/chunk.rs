use std::ops::Range;

use super::IncrementalConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkRange {
    pub start_sample: usize,
    pub end_sample: usize,
}

impl ChunkRange {
    pub fn samples(self) -> Range<usize> {
        self.start_sample..self.end_sample
    }

    pub fn duration_samples(self) -> usize {
        self.end_sample.saturating_sub(self.start_sample)
    }

    pub fn start_ms(self, config: IncrementalConfig) -> u64 {
        config.ms_for_samples(self.start_sample)
    }

    pub fn end_ms(self, config: IncrementalConfig) -> u64 {
        config.ms_for_samples(self.end_sample)
    }
}

#[derive(Debug, Clone)]
pub struct ChunkPlanner {
    config: IncrementalConfig,
    next_start: usize,
}

impl ChunkPlanner {
    pub fn new(config: IncrementalConfig) -> Self {
        Self {
            config,
            next_start: 0,
        }
    }

    pub fn next_ready(&mut self, available_samples: usize) -> Option<ChunkRange> {
        let chunk_samples = self.config.samples_for_ms(self.config.chunk_ms);
        let end_sample = self.next_start.checked_add(chunk_samples)?;
        if available_samples < end_sample {
            return None;
        }
        let range = ChunkRange {
            start_sample: self.next_start,
            end_sample,
        };
        self.next_start = self
            .next_start
            .saturating_add(self.config.samples_for_ms(self.config.step_ms));
        Some(range)
    }

    pub fn next_start(&self) -> usize {
        self.next_start
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waits_for_a_complete_first_window() {
        let config = IncrementalConfig::default();
        let mut planner = ChunkPlanner::new(config);
        assert_eq!(planner.next_ready(71_999), None);
        assert_eq!(
            planner.next_ready(72_000),
            Some(ChunkRange {
                start_sample: 0,
                end_sample: 72_000,
            })
        );
    }

    #[test]
    fn advances_with_overlap() {
        let config = IncrementalConfig::default();
        let mut planner = ChunkPlanner::new(config);
        let first = planner.next_ready(120_000).unwrap();
        let second = planner.next_ready(120_000).unwrap();
        assert_eq!(first.samples(), 0..72_000);
        assert_eq!(second.samples(), 48_000..120_000);
        assert_eq!(first.end_sample - second.start_sample, 24_000);
    }
}
