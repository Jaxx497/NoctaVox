use anyhow::Context;
use serde::Deserialize;
use std::fs;
use voxio::ReplayGainMode;

use crate::CONFIG_DIR;

#[derive(serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UserConfig {
    #[serde(
        default = "defaults::framerate",
        deserialize_with = "deserialize_framerate"
    )]
    pub framerate: u16,

    #[serde(
        default = "defaults::history",
        deserialize_with = "deserialize_history"
    )]
    pub history_capacity: u32,

    #[serde(default = "defaults::update_on_start")]
    pub update_on_start: bool,

    #[serde(default = "defaults::auto_resume")]
    pub auto_resume: bool,

    #[serde(
        default = "defaults::replay_gain",
        deserialize_with = "deserialize_replay_gain"
    )]
    pub replay_gain: ReplayGainMode,

    #[serde(default = "defaults::broadcast")]
    pub broadcast: bool,
}

mod defaults {
    use voxio::ReplayGainMode;

    pub fn framerate() -> u16 {
        60
    }

    pub fn history() -> u32 {
        64
    }

    pub fn update_on_start() -> bool {
        true
    }

    pub fn auto_resume() -> bool {
        false
    }
    pub fn broadcast() -> bool {
        false
    }

    pub fn replay_gain() -> ReplayGainMode {
        ReplayGainMode::Off
    }
}

fn deserialize_framerate<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u16, D::Error> {
    u16::deserialize(d).map(|v| v.clamp(20, 360))
}

fn deserialize_history<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
    u32::deserialize(d).map(|v| v.clamp(8, 1024))
}

fn deserialize_replay_gain<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<ReplayGainMode, D::Error> {
    String::deserialize(d).map(|s| match s.to_lowercase().as_str() {
        "album" => ReplayGainMode::Album,
        "track" => ReplayGainMode::Track,
        _ => ReplayGainMode::Off,
    })
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            framerate: defaults::framerate(),
            history_capacity: defaults::history(),
            update_on_start: defaults::update_on_start(),
            auto_resume: defaults::auto_resume(),
            broadcast: defaults::broadcast(),
            replay_gain: ReplayGainMode::Off,
        }
    }
}

impl UserConfig {
    pub fn load() -> anyhow::Result<Self> {
        let path = CONFIG_DIR.join("config.toml");
        match fs::read_to_string(&path) {
            Ok(s) => {
                toml::from_str(&s).with_context(|| format!("Failed to parse {}", path.display()))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).with_context(|| format!("Failed to parse {}", path.display())),
        }
    }
}
