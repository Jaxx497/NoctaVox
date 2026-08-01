use super::fade_color;
use ratatui::text::{Line, Span};
use std::f32::consts::PI;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const SWEEP_SECONDS: f32 = 2.0;
const DELAY_SECONDS: f32 = 7.0;
const BAND_HALF_WIDTH: f32 = 10.0;
const PAD: f32 = 10.0;
const PEAK: f32 = 0.8;

pub fn shimmer_line<'a>(mut line: Line<'a>, time: f32) -> Line<'a> {
    let cycle_pos = time.rem_euclid(DELAY_SECONDS);
    if cycle_pos > SWEEP_SECONDS {
        return line; // resting, which is where most frames land
    }

    let total: usize = line.spans.iter().map(|s| s.content.width()).sum();
    if total == 0 {
        return line;
    }

    let period = total as f32 + PAD * 2.0;
    let center = (cycle_pos / SWEEP_SECONDS) * period - PAD;
    let band_start = center - BAND_HALF_WIDTH;
    let band_end = center + BAND_HALF_WIDTH;

    let mut out = Vec::with_capacity(line.spans.len() + 8);
    let mut idx = 0.0;

    for span in std::mem::take(&mut line.spans) {
        let width = span.content.width() as f32;

        let base = match span.style.fg.or(line.style.fg) {
            Some(fg) if idx + width > band_start && idx < band_end => fg,
            _ => {
                idx += width;
                out.push(span);
                continue;
            }
        };

        let mut buffer = String::new();
        let mut current = base;

        for ch in span.content.chars() {
            let w = UnicodeWidthChar::width(ch).unwrap_or(0);

            if w > 0 {
                let dist = (idx - center).abs();

                let intensity = match dist < BAND_HALF_WIDTH {
                    true => 0.5 * (1.0 + (PI * dist / BAND_HALF_WIDTH).cos()),
                    false => 0.0,
                };

                let color = fade_color(false, base, 1.0 - intensity * PEAK);

                if color != current && !buffer.is_empty() {
                    out.push(Span::styled(
                        std::mem::take(&mut buffer),
                        span.style.fg(current),
                    ));
                }

                current = color;
                idx += w as f32;
            }

            buffer.push(ch);
        }

        if !buffer.is_empty() {
            out.push(Span::styled(buffer, span.style.fg(current)));
        }
    }

    line.spans = out;
    line
}
