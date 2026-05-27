use crate::{
    config::timing,
    player::{
        PlaybackMetrics, PlaybackState, PlayerBackend, PlayerCommand, PlayerEvent,
        track::NoctavoxTrack,
    },
};
use crossbeam::channel::{Receiver, Sender, TryRecvError};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

pub struct PlayerCore {
    backend: Box<dyn PlayerBackend>,
    commands: Receiver<PlayerCommand>,
    events: Sender<PlayerEvent>,
    metrics: Arc<PlaybackMetrics>,

    current: Option<NoctavoxTrack>,
    next: Option<NoctavoxTrack>,
}

impl PlayerCore {
    pub fn spawn(
        backend: Box<dyn PlayerBackend>,
        commands: Receiver<PlayerCommand>,
        events: Sender<PlayerEvent>,
        metrics: Arc<PlaybackMetrics>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut core = PlayerCore {
                backend,
                commands,
                events,
                metrics,

                current: None,
                next: None,
            };

            core.run();
        })
    }

    fn run(&mut self) {
        self.metrics.set_sample_rate(self.backend.sample_rate());
        loop {
            if !self.process_commands() {
                break;
            }
            self.process_commands();
            self.check_track_end();
            self.update_metrics();
            thread::sleep(timing().refresh_rate);
        }
    }

    fn process_commands(&mut self) -> bool {
        loop {
            match self.commands.try_recv() {
                Ok(cmd) => match cmd {
                    PlayerCommand::Play(s) => self.play_song(s),
                    PlayerCommand::SetNext(s) => self.set_next(s),
                    PlayerCommand::ClearNext => self.clear_next(),
                    PlayerCommand::TogglePlayback => self.toggle_playback(),
                    PlayerCommand::Resume => self.resume(),
                    PlayerCommand::Pause => self.pause(),
                    PlayerCommand::Stop => self.stop(),
                    PlayerCommand::SeekTo(x) => self.seek_to(x),
                    PlayerCommand::SeekForward(x) => self.seek_forward(x),
                    PlayerCommand::SeekBack(x) => self.seek_back(x),
                },
                Err(TryRecvError::Empty) => return true,
                Err(TryRecvError::Disconnected) => return false,
            }
        }
    }

    fn check_track_end(&mut self) {
        // Checking status of `current` ensures the stop event is only sent once
        if self.backend.track_ended() && self.current.is_some() {
            match self.next.take() {
                // GAPLESS BRANCH
                Some(next) => {
                    self.current = Some(next.clone());
                    self.emit(PlayerEvent::TrackStarted((next, true)));
                }
                // STANDARD BRANCH
                None => {
                    self.current = None;
                    self.metrics.set_playback_state(PlaybackState::Stopped);
                    self.emit(PlayerEvent::PlaybackStopped);
                }
            }
        }
    }

    fn update_metrics(&mut self) {
        if self.current.is_some() {
            self.metrics.set_elapsed(self.backend.position())
        }
        self.tap_samples();
    }

    fn tap_samples(&mut self) {
        let samples = self.backend.drain_samples();
        for s in samples {
            let _ = self.metrics.audio_tap.force_push(s);
        }
    }

    fn play_song(&mut self, song: NoctavoxTrack) {
        if let Err(e) = self.backend.play(&song.path()) {
            self.emit(PlayerEvent::Error(e.to_string()));
            return;
        }

        self.current = Some(song.clone());
        self.metrics.set_playback_state(PlaybackState::Playing);
        self.metrics.set_channels(self.backend.channels() as u8);
        self.emit(PlayerEvent::TrackStarted((song, false)));
    }

    fn set_next(&mut self, next: Option<NoctavoxTrack>) {
        if self.backend.supports_gapless() {
            if let Some(song) = &next {
                if let Err(e) = self.backend.set_next(&song.path()) {
                    self.emit(PlayerEvent::Error(e.to_string()));
                    return;
                }
            }

            self.next = next;
        }
    }

    fn clear_next(&mut self) {
        self.next = None
    }

    fn toggle_playback(&mut self) {
        match self.backend.is_paused() {
            true => self.resume(),
            false => self.pause(),
        }
    }

    fn resume(&mut self) {
        if self.backend.is_paused() {
            self.backend.resume();
            self.metrics.set_playback_state(PlaybackState::Playing);
            self.emit(PlayerEvent::StateChanged(PlaybackState::Playing));
        }
    }

    fn pause(&mut self) {
        if !self.backend.is_paused() && !self.backend.is_stopped() {
            self.backend.pause();
            self.metrics.set_playback_state(PlaybackState::Paused);
            self.emit(PlayerEvent::StateChanged(PlaybackState::Paused));
        }
    }

    fn stop(&mut self) {
        self.backend.stop();
        self.current = None;
        self.metrics.reset();
        self.emit(PlayerEvent::PlaybackStopped);
    }

    fn seek_to(&mut self, secs: f32) {
        if !self.backend.is_stopped() {
            let _ = self.backend.seek_to(secs);
        }
    }

    fn seek_forward(&mut self, secs: u64) {
        if !self.backend.is_stopped() {
            let _ = self.backend.seek_forward(secs);
        }
    }

    fn seek_back(&mut self, secs: u64) {
        if !self.backend.is_stopped() {
            if let Err(e) = self.backend.seek_back(secs) {
                self.emit(PlayerEvent::Error(e.to_string()));
            }
        }
    }

    fn emit(&self, event: PlayerEvent) {
        let _ = self.events.send(event);
    }
}
