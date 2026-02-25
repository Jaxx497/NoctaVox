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

use crate::ui_state::{ProgressDisplay, UiState};
use ratatui::widgets::StatefulWidget;

pub(crate) const DEFAULT_AMP: f32 = 1.0;

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
                ProgressDisplay::Waveform => match !state.get_waveform_as_slice().is_empty() {
                    true => Waveform.render(area, buf, state),
                    false => Oscilloscope.render(area, buf, state),
                },
                ProgressDisplay::Oscilloscope => Oscilloscope.render(area, buf, state),
                ProgressDisplay::Spectrum => SpectrumAnalyzer.render(area, buf, state),
            }
            Timer.render(area, buf, state);
        }
    }
}
