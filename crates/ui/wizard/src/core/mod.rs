//! Core subsystem of the Wizard application.
//
//! Phase 3 (Monolith-Aufbruch):
//!   - Task 3.1 verschiebt die frühere `wizard.rs` nach `core/app.rs` (dieses
//!     Modul stellt sie unter `core::app` bereit).
//!   - Task 3.2 wird einen separaten Eventloop in `core/loop.rs` (`AppLoop`) extrahieren.
//!   - Task 3.3 wird das reine Rendering in ein eigenes Modul (z. B. `ui::render`
//!     oder `core::render`) auslagern.
//
//! Ziele dieser Schicht:
//!   * Zentrale Zustandsstruktur (`WizardApp`)
//!   * Kapselung der App-Lifecycle-Hooks
//!   * Vorbereitung für Reducer / Intent / Effect Pipeline
//
//! Geplante Module (Folge-Phasen):
//!   - `loop`     : Enthält den entkoppelten Event-/Action-Verarbeitungsloop
//!   - `render`   : Reines Rendering ohne Steuerlogik
//!   - `state`    : Aggregation komplexerer App- oder Domainzustände (Phase 4)
//!   - `effects`  : (Phase 10) Deklaratives Effect / TaskKind Modell
//!   - `executor` : (Phase 10) Hintergrund-Task Scheduler (führt TaskKind asynchron aus)
//!
//! Migrationshinweise:
//!   Bestehende Importe von `crate::wizard::WizardApp` bitte auf
//!   `crate::core::app::WizardApp` anpassen (bereits für `main.rs` erledigt).
//
//! Diese Datei ist bewusst schlank; sie dient als klarer Einstiegspunkt für
//! zukünftige Kernmodule.
pub mod app;
pub mod effects; // Phase 10: Effect & TaskKind definitions
pub mod executor; // Phase 10: Async task executor
pub mod intent_model; // Phase 11: Intent / UiCommand / InternalEvent scaffold
pub mod r#loop;
pub mod reducer;
pub mod state; // Phase 4.1: High-level AppState & RootState eingeführt // Phase 4.2: Reducer prototype (intents -> state mutations)

// Re-Exports (optional aktuell leer):
// pub use app::WizardApp;
