use std::sync::Arc;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Root {
    Library,
    Playlist,
}

impl Root {
    pub fn label(&self) -> &'static str {
        match self {
            Root::Library => "Albums",
            Root::Playlist => "Playlists",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum NodeKey {
    Root(Root),
    Artist(Arc<String>),
    Album(i64),
    Playlist(i64),
}

impl NodeKey {
    pub fn serialize(&self) -> String {
        match self {
            Self::Root(Root::Library) => "root:music".into(),
            Self::Root(Root::Playlist) => "root:playlists".into(),
            Self::Artist(n) => format!("artist:{n}"),
            Self::Album(id) => format!("album:{id}"),
            Self::Playlist(id) => format!("playlist:{id}"),
        }
    }

    pub fn deserialize(s: &str) -> Option<Self> {
        let (tag, rest) = s.split_once(':')?;
        match tag {
            "root" if rest == "music" => Some(Self::Root(Root::Library)),
            "root" if rest == "playlists" => Some(Self::Root(Root::Playlist)),
            "artist" => Some(Self::Artist(Arc::new(rest.to_string()))),
            "album" => rest.parse().ok().map(Self::Album),
            "playlist" => rest.parse().ok().map(Self::Playlist),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub enum RowKind {
    Category(Root),
    Artist {
        name: Arc<String>,
        children: Vec<i64>,
    },
    LoneAlbum {
        artist: Arc<String>,
        id: i64,
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
            RowKind::Category(r) => NodeKey::Root(*r),
            RowKind::Artist { name, .. } => NodeKey::Artist(Arc::clone(name)),
            RowKind::LoneAlbum { id, .. } => NodeKey::Album(*id),
            RowKind::Album(id) => NodeKey::Album(*id),
            RowKind::Playlist(id) => NodeKey::Playlist(*id),
        }
    }

    pub fn root(&self) -> Root {
        match &self.kind {
            RowKind::Category(r) => *r,
            RowKind::Artist { .. } | RowKind::LoneAlbum { .. } | RowKind::Album(_) => Root::Library,
            RowKind::Playlist(_) => Root::Playlist,
        }
    }

    pub fn collapse_key(&self) -> Option<NodeKey> {
        match &self.kind {
            RowKind::Category(_) | RowKind::Artist { .. } => Some(self.key()),
            RowKind::LoneAlbum { artist, .. } => Some(NodeKey::Artist(Arc::clone(artist))),
            _ => None,
        }
    }

    pub fn folds_in_place(&self) -> bool {
        matches!(self.kind, RowKind::LoneAlbum { .. })
    }
}
