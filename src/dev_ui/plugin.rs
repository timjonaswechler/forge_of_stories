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
                provide_simulation_graph_data.before(graph_ui_system),
                handle_graph_changes_system.after(graph_ui_system),
                // update_selected_node_details // ALT
                update_selected_node_details // NEU (oder alter Name, aber mit neuer Logik)
                    .after(handle_graph_changes_system),
            ),
        );
    }
}
