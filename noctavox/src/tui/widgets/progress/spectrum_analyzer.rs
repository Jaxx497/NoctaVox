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
/// Fraction of each slot used by the bar; rest is gap between bars.
const BAR_WIDTH: f64 = 0.85;
/// Vertical lines per bar to fill the interior (ratatui Rectangle is outline-only, so we fill with lines).
const FILL_LINES_PER_BAR: u32 = 8;
/// Peak hold "gravity": how fast the peak line falls per frame (Winamp-style). Lower = hangs in the air longer.
const PEAK_FALL_RATE: f32 = 0.008;
/// Minimum bar height so every bar draws as a filled block (not hollow); same visual style across all bands.
const MIN_BAR_HEIGHT: f64 = 0.08;
/// Boost for left/right bands so the spectrum looks more evenly filled (bass/treble often weaker than mids).
const BAND_EDGE_BOOST: f32 = 0.35;
/// Log10(22050/20) — for log-scale frequency mapping so bass/mid/treble are perceptually even (left=bass, right=treble).
const LOG_FREQ_RANGE: f32 = 3.04_f32;

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

        let num_bars = (area.width.saturating_sub(2)).clamp(1, 256) as usize;

        match spectrum {
            Ok(spectrum_data) => {
                let freqs = spectrum_data.data();

                // We keep track of the smoothed bars and peak-hold (gravity) per column
                if state.spectrum_bars.len() != num_bars {
                    state.spectrum_bars = vec![0.0; num_bars];
                    state.spectrum_peaks = vec![0.0; num_bars];
                }

                if !freqs.is_empty() {
                    let max_freq_idx = freqs.len();
                    let nyquist = 22050.0_f32;

                    for i in 0..num_bars {
                        // Log-scale frequency mapping: 20 Hz (bass) to 22050 Hz (treble), so each bar gets
                        // a perceptually even slice (left = bass, middle = mid, right = treble).
                        let fraction_start = i as f32 / num_bars as f32;
                        let fraction_end = (i + 1) as f32 / num_bars as f32;
                        let start_freq = 20.0 * 10.0_f32.powf(fraction_start * LOG_FREQ_RANGE);
                        let end_freq = 20.0 * 10.0_f32.powf(fraction_end * LOG_FREQ_RANGE);
                        let start_idx =
                            (start_freq / nyquist * max_freq_idx as f32).clamp(0.0, (max_freq_idx as f32) - 1.0) as usize;
                        let mut end_idx =
                            (end_freq / nyquist * max_freq_idx as f32).clamp(1.0, max_freq_idx as f32) as usize;
                        if end_idx <= start_idx {
                            end_idx = start_idx + 1;
                        }
                        end_idx = end_idx.min(max_freq_idx);
                        
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

                        // Per-band visual boost: bass (left) and treble (right) are often weaker than mids;
                        // boost them so the whole spectrum looks more evenly filled.
                        let position = i as f32 / num_bars as f32; // 0 = left, 1 = right
                        let edge_gain = 1.0 + BAND_EDGE_BOOST * (2.0 * (position - 0.5).abs());
                        normalized = (normalized * edge_gain).min(1.0);

                        // Apply smoothing and decay
                        if normalized > state.spectrum_bars[i] {
                            state.spectrum_bars[i] = state.spectrum_bars[i] * 0.5 + normalized * 0.5;
                        } else {
                            state.spectrum_bars[i] = (state.spectrum_bars[i] - FALLOFF_RATE).max(0.0);
                        }

                        // Winamp-style peak hold: rises with bar, falls slowly (gravity)
                        let bar = state.spectrum_bars[i];
                        let peak = &mut state.spectrum_peaks[i];
                        *peak = if bar >= *peak {
                            bar
                        } else {
                            (*peak - PEAK_FALL_RATE).max(bar)
                        };
                    }
                }
            }
            Err(_) => {
                return;
            }
        }

        let bars = state.spectrum_bars.clone();
        let peaks = state.spectrum_peaks.clone();

        // Match oscilloscope: same padding (1,1) and vertical margin (25% when height > 20) so the pane is filled consistently.
        let v_marg = match area.height > 20 {
            true => ((area.height as f32) * 0.25) as u16,
            false => 0,
        };

        Canvas::default()
            .x_bounds([0.0, bars.len() as f64])
            .y_bounds([0.0, 1.0])
            .marker(theme.spectrum_style)
            .paint(move |ctx| {
                let width = area.width;
                draw_spectrum(ctx, &bars, &peaks, elapsed, &theme, width);
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

fn draw_spectrum(
    ctx: &mut Context,
    bars: &[f32],
    peaks: &[f32],
    time: f32,
    theme: &crate::ui_state::DisplayTheme,
    _area_width: u16,
) {
    let num_bars = bars.len();
    let bar_gap = (1.0 - BAR_WIDTH) / 2.0;
    let bar_w = BAR_WIDTH;

    for i in 0..num_bars {
        let x = i as f64;
        let x_bar = x + bar_gap;
        let height = bars[i].max(MIN_BAR_HEIGHT as f32) as f64;
        let peak = peaks.get(i).copied().unwrap_or(0.0) as f64;

        let progress = i as f32 / num_bars as f32;
        let color = theme.get_focused_color(progress, time / 2.0);

        // Ratatui Rectangle is outline-only (4 lines), so interior looks hollow. Fill the bar by drawing
        // multiple vertical lines across the width so every bar looks solid like waveform/oscilloscope.
        let n = FILL_LINES_PER_BAR.max(1) as f64;
        for k in 0..=FILL_LINES_PER_BAR {
            let t = if n > 0.0 { k as f64 / n } else { 0.0 };
            let x_line = x_bar + t * bar_w;
            ctx.draw(&Line {
                x1: x_line,
                y1: 0.0,
                x2: x_line,
                y2: height,
                color,
            });
        }

        if peak > 0.01 {
            ctx.draw(&Line {
                x1: x_bar,
                y1: peak,
                x2: x_bar + bar_w,
                y2: peak,
                color,
            });
        }
    }
}
