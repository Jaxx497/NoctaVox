use crate::ui_state::UiState;
use ratatui::{
    style::Stylize,
    widgets::{
        canvas::{Canvas, Context, Line, Rectangle},
        Block, Padding, StatefulWidget, Widget,
    },
};
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, scaling::divide_by_N_sqrt, windows::hann_window};

pub struct SpectrumAnalyzer;

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
                
                let num_bars = 120; // Exact number of discrete visual columns
                
                // We keep track of the smoothed bars directly, one for each column
                if state.spectrum_bars.len() != num_bars {
                    state.spectrum_bars = vec![0.0; num_bars];
                }

                if !freqs.is_empty() {
                    let max_freq_idx = freqs.len();

                    for i in 0..num_bars {
                        // Artificially stretch the lower frequencies logarithmically across discrete columns
                        let fraction_start = i as f32 / num_bars as f32;
                        let fraction_end = (i + 1) as f32 / num_bars as f32;

                        let mut start_idx = (fraction_start.powf(3.0) * max_freq_idx as f32).max(1.0) as usize;
                        let mut end_idx = (fraction_end.powf(3.0) * max_freq_idx as f32).max(1.0) as usize;
                        
                        // Prevent empty buckets to ensure NO empty visual column spaces!
                        if end_idx <= start_idx {
                            end_idx = start_idx + 1;
                        }
                        end_idx = end_idx.min(max_freq_idx);
                        if start_idx >= max_freq_idx {
                            start_idx = max_freq_idx.saturating_sub(1);
                        }
                        
                        let mut sum = 0.0;
                        let mut count = 0;
                        
                        for j in start_idx..end_idx {
                            let mag = freqs[j].1.val();
                            sum += mag;
                            count += 1;
                        }
                        
                        let mut normalized = 0.0;
                        if count > 0 {
                            let mag = sum / count as f32;
                            let log_mag = (mag * 10.0).log10() / 2.0;
                            normalized = log_mag.clamp(0.0, 1.0);
                        }
                        
                        // Apply smoothing and decay
                        if normalized > state.spectrum_bars[i] {
                            state.spectrum_bars[i] = state.spectrum_bars[i] * 0.5 + normalized * 0.5;
                        } else {
                            state.spectrum_bars[i] = (state.spectrum_bars[i] - FALLOFF_RATE).max(0.0);
                        }
                    }
                }
            }
            Err(_) => {
                return;
            }
        }

        let bars = state.spectrum_bars.clone();

        let v_marg = match area.height >= 10 {
            true => ((area.height as f32) * 0.1) as u16,
            false => 0,
        };

        Canvas::default()
            .x_bounds([0.0, bars.len() as f64])
            .y_bounds([0.0, 1.0])
            .marker(theme.spectrum_style)
            .paint(move |ctx| {
                let width = area.width;
                draw_spectrum(ctx, &bars, elapsed, &theme, width);
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

fn draw_spectrum(ctx: &mut Context, bars: &[f32], time: f32, theme: &crate::ui_state::DisplayTheme, area_width: u16) {
    let num_bars = bars.len();
    
    for i in 0..num_bars {
        let x = i as f64;
        let height = bars[i] as f64;
        
        // Skip DC offset and very low values
        if i == 0 || height < 0.05 {
            continue;
        }

        let progress = i as f32 / num_bars as f32;
        let color = theme.get_focused_color(progress, time / 2.0);

        if area_width < 100 {
            // Lines create a more detailed and cleaner look on small screens
            ctx.draw(&Line {
                x1: x,
                y1: 0.0,
                x2: x,
                y2: height,
                color,
            });
        } else {
            // Rectangles cleanly extend the column shapes in wider views
            // Width of 0.5 adds Winamp-style gap spacing between vertical bars seamlessly
            ctx.draw(&Rectangle {
                x,
                y: 0.0,
                width: 0.5,
                height,
                color,
            });
        }
    }
}
