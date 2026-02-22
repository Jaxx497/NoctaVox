use crate::ui_state::UiState;
use ratatui::{
    style::Stylize,
    widgets::{
        canvas::{Canvas, Context, Line},
        Block, Padding, StatefulWidget, Widget,
    },
};
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, scaling::divide_by_N_sqrt, windows::hann_window};

pub struct SpectrumAnalyzer;

const NUM_BARS: usize = 120;
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

        // Apply Hann window for spectrum-analyzer
        let num_samples = samples.len().min(1024);
        let mut windowed_samples = samples[..num_samples].to_vec();
        
        // Ensure size is a power of 2 for the FFT
        let power_of_two = windowed_samples.len().next_power_of_two() / 2;
        windowed_samples.truncate(power_of_two);
        
        if windowed_samples.len() < 32 {
            return; // Not enough samples
        }
        
        let hann_window = hann_window(&windowed_samples);

        // Compute spectrum
        let spectrum = samples_fft_to_spectrum(
            &hann_window,
            44100, // Assuming standard sample rate, though visual shape doesn't strictly need accurate Hz
            FrequencyLimit::Max(22050.0), // Nyquist
            Some(&divide_by_N_sqrt), // Scale down output
        );

        match spectrum {
            Ok(spectrum_data) => {
                let freqs = spectrum_data.data();
                
                // Calculate magnitude for NUM_BARS bins using logarithmic frequency spread
                let mut new_bars = vec![0.0; NUM_BARS];
                
                let max_freq_idx = freqs.len();
                
                if max_freq_idx > 0 {
                    for i in 0..NUM_BARS {
                        // Non-linear mapping to give more resolution to lower frequencies
                        let start_freq = (i as f32 / NUM_BARS as f32).powf(1.5) * max_freq_idx as f32;
                        let end_freq = ((i + 1) as f32 / NUM_BARS as f32).powf(1.5) * max_freq_idx as f32;
                        
                        let mut start_idx = start_freq.floor().max(1.0) as usize; // skip DC offset (index 0)
                        let end_idx = end_freq.ceil().min((max_freq_idx - 1) as f32) as usize;
            
                        if start_idx > end_idx {
                            start_idx = end_idx;
                        }
                        
                        let mut sum = 0.0;
                        let mut count = 0;
                        
                        for j in start_idx..=end_idx {
                            if j < max_freq_idx {
                                let mag = freqs[j].1.val();
                                sum += mag;
                                count += 1;
                            }
                        }
                        
                        if count > 0 {
                            let mag = sum / count as f32;
                            // The spectrum-analyzer crate tends to output useful ranges, adjust scaling if needed
                            let log_mag = (mag * 10.0).log10() / 2.0;
                            new_bars[i] = log_mag.clamp(0.0, 1.0);
                        }
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
            }
            Err(_) => {
                // Return early without drawing if FFT fails
                return;
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
