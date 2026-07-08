use serde::Deserialize;

/// Theme-wide metadata (`[meta]`). Every field is optional and unknown keys are
/// ignored, so a theme may omit the table entirely or carry extra annotations.
#[derive(Deserialize, Default)]
#[serde(default)]
pub struct MetaScheme {
    pub dark: Option<bool>,
    pub author: Option<String>,
    pub name: Option<String>,
}
