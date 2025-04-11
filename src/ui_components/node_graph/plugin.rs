// src/ui_components/node_graph/plugin.rs
use bevy::log;
use bevy::prelude::*;

use super::resources::GraphUIData;
use super::settings::NodesSettings;
use super::state::GraphUiStateManager;
use super::storage::GraphStorage;
use super::systems::graph_ui_system;
use bevy_egui::{
    egui::{self, WidgetText},
    EguiPlugin,
};
use egui_dock::{DockState, NodeIndex, TabViewer};

pub struct MyTabViewer<'a> {
    pub graph_data: &'a GraphUIData,
}

// --- MyDockState Definition ---
#[derive(Resource, Deref, DerefMut)]
pub struct MyDockState(DockState<MyWindowType>); // Status des Docking-Layouts

// --- MyWindowType Definition ---
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MyWindowType {
    GraphEditor,
    DetailsView,
} // Mögliche Fenster-Typen im Dock

// --- TabViewer Implementierung ---
impl<'a> TabViewer for MyTabViewer<'a> {
    type Tab = MyWindowType;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        match tab {
            MyWindowType::GraphEditor => "Node Graph".into(), // Titel geändert für Klarheit
            MyWindowType::DetailsView => "Details".into(),
        }
    }

    // Diese Funktion rendert den *Inhalt* des jeweiligen Tabs
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            MyWindowType::GraphEditor => {}

            MyWindowType::DetailsView => {
                ui.heading("Selected Node Details");
                ui.separator();

                // Greife auf die vorbereiteten Daten in GraphUIData zu
                if let Some(details) = &self.graph_data.selected_node_details_display {
                    ui.label(&details.title); // Zeige den vorbereiteten Titel
                    ui.separator();

                    if !details.properties.is_empty() {
                        ui.heading("Components:");
                        egui::Grid::new("details_grid") // Optional: Grid-Layout
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                for (name, value) in &details.properties {
                                    ui.label(name);
                                    ui.label(value);
                                    ui.end_row();
                                }
                            });
                    } else {
                        ui.label("No component details found/queried.");
                    }

                    ui.separator();
                    // Log-Button (oder andere Aktionen)
                    if ui.button("Log Raw Entity Details").clicked() {
                        // Entity ID extrahieren (vielleicht nicht die beste Methode)
                        if let Some(entity_str) = details.title.split_whitespace().last() {
                            bevy::log::info!(
                                "Log button clicked for prepared details: {}",
                                entity_str
                            );
                            // Hier könnte man komplexere Aktionen auslösen, braucht aber wieder Zugriff auf ECS
                        }
                    }
                } else {
                    ui.label("No node selected.");
                }
            }
        }
    }
}

// --- NodeGraphPlugin Definition ---
/// Plugin für die **generische** Node Graph UI Komponente.
/// Registriert Egui, den NodesContext, die GraphUIData Ressource (als Schnittstelle)
/// und das System zum Anzeigen der Docking UI.
pub struct NodeGraphPlugin;

impl Plugin for NodeGraphPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        let mut initial_dock_state = DockState::new(vec![MyWindowType::GraphEditor]);
        let surface = initial_dock_state.main_surface_mut();

        let [_graph_node, _details_node] =
            surface.split_right(NodeIndex::root(), 0.75, vec![MyWindowType::DetailsView]);
        app.insert_resource(MyDockState(initial_dock_state));
        app.init_resource::<GraphUiStateManager>();
        app.init_resource::<GraphStorage>();

        app.init_resource::<GraphUIData>();
        app.init_resource::<NodesSettings>();
        // Optional: Wenn du andere Defaults willst, kannst du sie hier setzen:
        // app.insert_resource(NodesSettings { style: Style::classic(), ..Default::default() });

        app.add_systems(Update, graph_ui_system);

        log::info!("Generic NodeGraphPlugin loaded."); // Log angepasst
    }
}
