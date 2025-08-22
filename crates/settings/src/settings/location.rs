#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct SaveGameId(usize);

impl From<SaveGameId> for usize {
    fn from(value: SaveGameId) -> Self {
        value.0
    }
}

impl SaveGameId {
    pub fn from_usize(handle_id: usize) -> Self {
        Self(handle_id)
    }

    pub fn from_proto(id: u64) -> Self {
        Self(id as usize)
    }

    pub fn to_proto(&self) -> u64 {
        self.0 as u64
    }

    pub fn to_usize(&self) -> usize {
        self.0
    }
}

impl fmt::Display for SaveGameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SettingsLocation<'a> {
    pub worktree_id: SaveGameId,
    pub path: &'a Path,
}
