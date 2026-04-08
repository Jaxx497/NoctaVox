use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Resampler};

use crate::{RESAMPLER_CHUNK_SIZE, RESAMPLER_SUBCHUNK_SIZE, error::VoxError};

pub struct VoxResampler {
    resampler: Fft<f32>,
    output_buf: Vec<f32>,
    pub input_rate: u32,
    channels: usize,
}

impl VoxResampler {
    pub fn new(
        input_rate: u32,
        output_rate: u32,
        channels: usize,
    ) -> Result<Option<Self>, VoxError> {
        if input_rate == output_rate {
            return Ok(None);
        }

        let resampler = Fft::<f32>::new(
            input_rate as usize,
            output_rate as usize,
            RESAMPLER_CHUNK_SIZE,
            RESAMPLER_SUBCHUNK_SIZE,
            channels,
            FixedSync::Input,
        )
        .map_err(|e| VoxError::Resampler(e.to_string()))?;

        let output_buf = vec![0.0f32; resampler.output_frames_max() * channels];

        Ok(Some(Self {
            resampler,
            output_buf,
            input_rate,
            channels,
        }))
    }

    pub fn process<F>(&mut self, pending: &mut Vec<f32>, mut output: F) -> Result<(), VoxError>
    where
        F: FnMut(&[f32]),
    {
        let frames_needed = self.resampler.input_frames_next();
        let samples_needed = frames_needed * self.channels;

        while pending.len() >= samples_needed {
            let input = InterleavedSlice::new(&pending[..samples_needed], self.channels, frames_needed)
                .map_err(|e| VoxError::Resampler(e.to_string()))?;

            let out_frames_max = self.resampler.output_frames_max();
            let mut out = InterleavedSlice::new_mut(&mut self.output_buf, self.channels, out_frames_max)
                .map_err(|e| VoxError::Resampler(e.to_string()))?;

            let (_, out_frames) = self
                .resampler
                .process_into_buffer(&input, &mut out, None)
                .map_err(|e| VoxError::Resampler(e.to_string()))?;

            pending.drain(..samples_needed);
            output(&self.output_buf[..out_frames * self.channels]);
        }

        Ok(())
    }

    pub fn flush<F>(&mut self, pending: &mut Vec<f32>, output: F) -> Result<(), VoxError>
    where
        F: FnMut(&[f32]),
    {
        if pending.is_empty() {
            return Ok(());
        }

        let frames_needed = self.resampler.input_frames_next();
        let samples_needed = frames_needed * self.channels;
        pending.resize(samples_needed, 0.0);
        self.process(pending, output)
    }

    pub fn reset(&mut self) {
        self.resampler.reset();
    }
}
