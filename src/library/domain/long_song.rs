use super::{FileType, SongInfo};
use crate::{
    DurationStyle, calculate_signature, database::Database, get_readable_duration,
    normalize_metadata_str as nms,
};
use anyhow::{Result, anyhow, bail};
// use lofty::{
//     file::{AudioFile, TaggedFileExt},
//     read_from_path,
//     tag::{Accessor, ItemKey},
// };
use symphonia::{
    core::{
        formats::{TrackType, probe::Hint},
        io::MediaSourceStream,
        meta::StandardTag,
    },
    default::get_probe,
};

use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
    time::Duration,
};

static NO_ARTIST: LazyLock<Arc<String>> = LazyLock::new(|| Arc::new(String::from("[NO ARTIST!]")));

#[derive(Default, Debug)]
pub struct LongSong {
    pub(crate) id: u64,
    pub(crate) title: String,
    pub(crate) year: Option<u32>,
    pub(crate) artist: Arc<String>,
    pub(crate) album_artist: Arc<String>,
    pub(crate) album: Arc<String>,
    pub(crate) track_no: Option<u32>,
    pub(crate) disc_no: Option<u32>,
    pub(crate) duration: Duration,
    pub(crate) channels: Option<u8>,
    pub(crate) bit_rate: Option<u32>,
    pub(crate) sample_rate: Option<u32>,
    pub(crate) filetype: FileType,
    pub(crate) path: PathBuf,
}

impl LongSong {
    pub fn new(path: PathBuf) -> Self {
        LongSong {
            path,
            ..Default::default()
        }
    }

    pub fn build_song_symphonia<P: AsRef<Path>>(path_raw: P) -> Result<LongSong> {
        let path = path_raw.as_ref();
        let src = File::open(path)?;

        let size = src.metadata()?.len();
        let mss = MediaSourceStream::new(Box::new(src), Default::default());
        let mut hint = Hint::new();

        let ext = match path.extension() {
            Some(n) => FileType::from(
                n.to_str()
                    .ok_or_else(|| anyhow!("Failed to obtain filetype from {}", path.display()))?,
            ),
            None => bail!("Unsupported extension: {:?}", path.extension()),
        };

        hint.with_extension(ext.to_str());

        let mut probed = get_probe().probe(&hint, mss, Default::default(), Default::default())?;

        let mut song_info = LongSong::new(PathBuf::from(path));
        song_info.id = calculate_signature(path)?;

        let track = probed
            .first_track_known_codec(TrackType::Audio)
            .ok_or_else(|| anyhow!("No audio tracks!"))?;

        let duration = match (track.time_base, track.duration) {
            (Some(tb), Some(dur)) => {
                let secs = dur.get() as f64 * tb.numer.get() as f64 / tb.denom.get() as f64;
                Duration::from_secs_f64(secs)
            }
            _ => Duration::ZERO,
        };

        let (channels, sample_rate) = track
            .codec_params
            .as_ref()
            .and_then(|cp| cp.audio())
            .map(|audio| {
                (
                    audio.channels.as_ref().map(|ch| ch.count() as u8),
                    audio.sample_rate,
                )
            })
            .unwrap_or((None, None));

        song_info.filetype = ext;
        song_info.channels = channels;
        song_info.sample_rate = sample_rate;

        song_info.duration = duration;
        song_info.bit_rate = (duration > Duration::ZERO)
            .then(|| (size as f64 * 8.0 / duration.as_secs_f64()) as u32);

        let mut metadata = probed.metadata();

        let mut release_year = None;
        let mut recording_year = None;
        let mut release_date = None;
        let mut recording_date = None;

        loop {
            if let Some(md) = metadata.current() {
                for tag in &md.media.tags {
                    if let Some(std_tag) = &tag.std {
                        match std_tag {
                            StandardTag::TrackTitle(t) => song_info.title = nms(t),
                            StandardTag::Artist(a) => song_info.artist = Arc::new(nms(a)),
                            // StandardTag::Performer(p) => song_info.artist = Arc::new(nms(p)),
                            StandardTag::Album(a) => song_info.album = Arc::new(nms(a)),
                            StandardTag::AlbumArtist(aa) => {
                                if song_info.album_artist.is_empty() {
                                    song_info.album_artist = Arc::new(nms(aa))
                                }
                            }
                            StandardTag::TrackNumber(t) => song_info.track_no = Some(*t as u32),
                            StandardTag::DiscNumber(d) => song_info.disc_no = Some(*d as u32),

                            StandardTag::ReleaseYear(y) => release_year = Some(*y as u32),
                            StandardTag::RecordingYear(y) => recording_year = Some(*y as u32),
                            StandardTag::ReleaseDate(d) => release_date = Some(d.clone()),
                            StandardTag::RecordingDate(d) => recording_date = Some(d.clone()),
                            _ => {}
                        }
                    }
                }
            }
            if metadata.is_latest() {
                break;
            }
            metadata.pop();
        }

        if song_info.title.is_empty() {
            song_info.title = path
                .file_stem()
                .map(|s| nms(&s.to_string_lossy()))
                .ok_or_else(|| anyhow!("No song title!"))?
        }

        song_info.year = release_year.or(recording_year).or_else(|| {
            release_date
                .or(recording_date)
                .and_then(|d| d.get(..4)?.parse().ok())
        });

        if song_info.artist.is_empty() {
            song_info.artist = NO_ARTIST.clone()
        }

        if song_info.album_artist.is_empty() {
            song_info.album_artist = Arc::clone(&song_info.artist)
        }

        Ok(song_info)
    }

    // pub fn build_song_lofty<P: AsRef<Path>>(path_raw: P) -> Result<LongSong> {
    //     let path = path_raw.as_ref();
    //     let mut song_info = LongSong::new(PathBuf::from(path));
    //
    //     song_info.id = calculate_signature(path)?;
    //
    //     song_info.filetype = match path.extension() {
    //         Some(n) => FileType::from(
    //             n.to_str()
    //                 .ok_or_else(|| anyhow!("Failed to obtain filetype from {}", path.display()))?,
    //         ),
    //         None => bail!("Unsupported extension: {:?}", path.extension()),
    //     };
    //
    //     let tagged_file = read_from_path(path)?;
    //     let properties = tagged_file.properties();
    //
    //     song_info.duration = properties.duration();
    //     song_info.channels = properties.channels();
    //     song_info.sample_rate = properties.sample_rate();
    //     song_info.bit_rate = properties.audio_bitrate();
    //
    //     song_info.title = tagged_file
    //         .primary_tag()
    //         .and_then(|tag| tag.title())
    //         .map(|s| nms(&s))
    //         .filter(|s| !s.is_empty())
    //         .unwrap_or_else(|| {
    //             path.file_stem()
    //                 .map(|stem| stem.to_string_lossy().into_owned())
    //                 .unwrap_or_default()
    //         });
    //
    //     if let Some(tag) = tagged_file.primary_tag() {
    //         song_info.album = Arc::new(tag.album().map(|s| nms(&s)).unwrap_or_default());
    //
    //         let artist = tag
    //             .artist()
    //             .map(|s| nms(&s))
    //             .unwrap_or("[NO ARTIST!]".into());
    //
    //         let album_artist = tag
    //             .get_string(ItemKey::AlbumArtist)
    //             .map(|s| nms(&s))
    //             .filter(|s| !s.is_empty())
    //             .unwrap_or_else(|| artist.to_string());
    //
    //         song_info.artist = Arc::new(artist);
    //         song_info.album_artist = Arc::new(album_artist);
    //
    //         song_info.year = tag.date().map(|ts| ts.year as u32).or_else(|| {
    //             tag.get_string(ItemKey::Year)
    //                 .and_then(|s| {
    //                     nms(s)
    //                         .split_once('-')
    //                         .map(|(y, _)| y.to_string())
    //                         .or_else(|| Some(s.to_string()))
    //                 })
    //                 .and_then(|s| s.parse::<u32>().ok())
    //         });
    //
    //         song_info.track_no = tag.track();
    //         song_info.disc_no = tag.disk();
    //     }
    //
    //     Ok(song_info)
    // }

    pub fn get_path(&self, db: &mut Database) -> Result<String> {
        db.get_song_path(self.id)
    }
}

impl SongInfo for LongSong {
    fn get_id(&self) -> u64 {
        self.id
    }

    fn get_title(&self) -> &str {
        &self.title
    }

    fn get_artist(&self) -> &str {
        &self.artist
    }

    fn get_album(&self) -> &str {
        &self.album
    }

    fn get_duration(&self) -> Duration {
        self.duration
    }

    fn get_duration_f32(&self) -> f32 {
        self.duration.as_secs_f32()
    }

    fn get_duration_str(&self, style: DurationStyle) -> String {
        get_readable_duration(self.duration, style)
    }
}
