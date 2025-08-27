# Zed Settings - Kernidee & Konzept

## Die Grundidee

Zed implementiert ein **hierarchisches, typsicheres Settings-System** mit folgenden Kernprinzipien:

1. **Typsicherheit**: Jede Einstellung hat einen konkreten Rust-Typ
2. **Hierarchie**: Settings werden von global → lokal vererbt und überschrieben
3. **TOML-basiert**: Alle Settings werden in TOML-Dateien gespeichert
4. **Live-Updates**: Änderungen werden automatisch erkannt und angewendet

## Pseudo-Code: Das Kernkonzept

```rust
// === 1. SETTINGS DEFINITION ===
trait Settings {
    // Wo in der TOML-Datei steht diese Setting?
    KEY = "editor"  // z.B. [editor]

    // Wie wird aus TOML der Rust-Typ gemacht?
    function load(toml_sources: TomlHierarchy) -> Self

    // Was ist der Default-Wert?
    function default() -> Self
}

// Beispiel für eine konkrete Setting
struct EditorSettings {
    font_size: u32,
    theme: String,
    tab_size: u32,
}

impl Settings for EditorSettings {
    KEY = "editor""

    fn load(sources) {
        // Nimm Default-Werte
        result = EditorSettings::default()

        // Überschreibe mit Global Settings
        if sources.has_global() {
            result.merge_with(sources.global["editor"])
        }

        // Überschreibe mit User Settings
        if sources.has_user() {
            result.merge_with(sources.user["editor"])
        }

        // Überschreibe mit Project Settings
        if sources.has_project() {
            result.merge_with(sources.project["editor"])
        }

        return result
    }
}

// === 2. SETTINGS STORE - DER ZENTRALE MANAGER ===
struct SettingsStore {
    registered_settings: Map<TypeName, SettingValue>,
    toml_files: {
        default: TomlValue,
        user: TomlValue,
        global: TomlValue,
        project: Map<ProjectPath, TomlValue>,
        local: Map<FilePath, TomlValue>
    },
    file_watchers: List<FileWatcher>
}

impl SettingsStore {
    // Setting registrieren
    fn register<T: Settings>(default_value: T) {
        this.registered_settings[T::type_name()] = SettingValue::new<T>(default_value)

        // File watcher installieren für Live-Updates
        this.watch_settings_files()
    }

    // Setting abrufen (mit Hierarchie)
    fn get<T: Settings>(context: Optional<FileContext>) -> T {
        setting_value = this.registered_settings[T::type_name()]

        // TOML-Hierarchie für diesen Kontext bauen
        sources = TomlHierarchy {
            default: this.toml_files.default,
            global: this.toml_files.global,
            user: this.toml_files.user,
            project: this.get_project_settings(context),
            local: this.get_local_settings(context)
        }

        // Setting laden mit Hierarchie
        return T::load(sources)
    }

    // Setting aktualisieren
    fn update_setting<T: Settings>(key_path: String, new_value: TomlValue) {
        // Finde passende TOML-Datei (meist user settings)
        target_file = this.toml_files.user

        // Update TOML
        target_file = toml_update(target_file, key_path, new_value)

        // Schreibe Datei
        write_file("~/.config/zed/settings.toml", target_file)

        // Trigger reload (wird automatisch durch file watcher gemacht)
        this.reload_from_files()
    }
}

// === 3. HIERARCHIE-SYSTEM ===
struct TomlHierarchy {
    default: TomlValue,    // Eingebaute Defaults
    global: TomlValue,     // System-weite Settings
    user: TomlValue,       // User's globale Settings
    project: TomlValue,    // Project-spezifische Settings
    local: TomlValue       // Datei-spezifische Settings
}

impl  TomlHierarchy {
    fn merge_for_setting(setting_key: String) -> TomlValue {
        result = TomlValue::empty()

        // Hierarchie: später überschreibt früher
        result.merge_with(this.default[setting_key])
        result.merge_with(this.global[setting_key])
        result.merge_with(this.user[setting_key])
        result.merge_with(this.project[setting_key])
        result.merge_with(this.local[setting_key])

        return result
    }
}

// === 4. USAGE PATTERN ===
fn main() {
    // Store initialisieren
    store = SettingsStore::new()

    // Settings registrieren
    store.register<EditorSettings>(EditorSettings::default())
    store.register<TerminalSettings>(TerminalSettings::default())
    store.register<KeymapSettings>(KeymapSettings::default())

    // Settings laden
    store.load_from_files()

    // Usage im Code:
    while app_running {
        // Setting abrufen - bekommt immer den aktuellen Wert
        // basierend auf der Hierarchie für den aktuellen Kontext
        current_file = get_current_file()
        editor_settings = store.get<EditorSettings>(current_file.context)

        // Mit Settings arbeiten
        set_font_size(editor_settings.font_size)
        set_theme(editor_settings.theme)

        // Bei Änderung der Datei oder Settings automatisch neu laden
    }
}

// === 5. LIVE UPDATES ===
fn setup_file_watching(store: SettingsStore) {
    // User settings beobachten
    watch_file("~/.config/zed/settings.toml") {
        on_change: || {
            store.reload_user_settings()
            notify_all_observers("settings_changed")
        }
    }

    // Project settings beobachten
    watch_file(".zed/settings.toml") {
        on_change: || {
            store.reload_project_settings()
            notify_all_observers("settings_changed")
        }
    }
}
```

## Die Kernidee in einem Satz

**"Jede Setting ist ein typsicherer Rust-Struct, der aus einer TOML-Hierarchie (default → global → user → project → local) geladen wird und automatisch bei Dateiänderungen neu berechnet wird."**

## Warum diese Architektur?

1. **Typsicherheit**: Compiler verhindert Fehler bei Settings-Zugriff
2. **Flexibilität**: Verschiedene Kontexte können verschiedene Werte haben
3. **Benutzerfreundlich**: TOML ist menschenlesbar, kommentierbar und editierbar
4. **Performance**: Lazy loading + Caching nur bei Änderungen
5. **Erweiterbar**: Neue Settings einfach als Trait implementierbar

## Beispiel-Vererbung

```toml
# Default Settings
[editor]
font_size = 12
theme = "light"

# User Settings
[editor]
font_size = 14

# Project Settings
[editor]
theme = "dark"

# Lokale Settings
[editor]
font_size = 16

# Resultat
[editor]
font_size = 16  # <- lokal überschreibt
theme = "dark"  # <- project überschreibt user
```

Das ist die Kernarchitektur des Zed Settings Systems!
