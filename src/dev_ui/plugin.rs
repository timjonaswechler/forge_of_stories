// src/dev_ui/plugin.rs
use bevy::prelude::*;

// Importiere den generischen Node Graph Plugin
use crate::ui_components::node_graph::NodeGraphPlugin;
// Importiere die spezifischen Integrationssysteme
use crate::dev_ui::simulation_graph::{handle_graph_changes_system, provide_simulation_graph_data};
// Importiere das UI System vom generischen Plugin für die Ordnung (.before/.after)
use crate::ui_components::node_graph::plugin::graph_ui_system;
// Optional: Importiere AppState für run_if Bedingungen
// use crate::app_setup::AppState;

/// Plugin, das die Entwickler-UI-Tools integriert.
/// Es fügt die generischen UI-Komponenten hinzu (z.B. NodeGraphPlugin)
/// und registriert die spezifischen Systeme, die diese Komponenten
/// mit den Anwendungsdaten verbinden.
pub struct DevUIPlugin;

impl Plugin for DevUIPlugin {
    fn build(&self, app: &mut App) {
        // Füge zuerst die generischen UI-Komponenten-Plugins hinzu
        // NodeGraphPlugin fügt Egui hinzu (falls nicht vorhanden) und registriert
        // graph_ui_system, NodesContext, GraphUIData
        app.add_plugins(NodeGraphPlugin);

        // --- Registriere die spezifischen Integrationssysteme ---
        app.add_systems(
            Update,
            (
                // 1. Daten sammeln und für die UI aufbereiten (Provider)
                provide_simulation_graph_data.before(graph_ui_system),
                // (graph_ui_system läuft implizit durch NodeGraphPlugin)

                // 2. Auf UI-Änderungen reagieren und Simulation anpassen (Handler)
                handle_graph_changes_system.after(graph_ui_system),
            ),
        );

        // Hier könnten später weitere Dev-UI-Integrationen registriert werden
        // z.B. für einen Plotter, Inspektor etc.
    }
}
