use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UserIcons {
    #[serde(default = "defaults::decorator")]
    pub decorator: String,
    #[serde(default = "defaults::selector")]
    pub selector: String,
    #[serde(default = "defaults::playing")]
    pub playing: String,
    #[serde(default = "defaults::paused")]
    pub paused: String,
    #[serde(default = "defaults::queued")]
    pub queued: String,
    #[serde(default = "defaults::repeat")]
    pub repeat: String,
    #[serde(default = "defaults::upcoming")]
    pub upcoming: String,
    #[serde(default = "defaults::selected")]
    pub selected: String,
}

impl UserIcons {
    pub const DECORATOR: &'static str = "✧";
    pub const SELECTOR: &'static str = "⮞";
    pub const PLAYING: &'static str = "♫";
    pub const PAUSED: &'static str = "󰏤";
    pub const QUEUED: &'static str = "";
    pub const REPEAT: &'static str = "";
    pub const UPCOMING: &'static str = "󰐑";
    pub const SELECTED: &'static str = "󱕣";
}

impl Default for UserIcons {
    fn default() -> Self {
        Self {
            selector: defaults::selector(),
            playing: defaults::playing(),
            paused: defaults::paused(),
            queued: defaults::queued(),
            repeat: defaults::repeat(),
            upcoming: defaults::upcoming(),
            selected: defaults::selected(),
            decorator: defaults::decorator(),
        }
    }
}

#[rustfmt::skip]
mod defaults {
    use crate::config::UserIcons;

    pub fn paused() ->   String { UserIcons::PAUSED.into() }
    pub fn queued() ->   String { UserIcons::QUEUED.into() }
    pub fn repeat() ->   String { UserIcons::REPEAT.into() }
    pub fn playing() ->  String { UserIcons::PLAYING.into() }
    pub fn selector() -> String { UserIcons::SELECTOR.into() }
    pub fn upcoming() -> String { UserIcons::UPCOMING.into() }
    pub fn selected() -> String { UserIcons::SELECTED.into() }
    pub fn decorator()-> String { UserIcons::DECORATOR.into() }
}
