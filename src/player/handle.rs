use anyhow::Result;
use crossbeam_channel::Receiver;
use std::{sync::Arc, time::Duration};
use voxio::{Vox, VoxEvent, VoxEvents};

pub struct PlayerHandle {
    backend: Arc<Vox>,
    events: VoxEvents,
}

impl PlayerHandle {
    pub fn new(backend: Arc<Vox>, events: VoxEvents) -> Result<Self> {
        Ok(Self { backend, events })
    }
}

// =====================
//    COMMAND HANDLER
// =====================
impl PlayerHandle {
    pub fn play(&self, s: &str) -> Result<()> {
        self.backend.play(s)?;
        Ok(())
    }

    pub fn set_next(&self, song: Option<&str>) -> Result<()> {
        match song {
            Some(s) => self.backend.set_next(s)?,
            None => self.backend.clear_next(),
        }
        Ok(())
    }

    pub fn toggle_playback(&self) {
        match self.backend.is_paused() {
            true => self.backend.resume(),
            false => self.backend.pause(),
        }
    }

    pub fn resume(&self) {
        self.backend.resume();
    }

    pub fn pause(&self) {
        self.backend.pause();
    }

    pub fn stop(&self) {
        self.backend.stop();
    }

    pub fn seek_to(&self, secs: f32) {
        self.backend.seek_to(secs as f64);
    }

    pub fn seek(&self, dur: f64) {
        self.backend.seek_relative(dur as f64);
    }

    fn volume(&self) -> f32 {
        self.backend.volume()
    }

    pub fn set_volume(&self, vol: f32) {
        self.backend.set_volume(vol);
    }

    pub fn adjust_volume(&self, delta: f32) {
        self.backend.set_volume(self.volume() + delta);
    }
}

// ===============
//    ACCESSORS
// ===============

impl PlayerHandle {
    pub fn events(&self) -> &Receiver<VoxEvent> {
        self.events.receiver()
    }

    pub fn elapsed(&self) -> Duration {
        self.backend.position()
    }

    pub fn is_paused(&self) -> bool {
        self.backend.is_paused()
    }

    pub fn is_active(&self) -> bool {
        self.backend.is_active()
    }
}
