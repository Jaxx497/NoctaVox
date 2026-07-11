use ratatui::{
    layout::Alignment,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

use crate::{
    key_handler::{HelpRow, help_rows},
    tui::widgets::POPUP_PADDING,
    ui_state::UiState,
};

pub struct KeymapGuide;

impl StatefulWidget for KeymapGuide {
    type State = UiState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let theme = state.theme.get_display_theme(true);
        let (accent, primary, muted, border, border_type, bg) = (
            theme.accent,
            theme.text_primary,
            theme.text_muted,
            theme.border,
            theme.border_type,
            theme.bg,
        );

        // Keys are right-aligned within the left ~40% of the inner width;
        // descriptions start just after and run left-aligned. Inner width =
        // popup width minus borders (2) and POPUP_PADDING left+right (10).
        let key_col = ((area.width as usize).saturating_sub(12) * 2 / 5).max(6);

        let lines: Vec<Line> = help_rows()
            .into_iter()
            .map(|row| match row {
                HelpRow::Blank => Line::from(""),
                HelpRow::Header(title) => {
                    // Pad with a plain span so only the title is underlined,
                    // while its right edge still aligns with the key column.
                    let pad = " ".repeat(key_col.saturating_sub(title.chars().count()));
                    Line::from(vec![
                        Span::from(pad),
                        Span::from(title).fg(accent).bold().underlined(),
                    ])
                }
                HelpRow::Key(key) => Line::from(vec![
                    Span::from(format!("{:>kw$}   ", key.label, kw = key_col)).fg(muted),
                    Span::from(key.desc).fg(primary),
                ]),
            })
            .collect();

        let block = Block::bordered()
            .border_type(border_type)
            .border_style(border)
            .title(" Keymaps ")
            .title_bottom(" [Esc] close ──── j │ k scroll ")
            .title_alignment(Alignment::Center)
            .padding(POPUP_PADDING)
            .bg(bg);

        // Direct scroll offset (top visible line) — one line per keypress, no
        // invisible cursor. `popup.selection` stores the requested offset; clamp
        // it to what actually scrolls and write the clamped value back so the
        // handler can't run the offset past the end.
        let visible = block.inner(area).height as usize;
        let max_scroll = lines.len().saturating_sub(visible);
        let offset = state
            .popup
            .selection
            .selected()
            .unwrap_or(0)
            .min(max_scroll);
        state.popup.selection.select(Some(offset));

        Paragraph::new(lines)
            .block(block)
            .scroll((offset as u16, 0))
            .render(area, buf);
    }
}
