use std::{path::Path, sync::Arc, thread::JoinHandle, time::Duration};

use cpal::{
    Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use crossbeam::channel;

use crate::{
    BUFFER_MS, CHANNEL_COUNT, SAMPLE_TAP_CAPACITY, SEEK_FADE_MS,
    engine::command::SeekPosition,
    error::{Result, VoxError},
};

mod command;
mod decoder;
mod resampler;
mod state;
mod tap;

pub(crate) use command::VoxCommand;
pub(crate) use decoder::VoxDecoder;
pub(crate) use resampler::VoxResampler;
pub(crate) use state::SharedState;
pub(crate) use tap::TapReader;

/// An audio playback engine capable of handling various formats, providing gapless playback, and a
/// tap for visualization purposes.
pub struct Vox {
    state: Arc<SharedState>,
    commands: channel::Sender<VoxCommand>,
    sps: f64,
    output_rate: u32,
    output_channels: usize,
    tap: TapReader,
    _stream: Stream,
    _decoder_thread: JoinHandle<()>,
}

impl Vox {
    /// Create a new Vox object with device defaults
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .ok_or_else(|| VoxError::Output("No output device recognized!".into()))?;

        let config = device
            .default_output_config()
            .map_err(|e| VoxError::Output(e.to_string()))?;

        let output_rate = config.sample_rate() as f32;
        let output_channels = config.channels() as usize;
        let stream_config: StreamConfig = config.into();

        let buffer_size = (output_rate as usize * output_channels * BUFFER_MS) / 1000;
        let (producer, mut consumer) = rtrb::RingBuffer::new(buffer_size);
        let (mut tap_writer, tap_reader) = tap::new_tap(SAMPLE_TAP_CAPACITY);

        let state = Arc::new(SharedState::default());
        let state_clone = Arc::clone(&state);
        let mut was_seeking = false;
        let fade_total_samples = (output_rate as usize * output_channels * SEEK_FADE_MS) / 1000;
        let mut fade_samples_remaining: usize = 0;

        // Callback function
        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _| {
                    let is_seeking = state_clone.is_seeking();
                    let is_inactive =
                        state_clone.is_paused() || !state_clone.is_active() || is_seeking;

                    // Output silence if paused, not active, or seeking
                    if is_inactive {
                        // Drain ring buffer in bulk
                        let available = consumer.slots();
                        if available > 0 {
                            if let Ok(chunk) = consumer.read_chunk(available) {
                                chunk.commit_all();
                            }
                        }
                        data.fill(0.0);
                        was_seeking = is_seeking;
                        return;
                    }

                    // Start fade-in if we just finished seeking
                    if was_seeking && !is_seeking {
                        fade_samples_remaining = fade_total_samples;
                    }
                    was_seeking = is_seeking;

                    // Batch read from ring buffer
                    let available = consumer.slots();
                    let to_read = available.min(data.len());

                    if to_read > 0 {
                        if let Ok(chunk) = consumer.read_chunk(to_read) {
                            let (first, second) = chunk.as_slices();
                            let mut i = 0;
                            for &s in first.iter().chain(second.iter()) {
                                let mut sample = s;
                                if fade_samples_remaining > 0 {
                                    let progress = 1.0
                                        - (fade_samples_remaining as f32
                                            / fade_total_samples as f32);
                                    sample *= progress;
                                    fade_samples_remaining -= 1;
                                }
                                data[i] = sample;
                                i += 1;
                            }
                            chunk.commit_all();
                        }
                    }

                    // Fill remainder with silence (underrun)
                    data[to_read..].fill(0.0);

                    if to_read > 0 {
                        state_clone.add_samples(to_read as u64);
                    }

                    tap_writer.push(data);
                },
                |_e| {},
                None,
            )
            .expect("Failed to create stream");

        let (tx, rx) = channel::bounded(CHANNEL_COUNT);
        let decoder_thread = command::spawn(
            rx,
            producer,
            Arc::clone(&state),
            output_rate as u32,
            output_channels,
        );

        stream.play().map_err(|e| VoxError::Output(e.to_string()))?;

        Ok(Self {
            state: state,
            commands: tx,
            sps: output_rate as f64 * output_channels as f64,
            output_rate: output_rate as u32,
            output_channels,
            _stream: stream,
            _decoder_thread: decoder_thread,
            tap: tap_reader,
        })
    }

    // ===========================
    //     PUBLIC FACING API
    // =========================

    /// Play an audio file, provided a filepath
    pub fn play<P: AsRef<Path>>(&mut self, p: P) -> Result<()> {
        let path = p.as_ref();
        if !path.exists() {
            let s = path.to_string_lossy().to_string();
            return Err(VoxError::FileOpen(s));
        }

        self.state.start_seek();
        self.state.reset_samples();
        self.state.set_active(true);

        self.commands
            .send(VoxCommand::Play(path.to_string_lossy().to_string()))
            .map_err(|_| VoxError::Output("Channel closed".into()))
    }

    /// Set the next track to be played for gapless playback, provided a filepath.
    ///
    /// This is **NOT** a queueing function. Calling this function will overwrite the last
    /// value that was passed to it. Once a transition takes place, the *next* track will
    /// be set to None, and it will be safe to call this method without overwriting the
    /// previously passed value.
    ///
    /// This method uses non-blocking send - if the command channel is full, the command
    /// is dropped. This is safe because QueueNext commands are coalesced by the worker,
    /// so only the most recent one matters.
    pub fn set_next<P: AsRef<Path>>(&mut self, p: P) -> Result<()> {
        let path = p.as_ref();

        if !path.exists() {
            return Err(VoxError::FileOpen(path.to_string_lossy().to_string()));
        }

        // Use try_send to avoid blocking if channel is full.
        // Dropped commands are OK - we coalesce QueueNext and only the last one matters.
        let _ = self
            .commands
            .try_send(VoxCommand::QueueNext(path.to_string_lossy().to_string()));
        Ok(())
    }

    pub fn seek_to(&mut self, pos: f64) -> Result<()> {
        if !self.state.is_active() {
            return Ok(());
        }

        self.state.start_seek();
        self.commands
            .send(VoxCommand::Seek(SeekPosition::Absolute(pos)))
            .map_err(|e| VoxError::Seek(e.to_string()))?;
        Ok(())
    }

    pub fn seek_relative(&mut self, increment: f64) -> Result<()> {
        if !self.state.is_active() {
            return Ok(());
        }

        self.state.start_seek();
        self.commands
            .send(VoxCommand::Seek(SeekPosition::Relative(increment)))
            .map_err(|e| VoxError::Seek(e.to_string()))?;

        Ok(())
    }

    /// *Toggle playback status*
    ///
    /// If playing, pause the playback.
    /// If paused, resume the playback.
    pub fn toggle_playback(&self) {
        self.state.toggle_playback();
    }

    pub fn is_paused(&self) -> bool {
        self.state.is_paused()
    }

    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    /// Pause playback
    pub fn pause(&self) {
        self.state.set_paused(true)
    }

    /// Resume playback if paused
    pub fn resume(&self) {
        self.state.set_paused(false);
    }

    /// Stop all playback, current and upcoming
    pub fn stop(&self) -> Result<()> {
        self.commands
            .send(VoxCommand::Stop)
            .map_err(|_| VoxError::Output("Channel closed".into()))
    }

    /// Retrieve position of playback
    pub fn position(&self) -> Duration {
        Duration::from_secs_f64(self.state.get_samples() as f64 / self.sps)
    }

    /// Retrieve the playable duration of the current track.
    /// This excludes encoder delay and padding samples for accurate progress tracking.
    pub fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.state.get_duration_secs())
    }

    /// Retrieves the latest *amount* of requested samples
    /// Returns Vec<f32>
    pub fn get_latest_samples(&mut self, amount: usize) -> Vec<f32> {
        self.tap.get_latest(amount)
    }

    /// Returns the output sample rate of the audio device in Hz
    pub fn sample_rate(&self) -> u32 {
        self.output_rate
    }

    /// Returns the number of interleaved channels in the audio output
    pub fn channels(&self) -> usize {
        self.output_channels
    }

    pub fn track_ended(&self) -> bool {
        self.state.take_track_ended()
    }
}

impl Drop for Vox {
    fn drop(&mut self) {
        let _ = self.commands.send(VoxCommand::Shutdown);
    }
}
