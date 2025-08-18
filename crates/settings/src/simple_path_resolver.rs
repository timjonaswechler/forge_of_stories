use crate::source::SourceKind;
use std::path::PathBuf;

/// SimplePathResolver maps SourceKind to concrete filesystem paths using the
/// workspace `paths` crate.
pub struct SimplePathResolver;

impl SimplePathResolver {
    pub fn path_for(kind: SourceKind, extra: Option<&str>) -> Option<PathBuf> {
        match kind {
            SourceKind::Defaults => None,
            SourceKind::Global => Some(paths::config_dir().join("settings").join("global.toml")),
            SourceKind::User => Some(paths::config_dir().join("settings").join("user.toml")),
            SourceKind::Keybinds => {
                Some(paths::config_dir().join("settings").join("keybinds.toml"))
            }
            SourceKind::World => {
                extra.map(|id| paths::data_dir().join("saves").join(id).join("world.toml"))
            }
            SourceKind::Server => Some(paths::data_dir().join("server").join("server.toml")),
            SourceKind::Profiles => None,
        }
    }
}
