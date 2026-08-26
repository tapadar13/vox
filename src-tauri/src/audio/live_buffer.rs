use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

use crate::error::{VoxError, VoxResult};

pub struct LiveAudioBuffer {
    samples: Mutex<Vec<f32>>,
    capacity: usize,
    dropped: AtomicU64,
}

impl LiveAudioBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: Mutex::new(Vec::with_capacity(capacity)),
            capacity,
            dropped: AtomicU64::new(0),
        }
    }

    pub fn append(&self, samples: &[f32]) -> VoxResult<()> {
        let mut target = self
            .samples
            .lock()
            .map_err(|_| VoxError::Audio("live audio buffer lock was poisoned".to_owned()))?;
        let available = self.capacity.saturating_sub(target.len());
        let accepted = available.min(samples.len());
        target.extend_from_slice(&samples[..accepted]);
        self.dropped
            .fetch_add((samples.len() - accepted) as u64, Ordering::Relaxed);
        Ok(())
    }

    pub fn snapshot(&self) -> VoxResult<Vec<f32>> {
        self.samples
            .lock()
            .map(|samples| samples.clone())
            .map_err(|_| VoxError::Audio("live audio buffer lock was poisoned".to_owned()))
    }

    pub fn take(&self) -> VoxResult<Vec<f32>> {
        self.samples
            .lock()
            .map(|mut samples| std::mem::take(&mut *samples))
            .map_err(|_| VoxError::Audio("live audio buffer lock was poisoned".to_owned()))
    }

    pub fn dropped_samples(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshots_without_consuming_live_audio() {
        let buffer = LiveAudioBuffer::new(4);
        buffer.append(&[0.1, 0.2]).unwrap();
        assert_eq!(buffer.snapshot().unwrap(), vec![0.1, 0.2]);
        assert_eq!(buffer.snapshot().unwrap(), vec![0.1, 0.2]);
        assert_eq!(buffer.take().unwrap(), vec![0.1, 0.2]);
        assert!(buffer.snapshot().unwrap().is_empty());
    }

    #[test]
    fn caps_memory_and_reports_dropped_samples() {
        let buffer = LiveAudioBuffer::new(2);
        buffer.append(&[0.1, 0.2, 0.3]).unwrap();
        assert_eq!(buffer.snapshot().unwrap(), vec![0.1, 0.2]);
        assert_eq!(buffer.dropped_samples(), 1);
    }
}
