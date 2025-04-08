// src/dev_tools/node_graph/plugin.rs

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, WidgetText},
    EguiContexts, EguiPlugin,
}; // Style als EguiStyle importieren
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer}; // Style von egui_dock behalten

use super::context::NodesContext;
use super::resources::GraphUIData;
use super::systems::graph_data_provider_system;

// === NEU: Struct Definition für MyTabViewer ===
/// Hilfsstruktur, die die notwendigen Daten für das Rendern der Tabs hält.
/// Die Lebenszeit `'a` stellt sicher, dass die Referenzen auf die Bevy-Ressourcen
/// gültig bleiben, solange der TabViewer existiert (innerhalb des `graph_ui_system`).
struct MyTabViewer<'a> {
    nodes_context: &'a mut NodesContext,
    graph_data: &'a GraphUIData,
    // Hier könnten weitere Bevy-Ressourcen oder Queries übergeben werden,
    // die für das Rendern der Tabs benötigt werden.
    // world: &'a World, // Vorsicht mit World-Zugriff hier
}

#[derive(Resource, Deref, DerefMut)]
struct MyDockState(DockState<MyWindowType>);

#[derive(Debug, Clone, PartialEq, Eq)]
enum MyWindowType {
    GraphEditor,
    DetailsView,
}

// === Implementierung für das definierte Struct ===
impl<'a> TabViewer for MyTabViewer<'a> {
    type Tab = MyWindowType;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        /* ... wie zuvor ... */
        match tab {
            MyWindowType::GraphEditor => "Graph Editor".into(),
            MyWindowType::DetailsView => "Details".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        /* ... wie zuvor ... */
        match tab {
            MyWindowType::GraphEditor => {
                // Das Feld 'nodes_context' ist jetzt ein &mut NodesContext
                self.nodes_context.show(
                    self.graph_data.nodes.clone(), // Clone bleibt vorerst
                    self.graph_data.links.clone(), // Clone bleibt vorerst
                    ui,
                );
            }
            MyWindowType::DetailsView => {
                /* ... wie zuvor ... */
                ui.label("Details View Placeholder");
                if let Some(sel_id) = self.nodes_context.get_selected_nodes().first() {
                    if let Some(vis_node) = self.graph_data.nodes.iter().find(|n| n.id == *sel_id) {
                        if let Some(entity) = vis_node.entity {
                            ui.label(format!("Selected Entity: {:?}", entity));
                        } else {
                            ui.label("Selected VisNode missing Entity.");
                        }
                    } else {
                        ui.label("Selected Node ID not in data.");
                    }
                } else {
                    ui.label("No node selected.");
                }
            }
        }
    }

    // Optional: Methoden wie `force_closeable`, `add_popup` etc.
}

pub struct NodeGraphPlugin;

impl Plugin for NodeGraphPlugin {
    fn build(&self, app: &mut App) {
        /* ... wie zuvor ... */
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }

        let mut initial_dock_state = DockState::new(vec![MyWindowType::GraphEditor]);
        let surface = initial_dock_state.main_surface_mut();
        let [_left, _right] =
            surface.split_right(NodeIndex::root(), 0.75, vec![MyWindowType::DetailsView]);

        app.insert_resource(MyDockState(initial_dock_state));
        app.init_resource::<NodesContext>(); // Initialisiere NodesContext Ressource
        app.init_resource::<GraphUIData>();
        app.add_systems(
            Update,
            (
                graph_data_provider_system.before(graph_ui_system),
                graph_ui_system,
            ),
        );

        info!("NodeGraphPlugin loaded and configured."); // Using Bevy's info! macro
    }
}

fn graph_ui_system(
    mut egui_contexts: EguiContexts,
    mut dock_state: ResMut<MyDockState>,
    mut nodes_context: ResMut<NodesContext>, // Wird an MyTabViewer übergeben
    graph_data: Res<GraphUIData>,            // Wird an MyTabViewer übergeben
                                             // Optional: world: &World für Details View
) {
    let ctx = egui_contexts.ctx_mut();
    let egui_style = ctx.style().clone(); // Egui Style

    // === KORRIGIERT: Instanziiere das definierte Struct ===
    // Erstelle den TabViewer mit Referenzen auf die benötigten Ressourcen
    let mut tab_viewer = MyTabViewer {
        nodes_context: &mut nodes_context, // Mutable Referenz
        graph_data: &graph_data,           // Immutable Referenz
                                           // world: world, // Falls benötigt
    };

    // Nutze den DockState Stil oder einen eigenen. Hier wird `egui_dock::Style` verwendet
    let dock_style = Style::from_egui(&egui_style); // Style für die DockArea selbst

    DockArea::new(&mut dock_state.0) // DerefMut für inneres DockState
        .style(dock_style) // Übergib den DockArea-Style
        .show(ctx, &mut tab_viewer); // Übergib den TabViewer

    // Die tab_viewer_closure wird nicht mehr benötigt
    /*
    let mut tab_viewer_closure = |ui: &mut egui::Ui, tab: &mut MyWindowType| {
       match tab {
           MyWindowType::GraphEditor => { /* ... alter Code ... */ }
           MyWindowType::DetailsView => { /* ... alter Code ... */ }
       }
    };
    */
}
