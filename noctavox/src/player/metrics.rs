use crate::TAP_BUFFER_CAPACITY;
use crate::player::PlaybackState;
use crossbeam::queue::ArrayQueue;
use std::sync::atomic::AtomicU32;
use std::time::Duration;
use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicU8, AtomicU64, Ordering},
    },
};

pub struct PlaybackMetrics {
    state: AtomicU8,
    channels: AtomicU8,
    sample_rate: AtomicU32,
    elapsed_ms: AtomicU64,
    pub(crate) audio_tap: ArrayQueue<f32>,
}

impl PlaybackMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(PlaybackMetrics {
            state: AtomicU8::new(0),
            channels: AtomicU8::new(0),
            sample_rate: AtomicU32::new(0),
            elapsed_ms: AtomicU64::new(0),
            audio_tap: ArrayQueue::new(TAP_BUFFER_CAPACITY),
        })
    }

    pub fn get_state(&self) -> PlaybackState {
        self.state
            .load(Ordering::Relaxed)
            .try_into()
            .unwrap_or(PlaybackState::Stopped)
    }

    pub fn get_elapsed(&self) -> Duration {
        Duration::from_millis(self.elapsed_ms.load(Ordering::Relaxed))
    }

    pub fn is_paused(&self) -> bool {
        PlaybackState::Paused == self.get_state()
    }

    pub fn is_stopped(&self) -> bool {
        PlaybackState::Stopped == self.get_state()
    }

    pub fn set_playback_state(&self, state: PlaybackState) {
        self.state.store(state as u8, Ordering::Relaxed);
    }

    pub fn set_elapsed(&self, d: Duration) {
        self.elapsed_ms
            .store(d.as_millis() as u64, Ordering::Relaxed)
    }

    pub fn set_sample_rate(&self, sample_rate: u32) {
        self.sample_rate.store(sample_rate, Ordering::Relaxed);
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate.load(Ordering::Relaxed)
    }

    pub fn set_channels(&self, channels: u8) {
        self.channels.store(channels, Ordering::Relaxed);
    }

    pub fn channels(&self) -> u8 {
        self.channels.load(Ordering::Relaxed)
    }

    pub fn reset(&self) {
        self.set_elapsed(Duration::ZERO);
        self.set_playback_state(PlaybackState::Stopped);
        while let Some(_) = self.audio_tap.pop() {}
    }

    pub fn drain_into(&self, buf: &mut VecDeque<f32>, limit: usize) {
        while let Some(s) = self.audio_tap.pop() {
            buf.push_back(s);

            if buf.len() > limit {
                buf.pop_front();
            }
        }
    }
}
