use std::sync::Arc;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum NodeKey {
    MusicRoot,
    PlaylistsRoot,
    Artist(Arc<String>),
    Album(i64),
    Playlist(i64),
}

impl NodeKey {
    pub fn serialize(&self) -> String {
        match self {
            Self::MusicRoot => "root:music".into(),
            Self::PlaylistsRoot => "root:playlists".into(),
            Self::Artist(n) => format!("artist:{n}"),
            Self::Album(id) => format!("album:{id}"),
            Self::Playlist(id) => format!("playlist:{id}"),
        }
    }

    pub fn deserialize(s: &str) -> Option<Self> {
        let (tag, rest) = s.split_once(':')?;
        match tag {
            "root" if rest == "music" => Some(Self::MusicRoot),
            "root" if rest == "playlists" => Some(Self::PlaylistsRoot),
            "artist" => Some(Self::Artist(Arc::new(rest.to_string()))),
            "album" => rest.parse().ok().map(Self::Album),
            "playlist" => rest.parse().ok().map(Self::Playlist),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub enum RowKind {
    Category(NodeKey),
    Artist {
        name: Arc<String>,
        children: Vec<i64>,
    },
    Album(i64),
    Playlist(i64),
}

#[derive(Clone)]
pub struct SidebarRow {
    pub kind: RowKind,
    pub depth: u8,
}

impl SidebarRow {
    pub fn new(kind: RowKind, depth: u8) -> Self {
        Self { kind, depth }
    }

    pub fn key(&self) -> NodeKey {
        match &self.kind {
            RowKind::Category(k) => k.clone(),
            RowKind::Artist { name, .. } => NodeKey::Artist(Arc::clone(name)),
            RowKind::Album(id) => NodeKey::Album(*id),
            RowKind::Playlist(id) => NodeKey::Playlist(*id),
        }
    }

    pub fn collapse_key(&self) -> Option<NodeKey> {
        match &self.kind {
            RowKind::Category(_) | RowKind::Artist { .. } => Some(self.key()),
            _ => None,
        }
    }
}
