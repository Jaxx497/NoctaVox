use core::f64;
use crossbeam::channel::Receiver;
use rtrb::Producer;
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    PENDING_CAPACITY, SEEK_PREFILL_MS,
    engine::{SharedState, VoxDecoder, VoxResampler},
    error::{Result, VoxError},
};

pub enum SeekPosition {
    Absolute(f64),
    Relative(f64),
}

pub(crate) enum VoxCommand {
    Play(String),
    QueueNext(String),
    Seek(SeekPosition),
    Stop,
    Shutdown,
}

struct QueuedTrack {
    decoder: VoxDecoder,
    resampler: Option<VoxResampler>,
    input_channels: usize,
}

struct VoxWorker {
    rx: Receiver<VoxCommand>,
    producer: Producer<f32>,
    state: Arc<SharedState>,
    output_rate: u32,
    output_channels: usize,

    current: Option<VoxDecoder>,
    next: Option<QueuedTrack>,
    resampler: Option<VoxResampler>,
    pending: Vec<f32>,
    input_channels: usize,
    deferred_next: Option<String>,
}

pub fn spawn(
    rx: Receiver<VoxCommand>,
    producer: Producer<f32>,
    state: Arc<SharedState>,
    output_rate: u32,
    output_channels: usize,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut worker = VoxWorker::new(rx, producer, state, output_rate, output_channels);
        if let Err(e) = worker.run() {
            eprintln!("Decoder thread error: {}", e);
        }
    })
}

impl VoxWorker {
    fn new(
        rx: Receiver<VoxCommand>,
        producer: Producer<f32>,
        state: Arc<SharedState>,
        output_rate: u32,
        output_channels: usize,
    ) -> Self {
        VoxWorker {
            rx,
            producer,
            state,
            output_rate,
            output_channels,

            current: None,
            next: None,
            resampler: None,
            pending: Vec::with_capacity(PENDING_CAPACITY),
            input_channels: 2,
            deferred_next: None,
        }
    }

    fn run(&mut self) -> Result<()> {
        loop {
            match self.current.is_some() {
                true => {
                    self.poll_commands()?;
                    // Skip decoding while paused to avoid blocking on full buffer.
                    // This prevents deadlock when commands can't be processed.
                    if self.state.is_paused() {
                        thread::sleep(Duration::from_millis(5));
                    } else {
                        self.decode_handler()?;
                    }
                }
                false => self.wait_for_command()?,
            }
        }
    }

    fn handle_command(&mut self, cmd: VoxCommand) -> Result<bool> {
        match cmd {
            VoxCommand::Play(path) => self.handle_play(path)?,
            VoxCommand::QueueNext(next) => self.queue_next(next)?,
            VoxCommand::Stop => self.stop_playback(),
            VoxCommand::Shutdown => return Ok(true),
            _ => (),
        }
        Ok(false)
    }

    fn poll_commands(&mut self) -> Result<()> {
        // Process any deferred queue_next from a previous play cycle
        if let Some(path) = self.deferred_next.take() {
            self.queue_next(path)?;
        }

        let mut pending_seek: Option<f64> = None;
        let mut pending_next: Option<String> = None;
        let mut pending_play: Option<String> = None;

        while let Ok(cmd) = self.rx.try_recv() {
            match cmd {
                VoxCommand::Play(path) => {
                    // Coalesce: only keep the latest play command
                    // New play invalidates any queued next track
                    pending_next = None;
                    pending_play = Some(path);
                }
                VoxCommand::Seek(pos) => {
                    // Coalesce: accumulate relative seeks
                    let current = pending_seek.unwrap_or_else(|| self.get_elapsed());
                    pending_seek = Some(match pos {
                        SeekPosition::Absolute(t) => t,
                        SeekPosition::Relative(delta) => current + delta,
                    });
                }
                VoxCommand::QueueNext(path) => {
                    // Coalesce: only keep the latest queued track
                    // This prevents expensive file opens when shuffling rapidly
                    pending_next = Some(path);
                }
                cmd => {
                    // Flush pending operations before other commands
                    if let Some(path) = pending_play.take() {
                        self.handle_play(path)?;
                    }
                    if let Some(target) = pending_seek.take() {
                        self.handle_seek(target)?;
                    }
                    if let Some(path) = pending_next.take() {
                        self.queue_next(path)?;
                    }
                    if self.handle_command(cmd)? {
                        return Ok(());
                    }
                }
            }
        }

        // Execute final coalesced operations
        if let Some(path) = pending_play {
            self.handle_play(path)?;
            // Defer queue_next so it doesn't block the play path.
            // It will be processed on the next poll_commands iteration,
            // after decode has already started producing samples.
            if let Some(path) = pending_next.take() {
                self.deferred_next = Some(path);
            }
        }
        if let Some(target) = pending_seek {
            self.handle_seek(target)?;
        }
        if let Some(path) = pending_next {
            self.queue_next(path)?;
        }

        Ok(())
    }

    fn wait_for_command(&mut self) -> Result<()> {
        match self.rx.recv() {
            Ok(cmd) => {
                self.handle_command(cmd)?;
                Ok(())
            }
            Err(_) => Err(VoxError::ChannelClosed),
        }
    }

    // ==========================
    //     Decode functions
    // ==========================

    fn decode_handler(&mut self) -> Result<()> {
        if self.current.is_none() {
            return Ok(());
        }
        let packet = {
            let decoder = self.current.as_mut().unwrap();
            decoder.next_packet()?.map(|s| s.to_vec())
        };

        match packet {
            Some(s) => self.process_samples(&s),
            None => self.handle_track_end(),
        }
    }

    fn process_samples(&mut self, samples: &[f32]) -> Result<()> {
        match &mut self.resampler {
            Some(r) => {
                self.pending.extend_from_slice(samples);
                let producer = &mut self.producer;
                let input_ch = self.input_channels;
                let output_ch = self.output_channels;
                r.process(&mut self.pending, |s| {
                    push_samples_mapped(producer, s, input_ch, output_ch);
                })?;
            }
            None => push_samples_mapped(
                &mut self.producer,
                samples,
                self.input_channels,
                self.output_channels,
            ),
        }

        Ok(())
    }

    fn handle_track_end(&mut self) -> Result<()> {
        self.state.signal_track_ended();
        self.state.reset_samples();

        match self.next.take() {
            Some(queued) => self.transition_to(queued),
            None => {
                self.flush_resampler()?;
                self.stop_playback();
                Ok(())
            }
        }
    }

    fn transition_to(&mut self, queued: QueuedTrack) -> Result<()> {
        let same_rate = match (&self.resampler, &queued.resampler) {
            (Some(curr), Some(next)) => curr.input_rate == next.input_rate,
            (None, None) => true,
            _ => false,
        };

        if same_rate {
            self.current = Some(queued.decoder);
            self.input_channels = queued.input_channels;
        } else {
            self.flush_resampler()?;
            self.pending.clear();
            self.resampler = queued.resampler;
            self.current = Some(queued.decoder);
            self.input_channels = queued.input_channels;
        }

        Ok(())
    }

    fn flush_resampler(&mut self) -> Result<()> {
        if let Some(r) = self.resampler.as_mut() {
            let input_ch = self.input_channels;
            let output_ch = self.output_channels;
            r.flush(&mut self.pending, |s| {
                push_samples_mapped(&mut self.producer, s, input_ch, output_ch)
            })?;
        }
        Ok(())
    }

    // ==========================
    //     Playback commands
    // ==========================

    fn handle_play(&mut self, path: String) -> Result<()> {
        self.pending.clear();
        if let Some(r) = self.resampler.as_mut() {
            r.reset();
        }
        self.state.reset_samples();

        match VoxDecoder::open(&path) {
            Ok(decoder) => {
                let info = &decoder.info;

                // Store playable duration (excluding encoder delay/padding)
                if let Some(duration) = decoder.playable_duration() {
                    self.state.set_duration_secs(duration);
                } else {
                    // Fallback to n_frames-based duration if available
                    let fallback = info
                        .n_frames
                        .map(|n| n as f64 / info.sample_rate as f64)
                        .unwrap_or(0.0);
                    self.state.set_duration_secs(fallback);
                }

                self.input_channels = info.channels;
                self.resampler =
                    VoxResampler::new(info.sample_rate, self.output_rate, info.channels)?;
                self.current = Some(decoder);

                self.state.set_active(true);
                self.state.set_paused(false);

                // Prefill the ring buffer before unblocking the audio callback
                let current_generation = self.state.seek_generation();
                self.prefill_after_seek(current_generation)?;
            }
            Err(e) => {
                eprintln!("Failed to open {}: {}", path, e);
                self.state.set_active(false);
            }
        }

        self.state.finish_seek();
        Ok(())
    }

    fn queue_next(&mut self, next: String) -> Result<()> {
        match VoxDecoder::open(&next) {
            Ok(decoder) => {
                let info = &decoder.info;

                let input_channels = info.channels;
                let next_resampler =
                    VoxResampler::new(info.sample_rate, self.output_rate, info.channels)?;
                self.next = Some(QueuedTrack {
                    decoder,
                    resampler: next_resampler,
                    input_channels,
                });
                Ok(())
            }
            Err(_) => Err(VoxError::FileOpen(next)),
        }
    }

    fn handle_seek(&mut self, target_secs: f64) -> Result<()> {
        let decoder = match &mut self.current {
            Some(d) => d,
            None => {
                self.state.finish_seek();
                return Ok(());
            }
        };

        let info = &decoder.info;
        let sample_rate = info.sample_rate;

        let duration = info
            .n_frames
            .map(|n| n as f64 / sample_rate as f64)
            .unwrap_or(f64::MAX);

        if target_secs >= duration {
            self.state.finish_seek();
            return self.handle_track_end();
        }

        let target_secs = target_secs.max(0.0);

        // Seek (returns actual sample position)
        let actual_ts = match decoder.seek(target_secs) {
            Ok(ts) => ts,
            Err(e) => {
                self.state.finish_seek();
                return Err(e);
            }
        };

        // Clear stale audio data
        self.pending.clear();
        if let Some(r) = &mut self.resampler {
            r.reset();
        }

        let input_rate = sample_rate as f64;
        let actual_secs = actual_ts as f64 / input_rate;
        let output_samples =
            (actual_secs * self.output_rate as f64 * self.output_channels as f64) as u64;

        // Update position tracker to actual landed position
        self.state.set_samples(output_samples);

        // Capture current generation before prefilling
        let current_generation = self.state.seek_generation();

        // Prefill buffer for smooth playback (aborts if new seek comes in)
        self.prefill_after_seek(current_generation)?;

        self.state.finish_seek();

        Ok(())
    }

    fn prefill_after_seek(&mut self, seek_generation: u64) -> Result<()> {
        let target_prefill =
            (self.output_rate as usize * self.output_channels * SEEK_PREFILL_MS) / 1000;
        let mut prefilled = 0;

        while prefilled < target_prefill {
            // Abort if a new seek has started
            if self.state.seek_generation() != seek_generation {
                return Ok(());
            }

            let decoder = match &mut self.current {
                Some(d) => d,
                None => break,
            };

            let packet = decoder.next_packet()?.map(|s| s.to_vec());

            match packet {
                Some(samples) => match &mut self.resampler {
                    Some(r) => {
                        self.pending.extend_from_slice(&samples);
                        let producer = &mut self.producer;
                        let input_ch = self.input_channels;
                        let output_ch = self.output_channels;
                        let mut local_prefilled = 0;
                        r.process(&mut self.pending, |s| {
                            local_prefilled +=
                                push_samples_mapped_count(producer, s, input_ch, output_ch);
                        })?;
                        prefilled += local_prefilled;
                    }
                    None => {
                        prefilled += push_samples_mapped_count(
                            &mut self.producer,
                            &samples,
                            self.input_channels,
                            self.output_channels,
                        );
                    }
                },
                None => break,
            }
        }

        Ok(())
    }

    fn stop_playback(&mut self) {
        self.state.reset_samples();
        self.state.set_active(false);
        self.resampler = None;
        self.pending.clear();
        self.current = None;
        self.next = None;
    }

    fn get_elapsed(&self) -> f64 {
        self.state.get_samples() as f64 / (self.output_rate as f64 * self.output_channels as f64)
    }
}

// ==========================
//     Channel Mapping
// ==========================

/// Get a sample from interleaved data with channel mapping.
/// Handles: mono→stereo duplication, channel count mismatch.
#[inline]
fn get_mapped_sample(samples: &[f32], frame: usize, out_ch: usize, input_channels: usize) -> f32 {
    if out_ch < input_channels {
        // Direct mapping: output channel exists in input
        samples[frame * input_channels + out_ch]
    } else if input_channels == 1 {
        // Mono to stereo (or more): duplicate mono channel
        samples[frame * input_channels]
    } else {
        // Missing channel: output silence
        0.0
    }
}

/// Push samples to producer with channel mapping (blocking).
/// Uses batch writes to minimize atomic overhead.
fn push_samples_mapped(
    producer: &mut Producer<f32>,
    samples: &[f32],
    input_channels: usize,
    output_channels: usize,
) {
    let frame_count = samples.len() / input_channels;
    let total_out = frame_count * output_channels;
    let mut written = 0;

    while written < total_out {
        let available = producer.slots();
        if available == 0 {
            thread::sleep(Duration::from_micros(100));
            continue;
        }

        let to_write = (total_out - written).min(available);
        if let Ok(chunk) = producer.write_chunk_uninit(to_write) {
            let start = written;
            chunk.fill_from_iter((start..start + to_write).map(|idx| {
                let frame = idx / output_channels;
                let out_ch = idx % output_channels;
                get_mapped_sample(samples, frame, out_ch, input_channels)
            }));
        }
        written += to_write;
    }
}

/// Push samples to producer with channel mapping, returning count of samples pushed.
/// Non-blocking: writes as many as the buffer can hold.
fn push_samples_mapped_count(
    producer: &mut Producer<f32>,
    samples: &[f32],
    input_channels: usize,
    output_channels: usize,
) -> usize {
    let frame_count = samples.len() / input_channels;
    let total_out = frame_count * output_channels;

    let available = producer.slots();
    if available == 0 {
        return 0;
    }

    let to_write = total_out.min(available);
    if let Ok(chunk) = producer.write_chunk_uninit(to_write) {
        chunk.fill_from_iter((0..to_write).map(|idx| {
            let frame = idx / output_channels;
            let out_ch = idx % output_channels;
            get_mapped_sample(samples, frame, out_ch, input_channels)
        }));
    }

    to_write
}
