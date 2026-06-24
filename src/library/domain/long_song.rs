use super::{FileType, SongInfo};
use crate::{
    DurationStyle, calculate_signature, database::Database, get_readable_duration,
    normalize_metadata_str as nms,
};
use anyhow::{Result, anyhow, bail};
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
    time::Duration,
};
use symphonia::{
    core::{
        formats::{TrackType, probe::Hint},
        io::MediaSourceStream,
        meta::StandardTag,
        units::{Duration as SymphoniaDuration, TimeBase},
    },
    default::get_probe,
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
    pub(crate) bitrate: Option<u32>,
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

    pub fn build_song_symphonia(path: PathBuf) -> Result<LongSong> {
        let src = File::open(&path)?;

        let mss = MediaSourceStream::new(Box::new(src), Default::default());
        let mut hint = Hint::new();

        let ext = match &path.extension() {
            Some(n) => FileType::from(
                n.to_str()
                    .ok_or_else(|| anyhow!("Failed to obtain filetype from {}", path.display()))?,
            ),
            None => bail!("Unsupported extension: {:?}", path.extension()),
        };

        hint.with_extension(ext.to_str());

        let mut probed = get_probe().probe(&hint, mss, Default::default(), Default::default())?;

        let fallback_title = path
            .file_stem()
            .map(|s| nms(&s.to_string_lossy()))
            .ok_or_else(|| anyhow!("No song title!"))?;

        let id = calculate_signature(&path)?;
        let mut song_info = LongSong::new(path);
        song_info.id = id;

        let track = probed
            .first_track_known_codec(TrackType::Audio)
            .ok_or_else(|| anyhow!("No audio tracks!"))?;

        let duration = match (track.time_base, track.duration) {
            (Some(tb), Some(dur)) => get_duration(dur, tb),
            _ => {
                let mediainfo = probed.media_info();
                match (mediainfo.time_base, mediainfo.duration) {
                    (Some(tb), Some(dur)) => get_duration(dur, tb),
                    _ => Duration::ZERO,
                }
            }
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

        let mut release_year = None;
        let mut recording_year = None;

        let mut artist: Option<(u8, Arc<String>)> = None;
        let mut alb_art: Option<(u8, Arc<String>)> = None;

        let mut metadata = probed.metadata();
        loop {
            if let Some(md) = metadata.current() {
                for tag in &md.media.tags {
                    if let Some(std_tag) = &tag.std {
                        match std_tag {
                            StandardTag::TrackTitle(t) => song_info.title = nms(t),

                            StandardTag::Artist(a) => artist = best(artist, 0, a),
                            StandardTag::SortArtist(a) => artist = best(artist, 1, a),
                            StandardTag::Composer(c) => artist = best(artist, 2, c),
                            StandardTag::Performer(p) => artist = best(artist, 3, p),
                            StandardTag::OriginalArtist(o) => artist = best(artist, 4, o),
                            StandardTag::Author(a) => artist = best(artist, 5, a),

                            StandardTag::Album(a) => song_info.album = Arc::new(nms(a)),

                            StandardTag::AlbumArtist(aa) => alb_art = best(alb_art, 0, aa),
                            StandardTag::SortAlbumArtist(s) => alb_art = best(alb_art, 1, s),

                            StandardTag::TrackNumber(t) => song_info.track_no = Some(*t as u32),
                            StandardTag::DiscNumber(d) => song_info.disc_no = Some(*d as u32),

                            StandardTag::ReleaseYear(y) => release_year = Some(*y as u32),
                            StandardTag::RecordingYear(y) => recording_year = Some(*y as u32),
                            StandardTag::ReleaseDate(d) => {
                                release_year = release_year.or_else(|| d.get(..4)?.parse().ok());
                            }
                            StandardTag::RecordingDate(d) => {
                                recording_year =
                                    recording_year.or_else(|| d.get(..4)?.parse().ok());
                            }
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

        if ext == FileType::WAV {
            if let Ok(info) = read_wav_info_tags(&song_info.path) {
                for (key, val) in info {
                    match &key {
                        b"INAM" => song_info.title = nms(&val),
                        b"IART" => artist = best(artist, 0, &Arc::new(val)),
                        b"IPRD" => song_info.album = Arc::new(nms(&val)),
                        b"ICRD" | b"IYER" => {
                            release_year = release_year.or_else(|| val.get(..4)?.parse().ok());
                        }
                        b"ITRK" | b"IPRT" => song_info.track_no = val.parse().ok(),
                        _ => {}
                    }
                }
            }
        }

        if song_info.title.is_empty() {
            song_info.title = fallback_title
        }

        song_info.year = release_year.or(recording_year);

        match artist {
            Some((_, a)) => song_info.artist = Arc::new(nms(&a)),
            None => song_info.artist = Arc::clone(&NO_ARTIST),
        }

        match alb_art {
            Some((_, a)) => song_info.album_artist = Arc::new(nms(&a)),
            _ => song_info.album_artist = Arc::clone(&song_info.artist),
        }

        Ok(song_info)
    }

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

fn get_duration(dur: SymphoniaDuration, tb: TimeBase) -> Duration {
    let secs = dur.get() as f64 * tb.numer.get() as f64 / tb.denom.get() as f64;
    Duration::from_secs_f64(secs)
}

fn best<C: Clone>(current: Option<(u8, C)>, priority: u8, value: &C) -> Option<(u8, C)> {
    match current {
        Some((p, _)) if p < priority => current,
        _ => Some((priority, value.clone())),
    }
}

// BUG: This is a workaround for symphonia failing to properly parse RIFF tags

/// WAV stores tags in a RIFF `LIST/INFO` chunk that symphonia 0.6 never
/// surfaces (it discards parsed INFO and stops reading at the `data` chunk).
/// Walk the chunk structure ourselves and pull the INFO tags.
fn read_wav_info_tags(path: &Path) -> Result<Vec<([u8; 4], String)>> {
    let mut f = BufReader::new(File::open(path)?);

    // RIFF header: "RIFF" <u32 LE size> "WAVE"
    let mut hdr = [0u8; 12];
    f.read_exact(&mut hdr)?;
    if &hdr[0..4] != b"RIFF" || &hdr[8..12] != b"WAVE" {
        bail!("not a RIFF/WAVE file: {}", path.display());
    }

    let mut tags = Vec::new();
    let mut id = [0u8; 4];
    let mut sz = [0u8; 4];

    // Each chunk: <id:4> <size:u32 LE> <body...> (+1 pad byte if size is odd)
    loop {
        if f.read_exact(&mut id).is_err() {
            break; // clean EOF
        }
        f.read_exact(&mut sz)?;
        let size = u32::from_le_bytes(sz) as u64;

        if &id == b"LIST" {
            let mut form = [0u8; 4];
            f.read_exact(&mut form)?;
            if &form == b"INFO" {
                let mut buf = vec![0u8; (size - 4) as usize];
                f.read_exact(&mut buf)?;
                parse_info_body(&buf, &mut tags);
            } else {
                f.seek(SeekFrom::Current((size - 4) as i64))?;
            }
        } else {
            f.seek(SeekFrom::Current(size as i64))?;
        }

        if size % 2 == 1 {}
    }

    Ok(tags)
}

fn parse_info_body(buf: &[u8], tags: &mut Vec<([u8; 4], String)>) {
    let mut i = 0;
    while i + 8 <= buf.len() {
        let key: [u8; 4] = buf[i..i + 4].try_into().unwrap();
        let len = u32::from_le_bytes(buf[i + 4..i + 8].try_into().unwrap()) as usize;
        i += 8;
        if i + len > buf.len() {
            break;
        }
        // INFO values are null-terminated, typically ASCII/Latin-1.
        let val: String = buf[i..i + len]
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as char)
            .collect();
        let val = val.trim().to_string();
        if !val.is_empty() {
            tags.push((key, val));
        }
        i += len;
        if len % 2 == 1 {
            i += 1; // pad
        }
    }
}
