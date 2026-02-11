use rtrb::{Consumer, Producer, RingBuffer};

pub(crate) struct TapWriter {
    producer: Producer<f32>,
}

pub(crate) struct TapReader {
    consumer: Consumer<f32>,
    capacity: usize,
}

pub(crate) fn new_tap(capacity: usize) -> (TapWriter, TapReader) {
    let (producer, consumer) = RingBuffer::new(capacity);
    (
        TapWriter { producer },
        TapReader { consumer, capacity },
    )
}

impl TapWriter {
    /// Batch-write samples into the tap buffer.
    /// Prefers latest samples — if the buffer can't hold all samples,
    /// older ones are skipped so the visualization stays current.
    pub(crate) fn push(&mut self, samples: &[f32]) {
        let available = self.producer.slots();
        if available == 0 {
            return;
        }
        let to_write = samples.len().min(available);
        let skip = samples.len() - to_write;
        if let Ok(chunk) = self.producer.write_chunk_uninit(to_write) {
            chunk.fill_from_iter(samples[skip..].iter().copied());
        }
    }
}

impl TapReader {
    /// Read up to `amount` of the latest available samples.
    /// Older samples beyond the requested amount are discarded.
    pub(crate) fn get_latest(&mut self, amount: usize) -> Vec<f32> {
        let output_len = amount.min(self.capacity);
        let available = self.consumer.slots();

        if available == 0 {
            return Vec::new();
        }

        // Discard older samples if more are available than requested
        if available > output_len {
            let skip = available - output_len;
            if let Ok(chunk) = self.consumer.read_chunk(skip) {
                chunk.commit_all();
            }
        }

        let to_read = self.consumer.slots().min(output_len);
        if to_read == 0 {
            return Vec::new();
        }

        let mut output = Vec::with_capacity(to_read);
        if let Ok(chunk) = self.consumer.read_chunk(to_read) {
            let (first, second) = chunk.as_slices();
            output.extend_from_slice(first);
            output.extend_from_slice(second);
            chunk.commit_all();
        }

        output
    }
}
