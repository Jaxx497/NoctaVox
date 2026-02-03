use crate::{OSCILLO_BUFFER_CAPACITY, player::PlayerBackend};
use anyhow::Result;
use std::{path::Path, time::Duration};

pub struct VoxEngine {
    engine: voxio::Vox,
}

impl VoxEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            engine: voxio::Vox::new()?,
        })
    }
}

impl PlayerBackend for VoxEngine {
    fn play(&mut self, path: &Path) -> Result<()> {
        self.engine.play(path)?;

        Ok(())
    }

    fn pause(&mut self) {
        self.engine.pause();
    }

    fn resume(&mut self) {
        self.engine.resume();
    }

    fn stop(&mut self) {
        let _ = self.engine.stop();
    }

    fn seek_forward(&mut self, secs: u64) -> Result<()> {
        self.engine.seek_relative(secs as f64)?;
        Ok(())
    }

    fn seek_back(&mut self, secs: u64) -> Result<()> {
        // let elapsed = self.engine.position();
        // let new_time = elapsed.saturating_sub(Duration::from_secs(secs));
        self.engine.seek_relative(0.0 - secs as f64)?;
        Ok(())
    }

    fn position(&self) -> Duration {
        self.engine.position()
    }

    fn is_paused(&self) -> bool {
        self.engine.is_paused()
    }

    fn is_stopped(&self) -> bool {
        !self.engine.is_active()
    }

    fn track_ended(&self) -> bool {
        self.engine.track_ended()
    }

    fn supports_gapless(&self) -> bool {
        true
    }

    fn set_next(&mut self, path: &Path) -> Result<()> {
        self.engine.set_next(path)?;
        Ok(())
    }

    fn drain_samples(&mut self) -> Vec<f32> {
        self.engine.get_latest_samples(OSCILLO_BUFFER_CAPACITY)
    }
}
