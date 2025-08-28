
Quickstart-Checkliste
- Definiere deine Setting-Typen als Structs und implementiere für jeden das Settings-Trait (mit KEY = TOML-Abschnitt).
- Erzeuge beim Start einen SettingsStore, boote ihn für die aktuelle Rolle (Client, LocalServer, DedicatedServer).
- Registriere alle Setting-Typen.
- Lesen: store.get::<T>(None) oder mit Save-Kontext.
- Schreiben: store.update_settings_file::<T>(&pfad, |fc| { … }); danach User-/Save-Datei neu in den Store laden.

1) Beispiel: ein Setting-Typ (Abschnitt [graphics])
```/dev/null/settings_types.rs#L1-120
use serde::{Serialize, Deserialize};
use crate::settings::{Settings, SettingsResult};
use crate::settings::source::SettingsSources;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphicsSettings {
    pub resolution: String,
    pub ui_scale: f32,
    pub fullscreen: bool,
    pub field_of_view: u32,
}
type GraphicsFileContent = GraphicsSettings;

impl Settings for GraphicsSettings {
    const KEY: Option<&'static str> = Some("graphics");
    type FileContent = GraphicsFileContent;

    fn load(src: SettingsSources<Self::FileContent>) -> SettingsResult<Self> {
        // Einfache Merge-Strategie: user > default (später: server/admin/save einbauen)
        let mut out = src.default.clone();
        if let Some(user) = src.user { out = user.clone(); }
        Ok(out)
    }
}
```

2) Bootstrap: Store erzeugen, Rolle-abhängig laden, Typen registrieren
```/dev/null/main_bootstrap.rs#L1-120
use std::fs;
use crate::settings::store::SettingsStore;
use crate::settings::read_write::{NodeRole, bootstrap_store_for_role};
use crate::settings::settings_file; // falls du einen Helper hast
use crate::your_mod::GraphicsSettings; // dein Setting-Typ

fn init_settings(role: NodeRole) -> anyhow::Result<SettingsStore> {
    let mut store = SettingsStore::new();

    // Lädt Defaults (embedded) und – je nach Rolle – die passenden Dateien in den Store
    bootstrap_store_for_role(role, &mut store)?;

    // Setting-Typen registrieren (für jeden TOML-Abschnitt)
    store.register_setting::<GraphicsSettings>();
    // store.register_setting::<GeneralSettings>();
    // store.register_setting::<AudioSettings>();

    Ok(store)
}
```

3) Lesen in deiner App
```/dev/null/read_settings.rs#L1-60
use crate::settings::store::SettingsStore;
use crate::your_mod::GraphicsSettings;

fn print_graphics(store: &SettingsStore) {
    let g = store.get::<GraphicsSettings>(None);
    println!("fullscreen: {}, fov: {}", g.fullscreen, g.field_of_view);
}
```

4) Schreiben (persistieren) und neu laden
- Nach dem Schreiben musst du die User-Datei in den Store zurückladen (damit get::<T>() den neuen Wert liefert).

```/dev/null/update_and_reload.rs#L1-120
use std::fs;
use crate::settings::{settings_file};
use crate::settings::store::SettingsStore;
use crate::your_mod::GraphicsSettings;

fn toggle_fullscreen(store: &SettingsStore) -> anyhow::Result<()> {
    let path = settings_file();

    // Datei format-preserving aktualisieren
    store.update_settings_file::<GraphicsSettings>(&path, |fc| {
        fc.fullscreen = !fc.fullscreen;
    })?;

    // Danach User-TOML neu in den Store laden (aktualisiert interne Werte)
    let new_text = fs::read_to_string(&path)?;
    // Je nach deiner Store-Implementierung: set_user_settings übernimmt das Recompute
    // Falls dein set_user_settings bereits recompute aufruft, reicht diese Zeile:
    // (Wenn nicht: baue eine store.refresh_all() oder rufe recompute_values)
    // Achtung: Signatur kann bei dir SettingsResult erfordern.
    // Hier als Beispiel:
    // store.set_user_settings(&new_text)?;
    Ok(())
}
```

5) Pro-Spielstand (Save) – optional
- Wenn du APIs wie set_local_settings_for_save / recompute_for_save hast, nutze sie so:
```/dev/null/save_usage.rs#L1-160
use std::fs;
use std::path::Path;
use crate::settings::store::SettingsStore;
use crate::settings::location::{SaveGameId, SettingsLocation};
use crate::settings::read_write::save_settings_path;
use crate::your_mod::GraphicsSettings;

fn load_save_settings(store: &mut SettingsStore, save_id: usize) -> anyhow::Result<()> {
    let path = save_settings_path(save_id);
    let text = fs::read_to_string(&path).unwrap_or_default();
    store.set_local_settings_for_save(SaveGameId(save_id), Path::new(&format!("saves/{save_id}")), &text)?;
    store.recompute_for_save(SaveGameId(save_id))?;
    Ok(())
}

fn read_graphics_for_save(store: &SettingsStore, save_id: usize) {
    let root = Path::new(&format!("saves/{save_id}"));
    let loc = SettingsLocation::new(SaveGameId(save_id), root);
    let g = store.get::<GraphicsSettings>(Some(loc));
    println!("FOV (save {save_id}): {}", g.field_of_view);
}
```

6) Server vs. Client
- Client
  - role = NodeRole::Client
  - lädt nur defaults + user (config/settings.toml)
  - autoritative Shared/Gameplay-Settings kommen vom Server über Netzwerk und landen in einer Bevy-Resource (unabhängig von der Datei)
- LocalServer
  - role = NodeRole::LocalServer
  - lädt defaults + user + server + admin; merges je nach Setting in T::load (z. B. server > user > default)
- DedicatedServer
  - role = NodeRole::DedicatedServer
  - lädt defaults + server + admin
  - Clients lesen die Server-/Save-Settings nicht über Dateien, sondern bekommen Snapshots/Updates über Netzwerk

7) Bevy-Integration (minimal)
- SettingsStore als Resource einhängen, Settings-Werte beim Startup anwenden.

```/dev/null/bevy_integration.rs#L1-120
use bevy::prelude::*;
use crate::settings::store::SettingsStore;
use crate::settings::read_write::NodeRole;

#[derive(Resource)]
pub struct SettingsRes(pub SettingsStore);

pub struct SettingsPlugin {
    pub role: NodeRole,
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        // Bootstrap und als Resource einfügen
        let mut store = SettingsStore::new();
        crate::settings::read_write::bootstrap_store_for_role(self.role, &mut store).unwrap();
        // store.register_setting::<...>(); // alle Typen registrieren
        app.insert_resource(SettingsRes(store));

        // Optional: System zum Anwenden (z. B. vsync etc.) eintragen
    }
}
```

Tipps und nächste sinnvolle Schritte
- Ergänze für jeden Setting-Typ die gewünschte Merge-Politik in T::load (z. B. Save > Server > User > Default).
- Baue dir kleine Helfer für Pfade:
  - settings_file() = paths::config_dir()/settings.toml
  - server_settings_file() = paths::config_dir()/server.toml
  - admin_settings_file() = paths::config_dir()/admin.toml
  - save_settings_path(save_id)
- Wenn du nach einem Update sofort aktualisierte Werte willst:
  - Stelle sicher, dass set_user_settings/set_local_settings_for_save intern recompute anstoßen, oder biete eine refresh_all()/recompute_for_save API an.

Wenn du mir sagst, welche Setting-Typen du konkret hast (graphics, audio, gameplay/shared etc.), schreibe ich dir die passenden Settings-Impls (mit KEYs) und die minimalen Merge-Regeln dafür.
