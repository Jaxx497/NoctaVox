use super::{fade_color, interpolate_color, scale_color};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    text::{Line, Span},
};
use std::{f32::consts::PI, sync::LazyLock, time::Instant};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const PEAK: f32 = 0.8;

pub struct Sweep {
    pub sweep_seconds: f32,
    pub cycle_seconds: f32,
    pub half_width: f32,
    pub pad: f32,
}

impl Sweep {
    pub const TEXT: Self = Self {
        sweep_seconds: 2.0,
        cycle_seconds: 7.0,
        half_width: 10.0,
        pad: 10.0,
    };

    pub fn phase(&self, time: f32) -> Option<f32> {
        let cycle_pos = time.rem_euclid(self.cycle_seconds);
        (cycle_pos <= self.sweep_seconds).then(|| cycle_pos / self.sweep_seconds)
    }

    pub fn center(&self, phase: f32, width: f32) -> f32 {
        phase * (width + self.pad * 2.0) - self.pad
    }

    pub fn intensity(&self, offset_from_center: f32) -> f32 {
        let dist = offset_from_center.abs();
        match dist < self.half_width {
            true => 0.5 * (1.0 + (PI * dist / self.half_width).cos()),
            false => 0.0,
        }
    }
}

pub struct Bar {
    pub sweep: Sweep,
    pub glow: f32,
    pub gradient: f32,
}

impl Bar {
    pub const SELECTION: Self = Self {
        sweep: Sweep {
            sweep_seconds: 2.0,
            cycle_seconds: 7.0,
            half_width: 18.0,
            pad: 10.0,
        },
        glow: 0.45,
        gradient: 0.23,
    };

    pub fn paint(&self, buf: &mut Buffer, area: Rect, base: Color, time: f32) {
        if area.is_empty() {
            return;
        }

        let dark = scale_color(base, 1.0 - self.gradient / 2.0);
        let light = scale_color(base, 1.0 + self.gradient / 2.0);
        let span = area.width.saturating_sub(1).max(1) as f32;

        let center = self
            .sweep
            .phase(time)
            .map(|phase| self.sweep.center(phase, area.width as f32));

        for x in area.left()..area.right() {
            let offset = (x - area.left()) as f32;
            let mut color = interpolate_color(dark, light, offset / span);

            if let Some(center) = center {
                let intensity = self.sweep.intensity(offset - center);
                if intensity > 0.0 {
                    color =
                        interpolate_color(color, fade_color(false, color, self.glow), intensity);
                }
            }

            for y in area.top()..area.bottom() {
                buf[(x, y)].set_bg(color);
            }
        }
    }
}

pub fn animation_time() -> f32 {
    static START: LazyLock<Instant> = LazyLock::new(Instant::now);
    START.elapsed().as_secs_f32()
}

pub fn shimmer_line<'a>(mut line: Line<'a>, time: f32) -> Line<'a> {
    let Some(phase) = Sweep::TEXT.phase(time) else {
        return line; // resting, which is where most frames land
    };

    let total: usize = line.spans.iter().map(|s| s.content.width()).sum();
    if total == 0 {
        return line;
    }

    let center = Sweep::TEXT.center(phase, total as f32);
    let band_start = center - Sweep::TEXT.half_width;
    let band_end = center + Sweep::TEXT.half_width;

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
                let intensity = Sweep::TEXT.intensity(idx - center);
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
