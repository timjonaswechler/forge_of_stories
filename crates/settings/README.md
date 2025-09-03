# Settings & Keymaps – Kurz-API

Dieses Crate liefert ein schichtbasiertes (last‑wins) Settings- und Keymap-System mit atomischen, robusten Writes und einfacher TYP‑API.

## Quickstart

Rust-Beispiel:

```/dev/null/example.rs#L1-80
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use settings::{
    // Re-exports auf Crate-Root:
    SettingsStore, Settings, DeviceFilter,
    // Eingebettete Defaults:
    default_settings, default_keymap,
};

// 1) Store aufbauen (Defaults + Nutzerdateien unter config_dir)
let store = SettingsStore::builder()
    .with_embedded_setting_text(default_settings())
    .with_embedded_keymap_text(default_keymap())
    .with_user_config_dir() // <config_dir>/settings.toml + <config_dir>/keymap.toml
    .build()?;

// 2) Section-Typ definieren
#[derive(Clone, Default, Serialize, Deserialize)]
struct NetworkCfg { port: u16 }

struct Network;
impl Settings for Network {
    const SECTION: &'static str = "network";
    type Model = NetworkCfg;
}

// 3) Registrieren und lesen
store.register::<Network>()?;
let net: Arc<NetworkCfg> = store.get::<Network>()?;
assert_eq!(net.port, 0);

// 4) Aktualisieren (nur Delta zu Defaults wird geschrieben)
store.update::<Network>(|m| { m.port = 7777; })?;

// 5) Keymap-Export (global + Kontext, Kontext last-wins je Gerät)
let kb = store.export_keymap_for(DeviceFilter::Keyboard, "in_game");
// -> BTreeMap<String, Vec<String>>
```

## Builder – wichtigste Methoden

- Settings-Layer
  - `.with_embedded_setting_text(toml: impl Into<String>)`
  - `.with_embedded_setting_asset(path: &'static str)` – über `rust-embed`
  - `.with_settings_file(path: PathBuf)`
  - `.with_settings_file_optional(path: PathBuf)` – fehlend/leer ist neutral
- Keymap-Layer
  - `.with_embedded_keymap_text(toml)`
  - `.with_embedded_keymap_asset(path)`
  - `.with_keymap_file(path)`
  - `.with_keymap_file_optional(path)`
- Plattformpfade
  - `.with_user_config_dir()` – nutzt `paths::config_dir()` und fügt
    - `<config_dir>/settings.toml`
    - `<config_dir>/keymap.toml`
    hinzu
- Optionen
  - `.enable_env_layers(bool)` – aktiviert/deaktiviert Environment-Layer (derzeit ohne Mapping-Logik)

Hinweis: Layer-Reihenfolge ist Priorität – spätere Layer überschreiben frühere (last‑wins).

## Merge-Regeln (Settings)

- Pro Top‑Level‑Section wird tief gemerged:
  - Tabellen: rekursiv, last‑wins je Schlüssel
  - Skalare: ersetzen (last‑wins)
  - Arrays: standardmäßig ersetzen (konfigurierbar via `merge_arrays_policy(...)`)

## Keymaps – Daten und Export

- Devices: Keyboard, Mouse, Gamepad (Filter via `DeviceFilter`)
- Kontexte: `global` + beliebige weitere (z. B. `in_game`)
- Chords: Strings mit optionalem Präfix (`xbox:`, `dualshock:`, `gp:`/`gamepad:`)
- Export: `export_keymap_for(device, context)` liefert Actions → Liste von Chords
  - Zusammenführung: `global` + `context`, Kontext überschreibt je Gerät
  - Stabile Deduplizierung
  - Gamepad-Filter: bei `DeviceFilter::GamepadKind("xbox")` werden generische `gp:` ebenfalls berücksichtigt

## Persistenz

- Writes sind atomar und haltbar (Tempfile, `sync_all`, POSIX `rename` + Dir‑fsync, Windows `ReplaceFileW`).
- `update<T>` schreibt nur Abweichungen von eingebetteten Defaults; Rückkehr zum Default entfernt die Section aus der Datei.