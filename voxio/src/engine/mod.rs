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
pub(crate) use tap::SampleTap;

/// An audio playback engine capable of handling various formats, providing gapless playback, and a
/// tap for visualization purposes.
pub struct Vox {
    state: Arc<SharedState>,
    commands: channel::Sender<VoxCommand>,
    sps: f64,
    tap: SampleTap,
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
        let tap = SampleTap::new(SAMPLE_TAP_CAPACITY);
        let tap_input = tap.clone();

        let state = Arc::new(SharedState::default());
        let state_clone = Arc::clone(&state);
        let mut last_seek_gen = state_clone.seek_generation();
        let mut was_seeking = false;
        let fade_total_samples = (output_rate as usize * output_channels * SEEK_FADE_MS) / 1000;
        let mut fade_samples_remaining: usize = 0;

        // Callback function
        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _| {
                    let current_gen = state_clone.seek_generation();

                    // Drain ring buffer if seek generation changed
                    if current_gen != last_seek_gen {
                        last_seek_gen = current_gen;
                        while consumer.pop().is_ok() {}
                    }

                    let is_seeking = state_clone.is_seeking();

                    // Output silence if paused, not active, or seeking
                    if state_clone.is_paused() || !state_clone.is_active() || is_seeking {
                        data.fill(0.0);
                        was_seeking = is_seeking;
                        return;
                    }

                    // Start fade-in if we just finished seeking
                    if was_seeking && !is_seeking {
                        fade_samples_remaining = fade_total_samples;
                    }
                    was_seeking = is_seeking;

                    let mut samples_consumed = 0u64;
                    for sample in data.iter_mut() {
                        match consumer.pop() {
                            Ok(mut s) => {
                                // Apply fade-in envelope
                                if fade_samples_remaining > 0 {
                                    let progress = 1.0
                                        - (fade_samples_remaining as f32
                                            / fade_total_samples as f32);
                                    s *= progress;
                                    fade_samples_remaining -= 1;
                                }
                                *sample = s;
                                samples_consumed += 1;
                            }
                            Err(_) => *sample = 0.0,
                        }
                    }

                    if samples_consumed > 0 {
                        state_clone.add_samples(samples_consumed);
                    }

                    tap_input.push(data);
                },
                |err| eprintln!("Stream error: {}", err),
                None,
            )
            .expect("Failed to create stream");

        stream.play().map_err(|e| VoxError::Output(e.to_string()))?;

        let (tx, rx) = channel::bounded(CHANNEL_COUNT);
        let decoder_thread = command::spawn(
            rx,
            producer,
            Arc::clone(&state),
            output_rate as u32,
            output_channels,
        );

        Ok(Self {
            state: state,
            commands: tx,
            sps: output_rate as f64 * output_channels as f64,
            _stream: stream,
            _decoder_thread: decoder_thread,
            tap,
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
        let _ = self.commands
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

    /// Retrieves the latest *amount* of requested samples
    /// Returns Vec<f32>
    pub fn get_latest_samples(&self, amount: usize) -> Vec<f32> {
        self.tap.get_latest(amount)
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
