use rubato::{Fft, FixedSync, Resampler, audioadapter_buffers::owned::InterleavedOwned};

use crate::error::{VoxError, VoxResult};

pub const TRANSCRIPTION_SAMPLE_RATE: u32 = 16_000;

pub fn to_transcription_rate(samples: &[f32], input_rate: u32) -> VoxResult<Vec<f32>> {
    if samples.is_empty() {
        return Ok(Vec::new());
    }
    if input_rate == TRANSCRIPTION_SAMPLE_RATE {
        return Ok(samples.to_vec());
    }
    if input_rate == 0 {
        return Err(VoxError::Audio(
            "input sample rate cannot be zero".to_owned(),
        ));
    }

    let input = InterleavedOwned::new_from(samples.to_vec(), 1, samples.len())
        .map_err(|error| VoxError::Audio(error.to_string()))?;
    let mut resampler = Fft::<f32>::new(
        input_rate as usize,
        TRANSCRIPTION_SAMPLE_RATE as usize,
        1_024,
        1,
        FixedSync::Both,
    )
    .map_err(|error| VoxError::Audio(error.to_string()))?;
    let output = resampler
        .process_all(&input, samples.len(), None)
        .map_err(|error| VoxError::Audio(error.to_string()))?;
    Ok(output.take_data())
}

#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;

    use super::*;

    #[test]
    fn resamples_48khz_mono_to_whisper_rate() {
        let samples = (0..48_000)
            .map(|index| (TAU * 440.0 * index as f32 / 48_000.0).sin())
            .collect::<Vec<_>>();
        let output = to_transcription_rate(&samples, 48_000).unwrap();
        assert_eq!(output.len(), 16_000);
        assert!(output.iter().all(|sample| sample.is_finite()));
    }

    #[test]
    fn leaves_native_16khz_audio_untouched() {
        let samples = vec![0.1, -0.2, 0.3];
        assert_eq!(
            to_transcription_rate(&samples, TRANSCRIPTION_SAMPLE_RATE).unwrap(),
            samples
        );
    }
}
