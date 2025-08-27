//! UI-unabhängiges EditableSettingControl – MVP-Variante ohne gpui.
//! Default write() nutzt den Store und update_settings_file.

use crate::settings::{Settings, SettingsError, store::SettingsStore};
use std::path::Path;

pub trait EditableSettingControl {
    /// Wert, der in der UI verändert wird.
    type Value: Send + 'static;
    /// Zugehöriges Setting.
    type Setting: Settings;

    /// Anzeigename/Label in der UI (oder intern).
    fn name(&self) -> &'static str;

    /// Aktuellen Wert aus dem Store lesen (z. B. für UI-Initialisierung).
    fn read(store: &SettingsStore) -> Self::Value;

    /// Semantik: Wie wird ein Value in das FileContent überführt?
    fn apply(file: &mut <Self::Setting as Settings>::FileContent, value: Self::Value);

    /// Default-Schreiblogik: Settings-Datei auf Pfad aktualisieren.
    fn write(
        store: &SettingsStore,
        settings_path: &Path,
        value: Self::Value,
    ) -> Result<(), SettingsError> {
        // Schrittweise:
        // 1) Closure bauen, die `apply` auf FileContent anwendet
        // 2) store.update_settings_file::<Self::Setting>(settings_path, closure)
        store.update_settings_file::<Self::Setting>(settings_path, |file| {
            Self::apply(file, value);
        })
    }
}
