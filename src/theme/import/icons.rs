use serde::Deserialize;

#[derive(Deserialize)]
pub struct IconScheme {
    pub decorator: Option<String>,
    pub selector: Option<String>,
    pub playing: Option<String>,
    pub paused: Option<String>,
    pub queued: Option<String>,
    pub repeat: Option<String>,
    pub upcoming: Option<String>,
    pub selected: Option<String>,
}
