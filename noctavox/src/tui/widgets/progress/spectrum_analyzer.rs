use crate::ui_state::UiState;
use ratatui::{
    style::Stylize,
    widgets::{
        canvas::{Canvas, Context, Line},
        Block, Padding, StatefulWidget, Widget,
    },
};
use rustfft::{num_complex::Complex, FftPlanner};

pub struct SpectrumAnalyzer;

const NUM_BARS: usize = 120;
const FFT_SIZE: usize = 1024;
const FALLOFF_RATE: f32 = 0.05;

impl StatefulWidget for SpectrumAnalyzer {
    type State = UiState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let theme = state.theme_manager.get_display_theme(true);
        let elapsed = state.get_playback_elapsed_f32();
        let samples = state.sample_tap.make_contiguous();

        if samples.is_empty() {
            return;
        }

        let num_samples = samples.len().min(FFT_SIZE * 2);

        // Prepare input for FFT
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);

        let mut buffer: Vec<Complex<f32>> = vec![Complex::new(0.0, 0.0); FFT_SIZE];
        
        // Apply Hann window and fill buffer
        for i in 0..FFT_SIZE {
            if i < num_samples {
                let multiplier = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE as f32 - 1.0)).cos());
                buffer[i] = Complex::new(samples[i] * multiplier, 0.0);
            }
        }

        fft.process(&mut buffer);

        // Calculate magnitude for NUM_BARS bins using logarithmic frequency spread
        let mut new_bars = vec![0.0; NUM_BARS];
        
        // We only care about the first half (Nyquist limit)
        for i in 0..NUM_BARS {
            // Non-linear mapping to give more resolution to lower frequencies
            let start_freq = (i as f32 / NUM_BARS as f32).powf(1.5) * (FFT_SIZE / 2) as f32;
            let end_freq = ((i + 1) as f32 / NUM_BARS as f32).powf(1.5) * (FFT_SIZE / 2) as f32;
            
            let mut start_idx = start_freq.floor().max(1.0) as usize; // skip DC offset (index 0)
            let end_idx = end_freq.ceil().min((FFT_SIZE / 2) as f32) as usize;

            if start_idx > end_idx {
                start_idx = end_idx;
            }
            
            let mut sum = 0.0;
            let mut count = 0;
            
            for j in start_idx..=end_idx {
                if j < buffer.len() {
                    let mag = buffer[j].norm();
                    sum += mag;
                    count += 1;
                }
            }
            
            if count > 0 {
                let mag = sum / count as f32;
                // Convert to dB-like scale and normalize
                let log_mag = (mag.max(1e-5).log10() + 2.0) / 3.0;
                new_bars[i] = log_mag.clamp(0.0, 1.0);
            }
        }

        // Apply smoothing and decay (requires state mutability)
        if state.spectrum_bars.len() != NUM_BARS {
            state.spectrum_bars = vec![0.0; NUM_BARS];
            state.spectrum_peaks = vec![0.0; NUM_BARS];
        }

        for i in 0..NUM_BARS {
            // Smooth rising, slow falling
            if new_bars[i] > state.spectrum_bars[i] {
                // Rise quickly
                state.spectrum_bars[i] = state.spectrum_bars[i] * 0.5 + new_bars[i] * 0.5;
            } else {
                // Fall slowly
                state.spectrum_bars[i] = (state.spectrum_bars[i] - FALLOFF_RATE).max(0.0);
            }

            // Update peaks
            if state.spectrum_bars[i] >= state.spectrum_peaks[i] {
                state.spectrum_peaks[i] = state.spectrum_bars[i].min(1.0);
            } else {
                // Slower falloff for peaks
                state.spectrum_peaks[i] = (state.spectrum_peaks[i] - (FALLOFF_RATE * 0.3)).max(0.0);
            }
        }

        let bars = state.spectrum_bars.clone();
        let peaks = state.spectrum_peaks.clone();

        let v_marg = match area.height >= 10 {
            true => ((area.height as f32) * 0.1) as u16,
            false => 0,
        };

        Canvas::default()
            .x_bounds([0.0, NUM_BARS as f64])
            .y_bounds([0.0, 1.0])
            .marker(theme.spectrum_style)
            .paint(|ctx| {
                draw_spectrum(ctx, &bars, &peaks, elapsed, &theme);
            })
            .background_color(theme.bg_global)
            .block(Block::new().bg(theme.bg_global).padding(Padding {
                left: 1,
                right: 1,
                top: v_marg,
                bottom: v_marg,
            }))
            .render(area, buf);
    }
}

fn draw_spectrum(ctx: &mut Context, bars: &[f32], peaks: &[f32], time: f32, theme: &crate::ui_state::DisplayTheme) {
    let num_bars = bars.len();
    
    // Width of each bar
    let bar_width = 0.6;
    
    for i in 0..num_bars {
        let x = i as f64 + (1.0 - bar_width) / 2.0;
        let height = bars[i] as f64;
        let peak_height = peaks[i] as f64;
        
        let progress = i as f32 / num_bars as f32;
        let color = theme.get_focused_color(progress, time / 2.0);
        let peak_color = theme.get_inactive_color(progress, time / 2.0, 1.0);

        // Draw the main bar
        if height > 0.05 {
            ctx.draw(&Line {
                x1: x,
                y1: 0.0,
                x2: x,
                y2: height,
                color,
            });
        }
        
        // Draw the peak indicator
        if peak_height > 0.05 {
            ctx.draw(&Line {
                x1: x,
                y1: peak_height.min(0.99),
                x2: x + bar_width,
                y2: peak_height.min(0.99),
                color: peak_color,
            });
        }
    }
}
