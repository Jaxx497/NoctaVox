mod borders;
mod colors;
mod icons;
mod meta;
mod progress;

pub use progress::*;

#[derive(serde::Deserialize)]
pub struct ThemeImport {
    #[serde(default)]
    pub meta: meta::MetaScheme,
    pub colors: colors::ColorScheme,
    pub borders: Option<borders::BorderScheme>,
    pub progress: Option<progress::ProgressScheme>,
    pub icons: Option<icons::IconScheme>,
}
