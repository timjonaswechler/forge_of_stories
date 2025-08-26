mod location;
pub mod source;
mod store;
pub(crate) mod toml;
mod value;

use crate::settings::source::SettingsSources;
use crate::settings::store::SettingsStore;
pub use location::SettingsLocation;
use serde::{Serialize, de::DeserializeOwned};

/// A value that can be defined as a user setting.
///
/// Settings can be loaded from a combination of multiple JSON files.
pub trait Settings: 'static + Send + Sync {
    /// The name of a key within the JSON file from which this setting should
    /// be deserialized. If this is `None`, then the setting will be deserialized
    /// from the root object.
    const KEY: Option<&'static str>;

    const FALLBACK_KEY: Option<&'static str> = None;

    /// The name of the keys in the [`FileContent`](Self::FileContent) that should
    /// always be written to a settings file, even if their value matches the default
    /// value.
    ///
    /// This is useful for tagged [`FileContent`](Self::FileContent)s where the tag
    /// is a "version" field that should always be persisted, even if the current
    /// user settings match the current version of the settings.
    const PRESERVED_KEYS: Option<&'static [&'static str]> = None;

    /// The type that is stored in an individual JSON file.
    type FileContent: Clone + Default + Serialize + DeserializeOwned;

    /// The logic for combining together values from one or more JSON files into the
    /// final value for this setting.
    fn load(sources: SettingsSources<Self::FileContent>) -> Result<Self>
    where
        Self: Sized;

    fn missing_default() -> anyhow::Error {
        anyhow::anyhow!("missing default")
    }

    #[track_caller]
    fn register(cx: &mut App)
    where
        Self: Sized,
    {
        SettingsStore::update_global(cx, |store, cx| {
            store.register_setting::<Self>(cx);
        });
    }

    #[track_caller]
    fn get<'a>(path: Option<SettingsLocation>, cx: &'a App) -> &'a Self
    where
        Self: Sized,
    {
        cx.global::<SettingsStore>().get(path)
    }
}
