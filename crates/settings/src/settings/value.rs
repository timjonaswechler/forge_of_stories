use super::{
    Settings,
    location::{SaveGameId, SettingsLocation},
    source::SettingsSources,
};
use anyhow::Result;
use serde::Deserialize;
use std::{
    any::{Any, type_name},
    ops::Range,
    path::Path,
    sync::Arc,
};
use toml_edit::Value;

#[derive(Debug)]
pub struct SettingValue<T> {
    pub global_value: Option<T>,
    pub local_values: Vec<(SaveGameId, Arc<Path>, T)>,
}

#[derive(Debug)]
pub struct DeserializedSetting(pub Box<dyn Any>);

pub trait AnySettingValue: 'static + Send + Sync {
    fn key(&self) -> Option<&'static str>;
    fn setting_type_name(&self) -> &'static str;
    fn deserialize_setting(&self, json: &Value) -> Result<DeserializedSetting> {
        self.deserialize_setting_with_key(json).1
    }
    fn deserialize_setting_with_key(
        &self,
        json: &Value,
    ) -> (Option<&'static str>, Result<DeserializedSetting>);
    fn load_setting(&self, sources: SettingsSources<DeserializedSetting>) -> Result<Box<dyn Any>>;
    fn value_for_path(&self, path: Option<SettingsLocation>) -> &dyn Any;
    fn all_local_values(&self) -> Vec<(SaveGameId, Arc<Path>, &dyn Any)>;
    fn set_global_value(&mut self, value: Box<dyn Any>);
    fn set_local_value(&mut self, root_id: SaveGameId, path: Arc<Path>, value: Box<dyn Any>);
    fn edits_for_update(
        &self,
        raw_settings: &Value,
        tab_size: usize,
        text: &mut String,
        edits: &mut Vec<(Range<usize>, String)>,
    );
}

impl<T: Settings> AnySettingValue for SettingValue<T> {
    fn key(&self) -> Option<&'static str> {
        T::KEY
    }

    fn setting_type_name(&self) -> &'static str {
        type_name::<T>()
    }

    fn load_setting(&self, values: SettingsSources<DeserializedSetting>) -> Result<Box<dyn Any>> {
        Ok(Box::new(T::load(SettingsSources {
            default: values.default.0.downcast_ref::<T::FileContent>().unwrap(),
            global: values
                .global
                .map(|value| value.0.downcast_ref::<T::FileContent>().unwrap()),
            extensions: values
                .extensions
                .map(|value| value.0.downcast_ref::<T::FileContent>().unwrap()),
            user: values
                .user
                .map(|value| value.0.downcast_ref::<T::FileContent>().unwrap()),
            server: values
                .server
                .map(|value| value.0.downcast_ref::<T::FileContent>().unwrap()),
            project: &[],
        })?))
    }

    fn deserialize_setting_with_key(
        &self,
        mut toml: &Value,
    ) -> (Option<&'static str>, Result<DeserializedSetting>) {
        let mut key = None;
        if let Some(k) = T::KEY {
            if let Some(value) = toml.get(k) {
                toml = value;
                key = Some(k);
            } else if let Some((k, value)) = T::FALLBACK_KEY.and_then(|k| Some((k, toml.get(k)?))) {
                toml = value;
                key = Some(k);
            } else {
                let value = T::FileContent::default();
                return (T::KEY, Ok(DeserializedSetting(Box::new(value))));
            }
        }
        let value = T::FileContent::deserialize(toml)
            .map(|value| DeserializedSetting(Box::new(value)))
            .map_err(anyhow::Error::from);
        (key, value)
    }

    fn all_local_values(&self) -> Vec<(SaveGameId, Arc<Path>, &dyn Any)> {
        self.local_values
            .iter()
            .map(|(id, path, value)| (*id, path.clone(), value as _))
            .collect()
    }

    fn value_for_path(&self, path: Option<SettingsLocation>) -> &dyn Any {
        if let Some(SettingsLocation { savegame_id, path }) = path {
            for (settings_root_id, settings_path, value) in self.local_values.iter().rev() {
                if savegame_id == *settings_root_id && path.starts_with(settings_path) {
                    return value;
                }
            }
        }
        self.global_value
            .as_ref()
            .unwrap_or_else(|| panic!("no default value for setting {}", self.setting_type_name()))
    }

    fn set_global_value(&mut self, value: Box<dyn Any>) {
        self.global_value = Some(*value.downcast().unwrap());
    }

    fn set_local_value(&mut self, root_id: SaveGameId, path: Arc<Path>, value: Box<dyn Any>) {
        let value = *value.downcast().unwrap();
        match self
            .local_values
            .binary_search_by_key(&(root_id, &path), |e| (e.0, &e.1))
        {
            Ok(ix) => self.local_values[ix].2 = value,
            Err(ix) => self.local_values.insert(ix, (root_id, path, value)),
        }
    }

    fn edits_for_update(
        &self,
        _raw_settings: &Value,
        _tab_size: usize,
        _text: &mut String,
        _edits: &mut Vec<(Range<usize>, String)>,
    ) {
        // TOML-Migration: Die format-preserving Updates erfolgen zentral im Store
        // mittels toml_edit::DocumentMut. Diese Methode liefert daher aktuell
        // keine Edits zurück. Der Store erzeugt die finalen Änderungen.
        // Wenn du später feingranulare Edits hier haben willst, kannst du:
        // - das aktuelle T::FileContent aus _raw_settings herausziehen,
        // - die Differenzen ermitteln,
        // - gezielte Ranges/Replacement-Strings basierend auf toml_edit erzeugen.
    }
}
