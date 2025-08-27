//! SettingsStore – TOML-basiert, Sync-I/O, minimaler Funktionsumfang.

use super::location::{SaveGameId, SettingsLocation};
use super::source::SettingsSources;
use super::value::{AnySettingValue, SettingValue};
use super::{Settings, SettingsError, SettingsResult};
use std::{
    any::{TypeId, type_name},
    collections::{HashMap, hash_map},
    fs,
    path::Path,
};
use toml::Value as TomlValue;
use toml_edit::DocumentMut;

/// Der zentrale Settings-Container.
pub struct SettingsStore {
    setting_values: HashMap<TypeId, Box<dyn AnySettingValue>>,
    raw_default_settings: TomlValue,
    raw_user_settings: TomlValue,
    raw_server_settings: TomlValue, // NEU
    raw_admin_settings: TomlValue,  // NEU
}

impl SettingsStore {
    pub fn new() -> Self {
        Self {
            setting_values: HashMap::new(),
            raw_default_settings: TomlValue::Table(Default::default()),
            raw_user_settings: TomlValue::Table(Default::default()),
            raw_server_settings: TomlValue::Table(Default::default()),
            raw_admin_settings: TomlValue::Table(Default::default()),
        }
    }
    pub fn set_default_settings(&mut self, toml_text: &str) -> SettingsResult<()> {
        let v: TomlValue = toml::from_str(toml_text)?;
        if !v.is_table() {
            return Err(SettingsError::InvalidStructure("defaults must be table"));
        }
        self.raw_default_settings = v;
        Ok(())
    }

    pub fn set_user_settings(&mut self, toml_text: &str) -> SettingsResult<()> {
        let v: TomlValue = if toml_text.is_empty() {
            TomlValue::Table(Default::default())
        } else {
            toml::from_str(toml_text)?
        };
        if !v.is_table() {
            return Err(SettingsError::InvalidStructure("user must be table"));
        }
        self.raw_user_settings = v;
        Ok(())
    }

    pub fn set_server_settings(&mut self, toml_text: &str) -> SettingsResult<()> {
        let v: TomlValue = if toml_text.is_empty() {
            TomlValue::Table(Default::default())
        } else {
            toml::from_str(toml_text)?
        };
        if !v.is_table() {
            return Err(SettingsError::InvalidStructure("server must be table"));
        }
        self.raw_server_settings = v;
        Ok(())
    }

    pub fn set_admin_settings(&mut self, toml_text: &str) -> SettingsResult<()> {
        let v: TomlValue = if toml_text.is_empty() {
            TomlValue::Table(Default::default())
        } else {
            toml::from_str(toml_text)?
        };
        if !v.is_table() {
            return Err(SettingsError::InvalidStructure("admin must be table"));
        }
        self.raw_admin_settings = v;
        Ok(())
    }

    /// Registriert einen Setting-Typ T (lädt default+user und berechnet globalen Wert).
    pub fn register_setting<T: Settings>(&mut self) {
        let type_id = TypeId::of::<T>();
        if let hash_map::Entry::Occupied(_) = self.setting_values.entry(type_id) {
            return;
        }

        let entry = self.setting_values.entry(type_id).or_insert_with(|| {
            Box::new(SettingValue::<T> {
                global_value: None,
                local_values: vec![],
            })
        });

        let default = entry.deserialize_setting(&self.raw_default_settings);
        if let Some(default) = default {
            let user = entry.deserialize_setting(&self.raw_user_settings);
            let server = entry.deserialize_setting(&self.raw_server_settings);
            let admin = entry.deserialize_setting(&self.raw_admin_settings);

            // Alle Quellen an T::load übergeben; T entscheidet die Merge-Priorität
            match entry.load_setting(SettingsSources {
                default: &default,
                user: user.as_ref(),
                server: server.as_ref(),
                admin: admin.as_ref(),
            }) {
                Ok(value) => entry.set_global_value(value),
                Err(_) => panic!("missing default for {}", type_name::<T>()),
            }
        } else {
            panic!("missing default for {}", type_name::<T>());
        }
    }

    /// Liefert das getypte Setting (global) zurück.
    pub fn get<T: Settings>(&self, path: Option<SettingsLocation>) -> &T {
        self.setting_values
            .get(&TypeId::of::<T>())
            .unwrap_or_else(|| panic!("unregistered setting type {}", type_name::<T>()))
            .value_for_path(path)
            .downcast_ref::<T>()
            .expect("wrong setting type stored")
    }

    /// Lädt die User-Settings von Disk (oder Default, wenn Datei fehlt).
    pub fn load_or_default_from_path(
        &mut self,
        path: &Path,
        default_text: &str,
    ) -> SettingsResult<()> {
        // Schrittweise:
        // 1) Datei lesen; wenn nicht vorhanden: default_text verwenden
        // 2) set_user_settings(...) aufrufen
        let text = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => default_text.to_string(),
            Err(e) => return Err(e.into()),
        };
        self.set_user_settings(&text)
    }

    /// Persistierendes Update eines Settings in der Datei (format-preserving mit toml_edit).
    pub fn update_settings_file<T: Settings + 'static>(
        &self,
        path: &Path,
        mutator: impl FnOnce(&mut T::FileContent),
    ) -> SettingsResult<()> {
        // wie bisher: DocumentMut laden, FileContent extrahieren, mutieren, via set_key_to_serialized_item einfügen, schreiben
        let mut doc: DocumentMut = fs::read_to_string(path)
            .ok()
            .and_then(|s| s.parse::<DocumentMut>().ok())
            .unwrap_or_else(DocumentMut::new);
        let current: T::FileContent = if let Some(key) = T::KEY {
            match doc
                .get(key)
                .and_then(|i| i.as_value())
                .map(|v| v.to_string())
            {
                Some(s) => toml::from_str(&s).unwrap_or_default(),
                None => Default::default(),
            }
        } else {
            toml::from_str(&doc.to_string()).unwrap_or_default()
        };
        let mut new_content = current;
        mutator(&mut new_content);
        if let Some(key) = T::KEY {
            super::read_write::set_key_to_serialized_item(&mut doc, key, &new_content);
        } else {
            return Err(SettingsError::NotImplemented);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, doc.to_string())?;
        Ok(())
    }

    /// (Optional) Alle Settings neu berechnen – MVP vorerst leer.
    pub fn recompute_values(
        &mut self,
        _changed_local_path: Option<(SaveGameId, &Path)>,
    ) -> SettingsResult<()> {
        // Schrittweise (später):
        // 1) Für jeden Setting-Typ default/user/etc. neu laden
        // 2) T::load anwenden
        // 3) global/local Werte setzen
        Ok(())
    }
}
