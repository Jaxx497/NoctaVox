mod engine;
mod error;

pub use engine::Vox;
pub use error::VoxError;

const BUFFER_MS: usize = 100;
const CHANNEL_COUNT: usize = 16;
const PENDING_CAPACITY: usize = 8192;
const RESAMPLER_CHUNK_SIZE: usize = 1024;
const RESAMPLER_SUBCHUNK_SIZE: usize = 2;
const SAMPLE_TAP_CAPACITY: usize = 2048;
const SEEK_PREFILL_MS: usize = 10;
const SEEK_FADE_MS: usize = 30;
const MAX_PROBE_PACKETS: usize = 10;
