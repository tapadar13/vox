use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use ringbuf::{
    HeapCons, HeapProd, HeapRb,
    traits::{Consumer, Observer, Producer, Split},
};

pub struct SampleWriter {
    producer: HeapProd<f32>,
    dropped: Arc<AtomicU64>,
}

pub struct SampleReader {
    consumer: HeapCons<f32>,
    dropped: Arc<AtomicU64>,
}

pub fn bounded(capacity: usize) -> (SampleWriter, SampleReader) {
    let buffer = HeapRb::<f32>::new(capacity.max(1));
    let (producer, consumer) = buffer.split();
    let dropped = Arc::new(AtomicU64::new(0));
    (
        SampleWriter {
            producer,
            dropped: Arc::clone(&dropped),
        },
        SampleReader { consumer, dropped },
    )
}

impl SampleWriter {
    pub fn push(&mut self, sample: f32) {
        if self.producer.try_push(sample).is_err() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl SampleReader {
    pub fn drain(mut self) -> Vec<f32> {
        let mut samples = Vec::with_capacity(self.consumer.occupied_len());
        while let Some(sample) = self.consumer.try_pop() {
            samples.push(sample);
        }
        samples
    }

    pub fn dropped_samples(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_audio_bounded_and_reports_overflow() {
        let (mut writer, reader) = bounded(2);
        writer.push(0.1);
        writer.push(0.2);
        writer.push(0.3);

        assert_eq!(reader.dropped_samples(), 1);
        assert_eq!(reader.drain(), vec![0.1, 0.2]);
    }
}
