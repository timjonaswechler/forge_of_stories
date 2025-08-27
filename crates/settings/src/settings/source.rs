//! Quellenstapel für Settings – jetzt mit server/admin o. ä.

#[derive(Clone, Copy, Debug)]
pub struct SettingsSources<'a, T> {
    pub default: &'a T,      // eingebettete Defaults (global.toml)
    pub user: Option<&'a T>, // User-Config (settings.toml)

    // Weitere Dateien („Domains“):
    pub server: Option<&'a T>, // server.toml (autoritative Werte)
    pub admin: Option<&'a T>,  // admin.toml (TUI/Web-Config)

                               // Erweiterbar:
                               // pub global: Option<&'a T>,
                               // pub extensions: Option<&'a T>,
                               // pub project: &'a [&'a T],
}
