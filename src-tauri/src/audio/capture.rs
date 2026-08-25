use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use async_trait::async_trait;
use cpal::{
    Device, FromSample, Sample, SampleFormat, SizedSample, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::{
    error::{VoxError, VoxResult},
    ports::{AudioClip, AudioInput},
};

use super::{
    SampleReader, SampleWriter, TRANSCRIPTION_SAMPLE_RATE, bounded, to_transcription_rate,
};

struct ActiveCapture {
    stream: Stream,
    reader: SampleReader,
    input_rate: u32,
}

pub struct CpalAudioInput {
    active: Mutex<Option<ActiveCapture>>,
}

impl CpalAudioInput {
    pub fn new() -> Self {
        Self {
            active: Mutex::new(None),
        }
    }

    fn lock_active(&self) -> VoxResult<std::sync::MutexGuard<'_, Option<ActiveCapture>>> {
        self.active
            .lock()
            .map_err(|_| VoxError::Audio("audio capture lock was poisoned".to_owned()))
    }
}

#[async_trait]
impl AudioInput for CpalAudioInput {
    async fn start(
        &self,
        max_duration: Duration,
        on_level: Arc<dyn Fn(f32) + Send + Sync>,
    ) -> VoxResult<()> {
        let mut active = self.lock_active()?;
        if active.is_some() {
            return Err(VoxError::Audio("capture is already running".to_owned()));
        }

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| VoxError::Audio("no input device is available".to_owned()))?;
        let supported = device
            .default_input_config()
            .map_err(|error| VoxError::Audio(error.to_string()))?;
        let input_rate = supported.sample_rate();
        let channels = usize::from(supported.channels());
        let capacity = (u64::from(input_rate) * max_duration.as_secs())
            .try_into()
            .unwrap_or(usize::MAX);
        let (writer, reader) = bounded(capacity);
        let config: StreamConfig = supported.clone().into();

        let stream = match supported.sample_format() {
            SampleFormat::I8 => build_stream::<i8>(&device, &config, channels, writer, on_level),
            SampleFormat::I16 => build_stream::<i16>(&device, &config, channels, writer, on_level),
            SampleFormat::I32 => build_stream::<i32>(&device, &config, channels, writer, on_level),
            SampleFormat::I64 => build_stream::<i64>(&device, &config, channels, writer, on_level),
            SampleFormat::U8 => build_stream::<u8>(&device, &config, channels, writer, on_level),
            SampleFormat::U16 => build_stream::<u16>(&device, &config, channels, writer, on_level),
            SampleFormat::U32 => build_stream::<u32>(&device, &config, channels, writer, on_level),
            SampleFormat::U64 => build_stream::<u64>(&device, &config, channels, writer, on_level),
            SampleFormat::F32 => build_stream::<f32>(&device, &config, channels, writer, on_level),
            SampleFormat::F64 => build_stream::<f64>(&device, &config, channels, writer, on_level),
            format => Err(VoxError::Audio(format!(
                "unsupported microphone sample format: {format}"
            ))),
        }?;
        stream
            .play()
            .map_err(|error| VoxError::Audio(error.to_string()))?;
        *active = Some(ActiveCapture {
            stream,
            reader,
            input_rate,
        });
        Ok(())
    }

    async fn stop(&self) -> VoxResult<AudioClip> {
        let capture = self
            .lock_active()?
            .take()
            .ok_or_else(|| VoxError::Audio("capture is not running".to_owned()))?;
        drop(capture.stream);
        let dropped = capture.reader.dropped_samples();
        let input = capture.reader.drain();
        if dropped > 0 {
            tracing::warn!(dropped, "microphone ring buffer reached its hard limit");
        }
        let duration_ms = input.len() as u64 * 1_000 / u64::from(capture.input_rate);
        let samples = to_transcription_rate(&input, capture.input_rate)?;
        Ok(AudioClip {
            samples,
            sample_rate: TRANSCRIPTION_SAMPLE_RATE,
            duration_ms,
        })
    }

    async fn discard(&self) -> VoxResult<()> {
        if let Some(capture) = self.lock_active()?.take() {
            drop(capture.stream);
        }
        Ok(())
    }
}

fn build_stream<T>(
    device: &Device,
    config: &StreamConfig,
    channels: usize,
    mut writer: SampleWriter,
    on_level: Arc<dyn Fn(f32) + Send + Sync>,
) -> VoxResult<Stream>
where
    T: SizedSample + Copy,
    f32: FromSample<T>,
{
    let mut last_level = Instant::now() - Duration::from_millis(34);
    device
        .build_input_stream(
            config.clone(),
            move |data: &[T], _| {
                let mut peak = 0.0_f32;
                for frame in data.chunks(channels) {
                    let mono =
                        frame.iter().copied().map(f32::from_sample).sum::<f32>() / channels as f32;
                    peak = peak.max(mono.abs());
                    writer.push(mono);
                }
                if last_level.elapsed() >= Duration::from_millis(33) {
                    on_level(peak.clamp(0.0, 1.0));
                    last_level = Instant::now();
                }
            },
            |error| tracing::error!(%error, "microphone stream failed"),
            None,
        )
        .map_err(|error| VoxError::Audio(error.to_string()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn stereo_downmix_math_is_centered() {
        let stereo = [0.75_f32, -0.25_f32];
        let mono = stereo.iter().sum::<f32>() / stereo.len() as f32;
        assert_eq!(mono, 0.25);
    }
}
