// src/dev_ui/plugin.rs
use bevy::prelude::*;

// src/dev_ui/plugin.rs
use bevy::prelude::*;

// Importiere den generischen Node Graph Plugin
use crate::ui_components::node_graph::NodeGraphPlugin;
// Importiere die spezifischen Integrationssysteme
use crate::dev_ui::simulation_graph::{
    handle_graph_changes_system,
    provide_simulation_graph_data,
    update_selected_node_details, // Dieser Import sollte jetzt klappen
};

use crate::ui_components::node_graph::plugin::graph_ui_system;

/// Plugin, das die Entwickler-UI-Tools integriert.
/// Es fügt die generischen UI-Komponenten hinzu (z.B. NodeGraphPlugin)
/// und registriert die spezifischen Systeme, die diese Komponenten
/// mit den Anwendungsdaten verbinden.
pub struct DevUIPlugin;

impl Plugin for DevUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NodeGraphPlugin);

        // --- Ressource für Detailansicht entfernen ---
        // app.init_resource::<SelectedNodeDetails>(); // ENTFERNEN

        app.add_systems(
            Update,
            (
                // Richtige Reihenfolge (versimplifiziert):
                // (graph_ui_system läuft implizit, als erstes in dieser Kette betrachtet)

                // 1. Reagiere auf UI-Events und ändere die Simulation
                handle_graph_changes_system.after(graph_ui_system),
                // 2. Aktualisiere Detaildaten basierend auf neuer Auswahl/Zustand
                update_selected_node_details // Immer noch mit altem Namen? Wenn ja, hier anpassen
                    .after(handle_graph_changes_system),
                // 3. Sammle Daten aus der (jetzt geänderten) Simulation für den nächsten Frame
                provide_simulation_graph_data.after(update_selected_node_details), // Nach den Detail-Updates, falls die auch schreiben
                                                                                   // ALT: .before(graph_ui_system), // DIESE ZEILE ENTFERNEN/AUSKOMMENTIEREN
            ), // Entferne .chain() wenn du es nicht explizit für andere Sets brauchst
               // .chain(),
        );
    }
}
