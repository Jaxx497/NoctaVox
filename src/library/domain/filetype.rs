use rusqlite::{
    Result as RusqliteResult, ToSql,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef},
};
use std::fmt::Display;

pub static LEGAL_EXTENSION: std::sync::LazyLock<std::collections::HashSet<&'static str>> =
    std::sync::LazyLock::new(|| {
        std::collections::HashSet::from([
            "aiff", "alac", "flac", "m4a", "mp3", "ogg", "opus", "wav", "webm",
        ])
    });

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub enum FileType {
    MP3 = 1,
    M4A = 2,
    OGG = 3,
    WAV = 4,
    FLAC = 5,
    OPUS = 6,
    WEBM = 7,
    AIFF = 8,
    ALAC = 9,
    #[default]
    ERR = 0,
}

impl From<&str> for FileType {
    fn from(str: &str) -> Self {
        match str.to_lowercase().as_ref() {
            "mp3" => Self::MP3,
            "aac" | "m4a" => Self::M4A,
            "ogg" => Self::OGG,
            "wav" => Self::WAV,
            "flac" => Self::FLAC,
            "opus" => Self::OPUS,
            "webm" => Self::WEBM,
            "aiff" => Self::AIFF,
            "alac" => Self::ALAC,
            _ => Self::ERR,
        }
    }
}

impl FileType {
    pub fn to_str(&self) -> &str {
        match self {
            FileType::MP3 => "mp3",
            FileType::M4A => "m4a",
            FileType::OGG => "ogg",
            FileType::WAV => "wav",
            FileType::FLAC => "flac",
            FileType::OPUS => "opus",
            FileType::WEBM => "webm",
            FileType::AIFF => "aiff",
            FileType::ALAC => "ALAC",
            FileType::ERR => "ERR",
        }
    }
}

impl FromSql for FileType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Integer(i) => Ok(FileType::from_i64(i)),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for FileType {
    fn to_sql(&self) -> RusqliteResult<rusqlite::types::ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Integer(self.to_i64())))
    }
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            FileType::MP3 => write!(f, "ᵐᵖ³"),
            FileType::M4A => write!(f, "ᵐ⁴ᵃ"),
            FileType::OGG => write!(f, "ᵒᵍᵍ"),
            FileType::OPUS => write!(f, "ᵒᵖᵘˢ"),
            FileType::WAV => write!(f, "ʷᵃᵛ"),
            FileType::FLAC => write!(f, "ᶠˡᵃᶜ"),
            FileType::WEBM => write!(f, "ʷᵉᵇᵐ"),
            FileType::AIFF => write!(f, "ᵃᶦᶠᶠ"),
            FileType::ALAC => write!(f, "ᵃˡᵃᶜ"),
            FileType::ERR => write!(f, "ERR"),
        }
    }
}

impl FileType {
    pub fn from_i64(value: i64) -> Self {
        match value {
            1 => Self::MP3,
            2 => Self::M4A,
            3 => Self::OGG,
            4 => Self::WAV,
            5 => Self::FLAC,
            6 => Self::OPUS,
            7 => Self::WEBM,
            8 => Self::AIFF,
            9 => Self::ALAC,
            _ => Self::ERR,
        }
    }

    pub fn to_i64(&self) -> i64 {
        *self as i64
    }
}
