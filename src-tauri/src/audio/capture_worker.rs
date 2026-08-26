use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::error::{VoxError, VoxResult};

use super::{LiveAudioBuffer, SampleReader, StreamingResampler};

pub struct CaptureWorker {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    buffer: Arc<LiveAudioBuffer>,
    error: Arc<Mutex<Option<String>>>,
}

impl CaptureWorker {
    pub fn start(
        mut reader: SampleReader,
        input_rate: u32,
        output_capacity: usize,
    ) -> VoxResult<Self> {
        let mut resampler = StreamingResampler::new(input_rate)?;
        let stop = Arc::new(AtomicBool::new(false));
        let buffer = Arc::new(LiveAudioBuffer::new(output_capacity));
        let error = Arc::new(Mutex::new(None));
        let thread_stop = Arc::clone(&stop);
        let thread_buffer = Arc::clone(&buffer);
        let thread_error = Arc::clone(&error);
        let handle = thread::Builder::new()
            .name("vox-audio-resampler".to_owned())
            .spawn(move || {
                loop {
                    let input = reader.drain_available();
                    if !input.is_empty()
                        && let Err(error) = resampler
                            .push(&input)
                            .and_then(|output| thread_buffer.append(&output))
                    {
                        store_error(&thread_error, error);
                        break;
                    }
                    if thread_stop.load(Ordering::Acquire) {
                        if let Err(error) = resampler
                            .finish()
                            .and_then(|output| thread_buffer.append(&output))
                        {
                            store_error(&thread_error, error);
                        }
                        break;
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                let dropped = reader.dropped_samples();
                if dropped > 0 {
                    tracing::warn!(dropped, "live microphone consumer fell behind");
                }
            })
            .map_err(|error| VoxError::Audio(error.to_string()))?;

        Ok(Self {
            stop,
            handle: Some(handle),
            buffer,
            error,
        })
    }

    pub fn snapshot(&self) -> VoxResult<Vec<f32>> {
        self.check_error()?;
        self.buffer.snapshot()
    }

    pub fn finish(mut self) -> VoxResult<Vec<f32>> {
        self.stop_and_join()?;
        let dropped = self.buffer.dropped_samples();
        if dropped > 0 {
            tracing::warn!(dropped, "resampled audio reached its hard limit");
        }
        self.buffer.take()
    }

    pub fn discard(mut self) -> VoxResult<()> {
        self.stop_and_join()
    }

    fn stop_and_join(&mut self) -> VoxResult<()> {
        self.stop.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .map_err(|_| VoxError::Audio("audio resampler thread panicked".to_owned()))?;
        }
        self.check_error()
    }

    fn check_error(&self) -> VoxResult<()> {
        let error = self
            .error
            .lock()
            .map_err(|_| VoxError::Audio("audio worker error lock was poisoned".to_owned()))?;
        match error.as_ref() {
            Some(message) => Err(VoxError::Audio(message.clone())),
            None => Ok(()),
        }
    }
}

fn store_error(target: &Mutex<Option<String>>, error: VoxError) {
    if let Ok(mut slot) = target.lock() {
        *slot = Some(error.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::bounded;

    #[test]
    fn drains_and_retains_audio_until_finish() {
        let (mut writer, reader) = bounded(8);
        let worker = CaptureWorker::start(reader, 16_000, 8).unwrap();
        writer.push(0.1);
        writer.push(0.2);
        writer.push(0.3);
        assert_eq!(worker.finish().unwrap(), vec![0.1, 0.2, 0.3]);
    }
}
