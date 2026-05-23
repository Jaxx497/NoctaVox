use anyhow::Context;
use serde::Deserialize;
use std::fs;

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
    pub history_capacity: usize,

    #[serde(default = "defaults::auto_resume")]
    pub auto_resume: bool,

    #[serde(default = "defaults::broadcast")]
    pub broadcast: bool,
}

mod defaults {
    pub fn framerate() -> u16 {
        60
    }

    pub fn history() -> usize {
        64
    }

    pub fn auto_resume() -> bool {
        false
    }

    pub fn broadcast() -> bool {
        false
    }
}

fn deserialize_framerate<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u16, D::Error> {
    u16::deserialize(d).map(|v| v.clamp(20, 360))
}

fn deserialize_history<'de, D: serde::Deserializer<'de>>(d: D) -> Result<usize, D::Error> {
    usize::deserialize(d).map(|v| v.max(1024))
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            framerate: defaults::framerate(),
            history_capacity: defaults::history(),
            auto_resume: defaults::auto_resume(),
            broadcast: defaults::broadcast(),
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
