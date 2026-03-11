use crate::ui_state::{
    ProgressGradientRaw, fade_color,
    theme::{color_utils::get_gradient_color, theme_utils::parse_color},
};
use anyhow::Result;
use ratatui::style::Color;
use std::sync::Arc;

#[derive(Clone, PartialEq)]
pub enum ProgressGradient {
    Static(Color),
    Gradient(Arc<[Color]>),
}

#[derive(Clone)]
pub enum InactiveGradient {
    Dimmed,
    Still,
    Static(Color),
    Gradient(Arc<[Color]>),
}

impl ProgressGradient {
    pub(super) fn from_raw(raw: &ProgressGradientRaw) -> Result<ProgressGradient> {
        match raw {
            ProgressGradientRaw::Single(c) => Ok(ProgressGradient::Static(parse_color(&c)?)),
            ProgressGradientRaw::Gradient(colors) => {
                if colors.len() == 1 {
                    return Ok(ProgressGradient::Static(parse_color(&colors[0])?));
                }

                let gradient = colors
                    .iter()
                    .map(|c| parse_color(&c))
                    .collect::<Result<Vec<Color>>>()?;

                Ok(ProgressGradient::Gradient(gradient.into()))
            }
        }
    }

    pub fn color_at(&self, position: f32, time: f32, speed: f32) -> Color {
        match &self {
            ProgressGradient::Static(c) => *c,
            ProgressGradient::Gradient(g) => get_gradient_color(&g, position, time * speed),
        }
    }
}

impl InactiveGradient {
    pub(super) fn from_raw(raw: &ProgressGradientRaw) -> Result<InactiveGradient> {
        match raw {
            ProgressGradientRaw::Single(s) if s.to_lowercase().as_str() == "dimmed" => {
                Ok(InactiveGradient::Dimmed)
            }
            ProgressGradientRaw::Single(s) if s.to_lowercase().as_str() == "still" => {
                Ok(InactiveGradient::Still)
            }
            ProgressGradientRaw::Single(s) => {
                let color = parse_color(&s)?;
                Ok(InactiveGradient::Static(color))
            }
            ProgressGradientRaw::Gradient(colors) => {
                let gradient = colors
                    .iter()
                    .map(|c| parse_color(&c))
                    .collect::<Result<Vec<Color>>>()?;

                Ok(InactiveGradient::Gradient(gradient.into()))
            }
        }
    }

    pub fn color_at(
        &self,
        position: f32,
        time: f32,
        speed: f32,
        dark: bool,
        amp: f32,
        played: &ProgressGradient,
    ) -> Color {
        match self {
            InactiveGradient::Static(c) => *c,
            InactiveGradient::Gradient(g) => get_gradient_color(g, position, time * speed),
            InactiveGradient::Dimmed => {
                let brightness = match played {
                    ProgressGradient::Static(_) => 0.4,
                    ProgressGradient::Gradient(g) if g.len() == 1 => 0.4,
                    _ => 0.12 + (amp * 0.4),
                };
                fade_color(dark, played.color_at(position, time, speed), brightness)
            }
            InactiveGradient::Still => {
                let brightness = match played {
                    ProgressGradient::Static(_) => 0.4,
                    ProgressGradient::Gradient(g) if g.len() == 1 => 0.4,
                    _ => 0.12 + (amp * 0.4),
                };
                fade_color(dark, played.color_at(position, 0.0, speed), brightness)
            }
        }
    }
}
