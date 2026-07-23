#[derive(Default, PartialEq, Eq, Clone)]
pub enum Mode {
    Power,
    #[default]
    Library,
    Fullscreen,
    Queue,
    Search,
    QUIT,
}

impl PartialEq<Mode> for &Mode {
    fn eq(&self, other: &Mode) -> bool {
        std::mem::discriminant(*self) == std::mem::discriminant(other)
    }
}

impl Mode {
    pub fn to_string(&self) -> String {
        match self {
            Mode::Power => "Power",
            Mode::Library => "Library",
            Mode::Fullscreen => "Fullscreen",
            Mode::Queue => "Queue",
            Mode::Search => "Search",
            Mode::QUIT => "Quit",
        }
        .to_string()
    }
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "power" => Mode::Power,
            "library" => Mode::Library,
            "queue" => Mode::Queue,
            "search" => Mode::Search,
            "quit" => Mode::QUIT,
            _ => Mode::Library,
        }
    }
}
