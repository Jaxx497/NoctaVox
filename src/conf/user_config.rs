use anyhow::Context;
use std::fs;

use crate::CONFIG_DIR;

#[derive(serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UserConfig {
    #[serde(default = "defaults::framerate")]
    pub framerate: u8,

    #[serde(default = "defaults::auto_resume")]
    pub auto_resume: bool,

    #[serde(default = "defaults::broadcast")]
    pub broadcast: bool,
}

mod defaults {
    pub fn framerate() -> u8 {
        60
    }

    pub fn auto_resume() -> bool {
        false
    }

    pub fn broadcast() -> bool {
        false
    }
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            framerate: defaults::framerate(),
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
