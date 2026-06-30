mod oscilloscope;
mod progress_bar;
mod spectrum;
mod timer;
mod waveform;

pub use oscilloscope::Oscilloscope;
pub use progress_bar::ProgressBar;
pub use spectrum::SpectrumAnalyzer;
pub use timer::Timer;
pub use waveform::Waveform;

use crate::ui_state::{
    LayoutStyle,
    ProgressDisplay::{self},
    UiState,
};
use ratatui::widgets::StatefulWidget;

const DEFAULT_AMP: f32 = 1.0;

pub struct Progress;
impl StatefulWidget for Progress {
    type State = UiState;
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        if state.player_is_active() {
            state.fill_tap();
            match &state.get_progress_display() {
                ProgressDisplay::ProgressBar => ProgressBar.render(area, buf, state),
                ProgressDisplay::Waveform => match state.waveform_is_valid() {
                    true => Waveform.render(area, buf, state),
                    false => SpectrumAnalyzer.render(area, buf, state),
                },
                ProgressDisplay::Oscilloscope => Oscilloscope.render(area, buf, state),
                ProgressDisplay::Spectrum => SpectrumAnalyzer.render(area, buf, state),
            }
            if state.get_layout() == &LayoutStyle::Traditional {
                Timer.render(area, buf, state);
            }
        }
    }
}
