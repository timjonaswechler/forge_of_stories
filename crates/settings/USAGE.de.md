# Settings & Keymaps – Nutzung und Gestaltung

Diese Datei erklärt (auf Deutsch) wie das Settings‑System funktioniert und wie du flexible Keymaps aufbauen kannst. Sie richtet sich an Entwickler:innen des Projekts sowie an Power‑User, die eigene Profile/Layouts pflegen wollen.

> Hinweis: Einige Details beruhen auf Konventionen des Repos und typischen Patterns (Assumption). Falls ein Name geringfügig anders ist, bitte im Code (`crates/settings/src`) gegenprüfen.

## 1. Grundidee des Settings-Systems

Das Settings‑Crate bündelt konfigurierbare Werte in versionierbaren TOML‑Dateien. Es bietet:
- Mehrstufige *Defaults* (globale + domänenspezifische `*-default.toml`).
- Nutzer‑Overrides (benutzerverzeichnis / projektlokal / Laufzeit‑Injektion).
- (Vermutlich) Hot‑Reload oder zumindest einfaches Neuladen beim Start.
- Typsichere Deserialisierung (serde) in Rust‑Strukturen.

Typische Dateien (aus `assets/settings/`):
- `global.toml` – Globale gemeinsame Werte (z.B. Pfade, Feature‑Flags, Theme Basis).
- `aether-default.toml` – Defaults für den Aether / Engine Layer.
- `cli-default.toml` – Spezifische CLI Defaults.
- `client-default.toml` – Client/App spezifische Defaults.
- `wizard-default.toml` – Wizard UI spezifische Defaults.

## 2. Ladeschichten (Layering / Priority)

Empfohlene (und übliche) Reihenfolge – spätere Ebenen überschreiben frühere:
1. Kompilierte *eingebettete* Defaults (Failsafe – z.B. via `include_bytes!` oder `embedded.rs`).
2. Projekt‑Assets `assets/settings/*-default.toml`.
3. `assets/settings/global.toml` (falls globale Basis getrennt behandelt wird, kann auch Schritt 2 zusammenfallen – je nach Implementierung).
4. Benutzer‑Konfiguration im Schreibbaren Pfad (z.B. `$HOME/Library/Application Support/forge_of_stories/` oder XDG: `~/.config/forge_of_stories/`).
5. Laufzeit‑Overrides (z.B. CLI Flags, Environment Variablen, UI Wizard Änderungen).

Konfliktauflösung: „Letzter gewinnt“ je Key. Teilstrukturen werden _gemerged_, nicht zwangsläufig vollständig ersetzt (angenommen: serde's `merge` / manuelles Zusammenführen). Fehlt ein Key in einer oberen Ebene, bleibt der Wert weiter unten bestehen.

## 3. Typischer Ladeablauf (High Level)

Pseudocode (vereinfacht):
```
let base = Embedded::load_defaults();
let asset_defaults = Disk::read_dir("assets/settings");
let merged = merge(base, asset_defaults);
let user_layer = try_read(user_config_path());
let merged = merge(merged, user_layer);
let runtime = collect_runtime_overrides();
let final_settings = merge(merged, runtime);
store.replace(final_settings);
```

`store.rs` stellt vermutlich einen (Arc/RwLock) Zugriff bereit, sodass überall `Settings::get()` o.ä. benutzt werden kann.

## 4. Erweiterung neuer Settings

1. Rust‑Struktur in `settings.rs` / passender Mod hinzufügen.
2. `#[derive(Serialize, Deserialize, Default, Debug)]` nutzen.
3. Feld mit sinnvollem Default (entweder im Code oder via `*-default.toml`).
4. Passende Sektion in einer `*-default.toml` ergänzen.
5. Falls UI/Wizard das Setting ändert: Adapter in `bevy_adapter.rs` oder UI‑Code anpassen, damit Änderung zurück in den `store` geschrieben wird.

### Best Practices
- Namen in TOML „snake_case“ halten.
- Komplexe Werte (Listen, Maps) bevorzugen statt zusammengesetzter Strings.
- Keine relativen Pfade ohne klaren Anker (nutze prefix wie `data_dir`, `cache_dir`).

## 5. Pfade & Speicherorte

Empfohlenes Schema (macOS Beispiel):
- System/Repo Defaults: `assets/settings/`
- Benutzer: `~/Library/Application Support/forge_of_stories/settings/` (oder XDG: `~/.config/forge_of_stories/`)
- Temporäre Laufzeitdumps / Export: `.../forge_of_stories/settings/export/`.

Beim Start: sicherstellen, dass Benutzer‑Ordner existiert (ggf. anlegen). Bei erstmaligem Start können Defaults dorthin kopiert werden – aber nur, falls keine Nutzerdatei existiert (um lokale Änderungen nicht zu überschreiben).

## 6. Updates & Migration

Versioniere ein Schemafeld, z.B. `schema_version = 1`. Bei inkompatiblen Änderungen:
1. Erhöhe Version.
2. Lade alte Datei, führe Migration (Mapping alt->neu) aus.
3. Speichere neue Struktur zurück.

## 7. Zugriff im Code (Beispiel)

Angenommen es gibt einen globalen Singleton:
```
let s = settings::store(); // oder SettingsStore::global()
let theme = s.read().ui.theme.clone();
```

Oder Event‑basiert (Bevy): Resource injizieren und dann lesen.

## 8. Schreiben / Persistieren

Wenn der Nutzer einen Wert ändert:
1. Mutieren der In‑Memory Struktur.
2. Unterschied (Diff) gegen Defaults ermitteln – nur veränderte Keys in Benutzerdatei speichern (reduziert Merge‑Konflikte bei künftigen Default‑Updates).
3. Speichern (atomic write: erst `.tmp` Datei, dann rename) um Datenverlust zu vermeiden.

## 9. Fehlerbehandlung

- Ungültige TOML: Datei umbenennen nach `*.broken-YYYYmmddHHMM.toml` und Fallback nutzen.
- Fehlender Schlüssel: mit Default auffüllen und optional Warn loggen.
- Serde Fehler klar loggen (Dateiname + Abschnitt + Feld).

---

## 10. Keymap-System – Aktueller Stand (korrigiert)

Dieser Abschnitt ist eine korrigierte Version (frühere Fassung enthielt geplante Features: Sequenzen, Trie, Layer-Vererbung, Klammerausdrücke). Hier steht nur das, was der Code aktuell unterstützt.

### 10.1 Unterstützte Konzepte
- Aktionen: Identifiziert durch String (`action_name`).
- BindingGroup: Hat genau ein `context` Feld (String mit einfacher Bool-Logik: `&&`, `||`, `!`, KEINE Klammern).
- Actions-Map: `action_name -> [KeyDefinition, ...]`.
- KeyDefinition: Felder `key` (z.B. `ctrl+s`), optional `device` (`Keyboard|Mouse|Gamepad`), optional `action_data` (beliebiges TOML Value, wird durchgereicht).
- Priorität: Spätere Einträge in der Datei haben höhere Priorität (wird durch Umkehr der Liste realisiert).
- Kontextauswertung: Parser unterstützt `a && b`, `a || b`, `!a`, sowie Mischformen wie `a && b || c` (implizit `a && (b || c)`). Keine Klammern.

Nicht implementiert (Stand jetzt): Sequenzen / mehrstufige Eingaben, Timeout, Layer-Vererbung, eigene `[[layers]]`, Parenthesen im Kontextparser.

### 10.2 Dateiformat (`keymap.toml`)
Struktur direkt an der Wurzel (KEIN zusätzlicher `[keymap]` Block):
```toml
[[bindings]]
context = "global"
      # Jede Action ist eine Array-of-Tables unter ihrem Namen
      [[bindings.actions.open_palette]]
      key = "ctrl+p"

      [[bindings.actions.save_story]]
      key = "ctrl+s"

[[bindings]]
context = "editor && !modal"
      [[bindings.actions.comment_block]]
      key = "ctrl+k"
      # zweites Binding für gleiche Action (höhere Priorität weil später):
      [[bindings.actions.comment_block]]
      key = "ctrl+alt+k"

[[bindings]]
context = "editor"
      [[bindings.actions.run_macro]]
      key = "f5"
      action_data = { macro = "build_and_run" }

[[bindings]]
context = "global"
      [[bindings.actions.accept]]
      key = "a"
      device = "Gamepad"
```

Hinweise:
- Gerätewerte müssen exakt den Enum-Namen treffen (`Keyboard`, `Mouse`, `Gamepad`).
- Modifier werden als Teil des `key` Strings geschrieben: `ctrl+shift+enter`.
- Kein spezieller Syntax für Sequenzen; mehrere Tasten hintereinander werden NICHT erkannt.

### 10.3 Priorität & Auflösung
- Intern werden die `bindings` Gruppen in Reihenfolge gelesen, dann rückwärts iteriert (Reverse) -> „last wins“.
- Bei mehreren KeyDefinitions für dieselbe Action + gleichen KeyChord gewinnt die zuletzt definierte.
- Konflikte zwischen verschiedenen Actions mit demselben KeyChord: die spätere BindingGroup gewinnt (höhere Priorität).

### 10.4 Kontext-Prädikate
Unterstützt: einfache Tokens + Operatoren:
- Negation: `!popup`
- UND: `a && b`
- ODER: `a || b`
- Kombi: `a && b || c` wird als `a && (b || c)` interpretiert (Parser-Heuristik). Klammern werden nicht erkannt, also `a && (b || c)` explizit ist NICHT möglich.

### 10.5 ActionRegistry
Das Trait `ActionRegistry` liefert:
- `resolve_action(name, action_data)` -> konkrete Action.
- Optionale Validierung (du kannst z.B. beim Laden prüfen, ob alle Namen existieren).

### 10.6 Gerätedifferenzierung
- Präfixe wie `mouse:` oder `gp:` tauchen nur im Legacy/Export Teil auf (Parsing von alten Chord-Strings). Im neuen TOML-Format nutzt du stattdessen das Feld `device`.

### 10.7 Export / Debug
- `debug_keymap_state_summary()` liefert Kurzinfo (Anzahl + Sample).
- `resolve_action_for_key("ctrl+s", &["editor"], registry)` evaluiert Kontext + Priorität.

### 10.8 Tests (empfohlen, realistisch)
- Parse Roundtrip einer Minimal-Datei.
- Priorität: spätes überschreibt frühes.
- Kontext: `a && !b` greift nur wenn `a` aktiv und `b` nicht.
- Device Unterscheidung: `Gamepad` Binding kollidiert nicht mit `Keyboard`.

## 11. Beispiel Minimaler Benutzer Override

Datei `~/.config/forge_of_stories/keymap.toml`:
```toml
[[bindings]]
context = "global"
      [[bindings.actions.open_palette]]
      key = "ctrl+p"  # oder auf macOS: "meta+p" falls du das so normalisierst
```
Nur die abweichende Belegung definieren.

## 12. Tipps für Konsistenz

- Ein Wort pro Aktion (oder snake_case); Verben bevorzugen (`open_`, `save_`, `toggle_`).
- Keine doppelten semantischen Synonyme (`open_palette` vs `show_palette`).
- Dokumentiere alle Aktionen zentral (Tabelle generierbar aus Registrierungscode).

## 13. Troubleshooting

| Symptom | Ursache | Lösung |
|---------|---------|-------|
| Binding reagiert nicht | `when` Bedingung false | Debug‑Overlay aktivieren: aktive Flags anzeigen |
| Falsche Aktion bei Sequenz | Timeout zu kurz | `sequence_timeout_ms` erhöhen |
| Änderung verschwindet nach Update | Nutzerdatei nicht geschrieben | Prüfe Schreibrechte & Log |
| Crash beim Laden | Ungültige Zeichen | TOML linter laufen lassen |

## 14. Evtl. Geplante / noch nicht implementierte Features (frühere Doku-Teile waren visionär)

- Sequenzen / Mehrstufige Shortcuts mit Timeout.
- Trie/Prefix-Struktur für schnellere Sequenzsuche.
- Layer / Vererbung (`inherits`) und Modus-spezifische Aktivierung.
- Parenthesen & vollständige Operator-Präzedenz im Kontextparser.
- Hot-Reload (Builder Flag `watch_files` ist vorbereitet, aber noch TODO).
- Environment Variable Mapping (`EnvPrefix`) – Funktion stubbt aktuell leer.
- Automatische Konfliktanalyse beim Rebinding.

---

## 15. Kurzzusammenfassung

Settings: Mehrschichtiges Merge (eingebettete Texte + Dateien + optional Env) mit last-wins; `update()` schreibt nur Deltas gegenüber Defaults. Keymaps: Aktuell einfache Gruppen mit Kontext-Ausdrücken ohne Sequenzen; Priorität = spätere Definition. Dokument korrigiert frühere visionäre Teile.

Fertig.
