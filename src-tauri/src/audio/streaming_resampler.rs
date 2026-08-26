use rubato::{Fft, FixedSync, Indexing, Resampler, audioadapter_buffers::owned::InterleavedOwned};

use crate::error::{VoxError, VoxResult};

use super::TRANSCRIPTION_SAMPLE_RATE;

pub struct StreamingResampler {
    input_rate: u32,
    resampler: Option<Fft<f32>>,
    pending: Vec<f32>,
    output_delay_remaining: usize,
    total_input: usize,
    total_output: usize,
}

impl StreamingResampler {
    pub fn new(input_rate: u32) -> VoxResult<Self> {
        if input_rate == 0 {
            return Err(VoxError::Audio(
                "input sample rate cannot be zero".to_owned(),
            ));
        }
        let resampler = if input_rate == TRANSCRIPTION_SAMPLE_RATE {
            None
        } else {
            Some(
                Fft::<f32>::new(
                    input_rate as usize,
                    TRANSCRIPTION_SAMPLE_RATE as usize,
                    1_024,
                    1,
                    FixedSync::Input,
                )
                .map_err(|error| VoxError::Audio(error.to_string()))?,
            )
        };
        let output_delay_remaining = resampler
            .as_ref()
            .map(Resampler::output_delay)
            .unwrap_or_default();
        Ok(Self {
            input_rate,
            resampler,
            pending: Vec::new(),
            output_delay_remaining,
            total_input: 0,
            total_output: 0,
        })
    }

    pub fn push(&mut self, samples: &[f32]) -> VoxResult<Vec<f32>> {
        self.total_input = self.total_input.saturating_add(samples.len());
        if self.resampler.is_none() {
            self.total_output = self.total_output.saturating_add(samples.len());
            return Ok(samples.to_vec());
        }

        self.pending.extend_from_slice(samples);
        let mut output = Vec::new();
        loop {
            let required = self
                .resampler
                .as_ref()
                .expect("resampler exists")
                .input_frames_next();
            if self.pending.len() < required {
                break;
            }
            let input = self.pending.drain(..required).collect::<Vec<_>>();
            output.extend(self.process(&input, None)?);
        }
        self.total_output = self.total_output.saturating_add(output.len());
        Ok(output)
    }

    pub fn finish(&mut self) -> VoxResult<Vec<f32>> {
        if self.resampler.is_none() {
            return Ok(Vec::new());
        }

        let expected_total = ((self.total_input as u128 * u128::from(TRANSCRIPTION_SAMPLE_RATE))
            .div_ceil(u128::from(self.input_rate))) as usize;
        let mut output = Vec::new();
        if !self.pending.is_empty() {
            let required = self
                .resampler
                .as_ref()
                .expect("resampler exists")
                .input_frames_next();
            let partial_length = self.pending.len();
            self.pending.resize(required, 0.0);
            let input = std::mem::take(&mut self.pending);
            let indexing = Indexing::new().partial_len(partial_length);
            output.extend(self.process(&input, Some(&indexing))?);
        }

        let mut flushes = 0;
        while self.total_output + output.len() < expected_total && flushes < 8 {
            let required = self
                .resampler
                .as_ref()
                .expect("resampler exists")
                .input_frames_next();
            let silence = vec![0.0; required];
            let indexing = Indexing::new().partial_len(0);
            output.extend(self.process(&silence, Some(&indexing))?);
            flushes += 1;
        }

        output.truncate(expected_total.saturating_sub(self.total_output));
        self.total_output = self.total_output.saturating_add(output.len());
        Ok(output)
    }

    fn process(&mut self, input: &[f32], indexing: Option<&Indexing>) -> VoxResult<Vec<f32>> {
        let adapter = InterleavedOwned::new_from(input.to_vec(), 1, input.len())
            .map_err(|error| VoxError::Audio(error.to_string()))?;
        let mut output = self
            .resampler
            .as_mut()
            .expect("process is only used when resampling")
            .process(&adapter, indexing)
            .map_err(|error| VoxError::Audio(error.to_string()))?
            .take_data();
        let trim = self.output_delay_remaining.min(output.len());
        output.drain(..trim);
        self.output_delay_remaining -= trim;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;

    use super::*;

    #[test]
    fn resamples_incremental_48khz_chunks_to_whisper_rate() {
        let input = (0..48_000)
            .map(|index| (TAU * 440.0 * index as f32 / 48_000.0).sin())
            .collect::<Vec<_>>();
        let mut resampler = StreamingResampler::new(48_000).unwrap();
        let mut output = Vec::new();
        for chunk in input.chunks(480) {
            output.extend(resampler.push(chunk).unwrap());
        }
        output.extend(resampler.finish().unwrap());
        assert_eq!(output.len(), 16_000);
        assert!(output.iter().all(|sample| sample.is_finite()));
    }

    #[test]
    fn native_rate_streams_without_buffering() {
        let mut resampler = StreamingResampler::new(16_000).unwrap();
        assert_eq!(resampler.push(&[0.1, -0.2]).unwrap(), vec![0.1, -0.2]);
        assert!(resampler.finish().unwrap().is_empty());
    }

    #[test]
    fn rejects_an_invalid_input_rate() {
        assert!(StreamingResampler::new(0).is_err());
    }
}
