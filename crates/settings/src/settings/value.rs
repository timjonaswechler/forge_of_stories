//! Interner Typ-Erasure-Adapter für Settings (T -> dyn AnySettingValue).
//! MVP: Minimal gehalten, später erweiterbar (lokale Werte etc.).

use super::location::{SaveGameId, SettingsLocation};
use super::source::SettingsSources;
use super::{Settings, SettingsError};
use serde::de::DeserializeOwned;
use std::{
    any::{Any, type_name},
    path::Path,
    sync::Arc,
};
use toml::Value as TomlValue;

/// Container pro Setting-Typ (global + optional lokale Werte).
#[derive(Debug)]
pub struct SettingValue<T> {
    pub global_value: Option<T>,
    pub local_values: Vec<(SaveGameId, Arc<Path>, T)>, // MVP: ungenutzt, aber API steht
}

/// Typ-erased Box für Deserialisierung.
#[derive(Debug)]
pub struct DeserializedSetting(pub Box<dyn Any>);

/// Objekt-sichere Operationen auf einem Setting.
pub trait AnySettingValue: 'static + Send + Sync {
    fn key(&self) -> Option<&'static str>;
    fn setting_type_name(&self) -> &'static str;

    fn deserialize_setting(&self, root: &TomlValue) -> Option<DeserializedSetting>;
    fn load_setting(
        &self,
        sources: SettingsSources<DeserializedSetting>,
    ) -> Result<Box<dyn Any>, SettingsError>;

    fn value_for_path(&self, _path: Option<SettingsLocation>) -> &dyn Any;
    fn all_local_values(&self) -> Vec<(SaveGameId, Arc<Path>, &dyn Any)>;

    fn set_global_value(&mut self, value: Box<dyn Any>);
    fn set_local_value(&mut self, root_id: SaveGameId, path: Arc<Path>, value: Box<dyn Any>);
}

impl<T> AnySettingValue for SettingValue<T>
where
    T: Settings + 'static,
    T::FileContent: DeserializeOwned,
{
    fn key(&self) -> Option<&'static str> {
        T::KEY
    }

    fn setting_type_name(&self) -> &'static str {
        type_name::<T>()
    }

    fn deserialize_setting(&self, root: &TomlValue) -> Option<DeserializedSetting> {
        // Schrittweise:
        // 1) Root (TOML) → passenden Teilbaum extrahieren:
        //    - Falls T::KEY Some(k): root[k], ansonsten root selbst.
        // 2) Diesen Teil mit toml::Value → T::FileContent (serde) deserialisieren.
        // 3) In Box<dyn Any> verpacken und zurückgeben.
        //
        // MVP: nur happy-path; bei Strukturfehlern -> None.
        let v = if let Some(k) = T::KEY {
            root.get(k)?
        } else {
            root
        };
        let fc: T::FileContent = toml::from_str(&v.to_string()).ok()?;
        Some(DeserializedSetting(Box::new(fc)))
    }

    fn load_setting(
        &self,
        sources: SettingsSources<DeserializedSetting>,
    ) -> Result<Box<dyn Any>, SettingsError> {
        // Schrittweise:
        // 1) default: &T::FileContent
        // 2) user: Option<&T::FileContent>
        // 3) T::load mit diesen Quellen aufrufen -> T
        // 4) Box<dyn Any> zurückgeben
        let default = sources
            .default
            .0
            .downcast_ref::<T::FileContent>()
            .expect("type mismatch in default source");
        let user = sources.user.map(|u| {
            u.0.downcast_ref::<T::FileContent>()
                .expect("type mismatch in user source")
        });
        let server = sources.server.map(|s| {
            s.0.downcast_ref::<T::FileContent>()
                .expect("type mismatch in server source")
        });
        let admin = sources.admin.map(|a| {
            a.0.downcast_ref::<T::FileContent>()
                .expect("type mismatch in admin source")
        });

        let t = T::load(SettingsSources {
            default,
            user,
            server,
            admin,
        })?;
        Ok(Box::new(t))
    }

    fn value_for_path(&self, _path: Option<SettingsLocation>) -> &dyn Any {
        // MVP: nur globaler Wert.
        self.global_value
            .as_ref()
            .expect("no global value set for setting (register_setting not called?)")
    }

    fn all_local_values(&self) -> Vec<(SaveGameId, Arc<Path>, &dyn Any)> {
        self.local_values
            .iter()
            .map(|(id, path, v)| (*id, path.clone(), v as &dyn Any))
            .collect()
    }

    fn set_global_value(&mut self, value: Box<dyn Any>) {
        self.global_value = Some(*value.downcast::<T>().expect("wrong value type"));
    }

    fn set_local_value(&mut self, root_id: SaveGameId, path: Arc<Path>, value: Box<dyn Any>) {
        // MVP: API vorhanden, Implementierung minimal.
        let v = *value.downcast::<T>().expect("wrong value type");
        if let Some(ix) = self
            .local_values
            .iter()
            .position(|(rid, p, _)| *rid == root_id && *p == path)
        {
            self.local_values[ix].2 = v;
        } else {
            self.local_values.push((root_id, path, v));
        }
    }
}
