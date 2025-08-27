use crate::settings::SettingsResult;
use crate::settings::store::SettingsStore;
use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;
use toml_edit::{DocumentMut, Item};

/// Setzt doc[key] auf den serialisierten Wert von `value` (format-preserving).
pub fn set_key_to_serialized_item<T: Serialize>(
    doc: &mut DocumentMut,
    key: &str,
    value: &T,
) -> Result<()> {
    // Serialize über toml::to_string, dann als snippet zurück in Item parsen
    let val: TomlValue = toml::from_str(&toml::to_string(value)?)?;
    fn to_snippet(v: &TomlValue) -> String {
        match v {
            TomlValue::Array(arr) => {
                let elems = arr.iter().map(to_snippet).collect::<Vec<_>>();
                format!("[{}]", elems.join(", "))
            }
            TomlValue::Table(map) => {
                let mut parts = Vec::new();
                for (k, v) in map.iter() {
                    parts.push(format!("{} = {}", k, to_snippet(v)));
                }
                format!("{{ {} }}", parts.join(", "))
            }
            _ => v.to_string(),
        }
    }
    let snippet = to_snippet(&val);
    let tmp = format!("__v__ = {}", snippet);
    let parsed: DocumentMut = tmp
        .parse::<DocumentMut>()
        .map_err(|e| anyhow::anyhow!("Failed to parse serialized TOML value into Item: {e}"))?;
    let item: Item = parsed["__v__"].clone();
    doc[key] = item;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Client,
    LocalServer,
    DedicatedServer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsDomain {
    Default, // embedded defaults (global.toml)
    User,    // config/settings.toml (Client)
    Server,  // config/server.toml   (Server)
    Admin,   // config/admin.toml    (Server)
}

#[derive(Debug, Clone)]
pub struct DomainPaths {
    pub default_embedded: &'static str, // assets/settings/global.toml
    pub user_path: Option<PathBuf>,     // client: Some(...), server: optional/None
    pub server_path: Option<PathBuf>,   // server: Some(...), client: None
    pub admin_path: Option<PathBuf>,    // server: Some(...), client: None
}

pub fn resolve_domain_paths(role: NodeRole) -> DomainPaths {
    match role {
        NodeRole::Client => DomainPaths {
            default_embedded: "settings/global.toml",
            user_path: Some(paths::config_dir().join("settings.toml")),
            server_path: None,
            admin_path: None,
        },
        NodeRole::LocalServer => DomainPaths {
            default_embedded: "settings/global.toml",
            user_path: Some(paths::config_dir().join("settings.toml")),
            server_path: Some(paths::config_dir().join("server.toml")),
            admin_path: Some(paths::config_dir().join("admin.toml")),
        },
        NodeRole::DedicatedServer => DomainPaths {
            default_embedded: "settings/global.toml",
            user_path: None,
            server_path: Some(paths::config_dir().join("server.toml")),
            admin_path: Some(paths::config_dir().join("admin.toml")),
        },
    }
}

// Hilfsfunktionen: Dateien optional lesen/parsen
fn read_toml_file_opt(path: &Path) -> SettingsResult<Option<TomlValue>> {
    match std::fs::read_to_string(path) {
        Ok(s) => Ok(Some(toml::from_str(&s)?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

// Bootstrap: lädt je nach Rolle die passenden Quellen in den Store
pub fn init_store_for_role(role: NodeRole, store: &mut SettingsStore) -> SettingsResult<()> {
    // 1) Defaults (embedded) setzen
    let defaults = crate::assets::default_settings(); // Cow<'static, str>
    store.set_default_settings(&defaults)?;

    // 2) Domain-Pfade bestimmen
    let paths = resolve_domain_paths(role);

    // 3) User/Server/Admin (optional) in den Store setzen, wenn Dateien vorhanden
    if let Some(p) = paths.user_path.as_deref() {
        if let Some(val) = read_toml_file_opt(p)? {
            store.set_user_settings(&toml::to_string(&val)?)?;
        }
    }
    // Falls du in deinem Store schon server/admin-Setter hast, nutze sie:
    if let Some(p) = paths.server_path.as_deref() {
        if let Some(val) = read_toml_file_opt(p)? {
            // Wenn dein Store noch kein set_server_settings hat, kannst du’s später ergänzen.
            let _ = val; // store.set_server_settings(&toml::to_string(&val)?)?;
        }
    }
    if let Some(p) = paths.admin_path.as_deref() {
        if let Some(val) = read_toml_file_opt(p)? {
            let _ = val; // store.set_admin_settings(&toml::to_string(&val)?)?;
        }
    }

    Ok(())
}
