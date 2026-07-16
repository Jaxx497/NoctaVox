use crate::{
    CONFIG_DIR,
    config::{GeneralConfig, icons::UserIcons},
};
use anyhow::{Context, anyhow};
use serde::Deserialize;
use std::fmt::Write as _;
use std::fs;
use voxio::ReplayGainMode;

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct UserConfig {
    #[serde(default = "GeneralConfig::default")]
    pub general: GeneralConfig,

    #[serde(default = "UserIcons::default")]
    pub icons: UserIcons,
}

impl UserConfig {
    pub fn load() -> anyhow::Result<Self> {
        let path = CONFIG_DIR.join("config.toml");
        match fs::read_to_string(&path) {
            Ok(s) => toml::from_str(&s).map_err(|e| parse_error(&s, e)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                fs::write(&path, default_config())
                    .with_context(|| format!("Failed to write {}", path.display()))?;
                Ok(Self::default())
            }
            Err(e) => Err(anyhow!("Failed to read config.toml\n\n{e}")),
        }
    }
}

fn parse_error(src: &str, e: toml::de::Error) -> anyhow::Error {
    let location = match e.span() {
        Some(span) => format!("line {}", src[..span.start].matches('\n').count() + 1),
        None => String::from("could not be read"),
    };

    anyhow!(
        "Failed to read [config.toml]\n\n[{location}]: {}\n\nFalling back to default config.",
        e.message().split(',').next().unwrap()
    )
}

fn default_config() -> String {
    let general = GeneralConfig::default();
    let icons = UserIcons::default();

    let replay_gain = match general.replay_gain {
        ReplayGainMode::Track => "track",
        ReplayGainMode::Album => "album",
        _ => "off",
    };

    let mut conf = String::from(
        "# NoctaVox base configuration\n\
         # Uncomment any value to override its default.\n\n\
         [general]\n",
    );

    let _ = writeln!(conf, "# {:<17}= {}", "framerate", general.framerate);
    let _ = writeln!(
        conf,
        "# {:<17}= {}",
        "history_capacity", general.history_capacity
    );
    let _ = writeln!(conf, "# {:<17}= {:?}", "seek_small", general.seek_small);
    let _ = writeln!(conf, "# {:<17}= {:?}", "seek_large", general.seek_large);
    let _ = writeln!(
        conf,
        "# {:<17}= {}",
        "update_on_start", general.update_on_start
    );
    let _ = writeln!(conf, "# {:<17}= {}", "auto_resume", general.auto_resume);
    let _ = writeln!(conf, "# {:<17}= \"{}\"", "replay_gain", replay_gain);
    let _ = writeln!(conf, "# {:<17}= {}", "broadcast", general.broadcast);

    conf.push_str("\n[icons]\n");

    let _ = writeln!(conf, "# {:<10}= \"{}\"", "decorator", icons.decorator);
    let _ = writeln!(conf, "# {:<10}= \"{}\"", "selector", icons.selector);
    let _ = writeln!(conf, "# {:<10}= \"{}\"", "playing", icons.playing);
    let _ = writeln!(conf, "# {:<10}= \"{}\"", "paused", icons.paused);
    let _ = writeln!(conf, "# {:<10}= \"{}\"", "queued", icons.queued);
    let _ = writeln!(conf, "# {:<10}= \"{}\"", "repeat", icons.repeat);
    let _ = writeln!(conf, "# {:<10}= \"{}\"", "upcoming", icons.upcoming);
    let _ = writeln!(conf, "# {:<10}= \"{}\"", "selected", icons.selected);
    let _ = writeln!(conf, "# {:<10}= \"{}\"", "expanded", icons.expanded);
    let _ = writeln!(conf, "# {:<10}= \"{}\"", "collapsed", icons.collapsed);

    conf
}
