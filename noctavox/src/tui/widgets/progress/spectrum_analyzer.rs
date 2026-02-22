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
                
                let max_freq_idx = freqs.len();
                
                // We keep track of the smoothed bars directly, one for each computed frequency output
                if state.spectrum_bars.len() != max_freq_idx {
                    state.spectrum_bars = vec![0.0; max_freq_idx];
                }

                if max_freq_idx > 0 {
                    for i in 0..max_freq_idx {
                        let mag = freqs[i].1.val();
                        let log_mag = (mag * 10.0).log10() / 2.0;
                        let normalized = log_mag.clamp(0.0, 1.0);
                        
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
            .paint(|ctx| {
                draw_spectrum(ctx, &bars, elapsed, &theme);
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

fn draw_spectrum(ctx: &mut Context, bars: &[f32], time: f32, theme: &crate::ui_state::DisplayTheme) {
    let num_bars = bars.len();
    
    for i in 0..num_bars {
        let x = i as f64;
        let height = bars[i] as f64;
        
        // Skip DC offset and very low values
        if i == 0 || height < 0.05 {
            continue;
        }

        // We apply a logarithmic visible scaling so low-freq lines are drawn over more x-space linearly
        // To match ratatui perfectly, we just map 1:1 and let the sub-pixel rendering handle visual density
        
        let progress = i as f32 / num_bars as f32;
        let color = theme.get_focused_color(progress, time / 2.0);

        ctx.draw(&Line {
            x1: x,
            y1: 0.0,
            x2: x,
            y2: height,
            color,
        });
    }
}
