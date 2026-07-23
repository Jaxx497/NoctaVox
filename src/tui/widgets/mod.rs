mod bread_crumbs;
mod buffer_line;
mod popup;
mod popups;
mod progress;
mod search;
mod sidebar;
mod tracklist;

pub use bread_crumbs::BreadCrumbs;
pub use buffer_line::BufferLine;
pub use popup::PopupManager;
pub use popups::{ErrorMsg, KeymapGuide, PlaylistPopup, RootManager, ThemeManager, UserStats};
pub use progress::Progress;
pub use search::SearchBar;
pub use sidebar::SideBarHandler;
pub use tracklist::SongTable;

static POPUP_PADDING: ratatui::widgets::Padding = ratatui::widgets::Padding {
    left: 5,
    right: 5,
    top: 2,
    bottom: 2,
};
