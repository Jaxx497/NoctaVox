use ratatui::{
    layout::{Constraint, Layout, Rect},
    widgets::{Clear, StatefulWidget, Widget},
};

use crate::{
    tui::{
        ErrorMsg,
        widgets::{KeymapGuide, PlaylistPopup, RootManager, ThemeManager, UserStats},
    },
    ui_state::{PopupType, UiState},
};

pub struct PopupManager;
impl StatefulWidget for PopupManager {
    type State = UiState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let popup_rect = match &state.popup.current {
            PopupType::Stats => centered_rect(60, 80, area),
            PopupType::Playlist(_) => centered_rect(35, 40, area),
            PopupType::Settings(_) => centered_rect(40, 40, area),
            PopupType::ThemeManager => centered_rect(40, 40, area),
            PopupType::KeymapGuide => centered_rect(65, 70, area),
            PopupType::Error(_) => centered_rect(50, 40, area),
            _ => return,
        };

        Clear.render(popup_rect, buf);
        match &state.popup.current {
            PopupType::Stats => UserStats.render(popup_rect, buf, state),
            PopupType::Playlist(_) => PlaylistPopup.render(popup_rect, buf, state),
            PopupType::Settings(_) => RootManager.render(popup_rect, buf, state),
            PopupType::ThemeManager => ThemeManager.render(popup_rect, buf, state),
            PopupType::KeymapGuide => KeymapGuide.render(popup_rect, buf, state),
            PopupType::Error(_) => ErrorMsg.render(popup_rect, buf, state),
            _ => unreachable!(),
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let x_pad = 100_u16.saturating_sub(percent_x) / 2;
    let y_pad = 100_u16.saturating_sub(percent_y) / 2;

    let popup_layout = Layout::vertical([
        Constraint::Percentage(y_pad),
        Constraint::Percentage(percent_y),
        Constraint::Percentage(y_pad),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage(x_pad),
        Constraint::Percentage(percent_x),
        Constraint::Percentage(x_pad),
    ])
    .split(popup_layout[1])[1]
}
