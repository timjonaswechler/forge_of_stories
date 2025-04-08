// src/ui_components/node_graph/plugin.rs
use bevy::log; // Für Logging konsistent verwenden
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, WidgetText},
    EguiContexts, EguiPlugin,
};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer}; // Docking spezifische Typen

// Lokale Module für die UI Komponente
use super::context::NodesContext;
use super::resources::GraphUIData;

// MyTabViewer struct muss hier definiert sein oder importiert werden, wenn es in ein eigenes Modul kommt.
// Momentan lassen wir es hier der Einfachheit halber.
// --- MyTabViewer Definition ---
struct MyTabViewer<'a> {
    nodes_context: &'a mut NodesContext,
    graph_data: &'a GraphUIData,
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
            MyWindowType::GraphEditor => {
                // Ruft die 'show'-Methode des NodesContext auf, um den Graphen zu zeichnen
                self.nodes_context.show(
                    self.graph_data.nodes.clone(), // Nimmt Daten aus GraphUIData
                    self.graph_data.links.clone(), // Clone ist noch hier (TODO 10)
                    ui,
                );
            }
            MyWindowType::DetailsView => {
                // Platzhalter UI für die Detailansicht (TODO 4)
                ui.label("Details View Placeholder");
                if let Some(sel_id) = self.nodes_context.get_selected_nodes().first() {
                    // Finde den VisNode anhand der ID... (Logik wie zuvor)
                    if let Some(vis_node) = self.graph_data.nodes.iter().find(|n| n.id == *sel_id) {
                        if let Some(entity) = vis_node.entity {
                            ui.label(format!("Selected Node ID: {}", sel_id)); // Zeige Node ID
                            ui.label(format!("Selected Entity: {:?}", entity)); // Zeige Entity ID
                                                                                // Hier würden weitere Infos aus vis_node.details etc. kommen
                        } else {
                            ui.label(format!("Selected Node ID: {} (No Entity!)", sel_id));
                        }
                    } else {
                        ui.label(format!(
                            "Selected Node ID: {} (Not in current data)",
                            sel_id
                        ));
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
        // Stelle sicher, dass Egui vorhanden ist
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }

        // Initialisiere den Docking-Zustand mit den Fenstertypen
        let mut initial_dock_state = DockState::new(vec![MyWindowType::GraphEditor]);
        let surface = initial_dock_state.main_surface_mut();
        // Optional: Split definieren (könnte auch ins DevUIPlugin)
        let [_graph_node, _details_node] =
            surface.split_right(NodeIndex::root(), 0.75, vec![MyWindowType::DetailsView]);
        app.insert_resource(MyDockState(initial_dock_state));

        // --- Wichtige Ressourcen für den Node Graphen ---
        app.init_resource::<NodesContext>(); // Der Kern-Kontext der UI
        app.init_resource::<GraphUIData>(); // Die Daten-Schnittstelle (wird von außen gefüllt)

        // --- Nur das UI-System wird hier registriert ---
        // Dieses System nimmt die Daten aus GraphUIData und übergibt sie an NodesContext.show()
        app.add_systems(Update, graph_ui_system);

        log::info!("Generic NodeGraphPlugin loaded."); // Log angepasst
    }
}

// --- graph_ui_system Funktion ---
/// Dieses System rendert die egui DockArea und den TabViewer.
/// Es liest den `GraphUIData` (gefüllt von einem externen System) und übergibt
/// ihn zusammen mit dem `NodesContext` an die `MyTabViewer` Instanz.
pub fn graph_ui_system(
    mut egui_contexts: EguiContexts,
    mut dock_state: ResMut<MyDockState>,     // Zustand des Docks
    mut nodes_context: ResMut<NodesContext>, // Kern-UI-Logik (mutable für show)
    graph_data: Res<GraphUIData>,            // UI-Daten aus externer Quelle
) {
    let ctx = egui_contexts.ctx_mut();
    let egui_style = ctx.style().clone();

    // Erstelle den TabViewer mit den nötigen Referenzen
    let mut tab_viewer = MyTabViewer {
        nodes_context: &mut nodes_context,
        graph_data: &graph_data,
    };

    // Definiere den Stil für die DockArea selbst
    let dock_style = Style::from_egui(&egui_style);

    // Zeige die DockArea an
    DockArea::new(&mut dock_state.0) // Greife auf inneres DockState zu
        .style(dock_style)
        .show(ctx, &mut tab_viewer); // Übergib den TabViewer zum Rendern der Inhalte
}
