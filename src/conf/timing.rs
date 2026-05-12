use std::{sync::OnceLock, time::Duration};

pub static TIMING: OnceLock<Timing> = OnceLock::new();

pub struct Timing {
    pub refresh_rate: Duration,
    pub media_tick: u32,
    pub db_tick: u32,
}

impl Timing {
    pub fn from_fps(fps: u8) -> Self {
        let fps = fps.clamp(20, 240);
        Self {
            refresh_rate: frame_period(fps),
            media_tick: ticks_per(48, fps),
            db_tick: ticks_per(600, fps),
        }
    }
}

fn frame_period(fps: u8) -> Duration {
    Duration::from_secs_f64(1.0 / fps as f64)
}

fn ticks_per(interval_ms: u32, fps: u8) -> u32 {
    let frame_ms = 1000.0 / fps as f64;
    ((interval_ms as f64 / frame_ms).round() as u32).max(1) // never 0
}

pub fn timing() -> &'static Timing {
    TIMING.get().expect("Timing failed to initialize")
}
