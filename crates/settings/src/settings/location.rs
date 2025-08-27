//! SettingsLocation (für lokale/kontextspezifische Werte).
//! MVP: Placeholder – du kannst später SaveGameId/Projektpfade anschließen.

use std::path::{Path, PathBuf};

/// Optional: SaveGameId/Projekt/World – jetzt nur Hilfstypen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SaveGameId(pub usize);

impl SaveGameId {
    pub fn from_usize(x: usize) -> Self {
        SaveGameId(x)
    }
    pub fn to_usize(self) -> usize {
        self.0
    }
}

/// Ein Ort, relativ zu dem ein Setting ausgewertet wird.
/// MVP: Wird noch nicht genutzt; API ist vorbereitet.
#[derive(Clone, Debug)]
pub struct SettingsLocation<'a> {
    pub savegame_id: SaveGameId,
    pub path: &'a Path,
}

impl<'a> SettingsLocation<'a> {
    pub fn new(savegame_id: SaveGameId, path: &'a Path) -> Self {
        Self { savegame_id, path }
    }

    pub fn to_owned(&self) -> (SaveGameId, PathBuf) {
        (self.savegame_id, self.path.to_path_buf())
    }
}
