// src/ui_components/node_graph/plugin.rs
use bevy::log;
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, WidgetText},
    EguiContexts, EguiPlugin,
};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer}; // Docking spezifische Typen

// Lokale Module für die UI Komponente
use super::context::{LinkValidationCallback, NodesContext};
use super::resources::GraphUIData;

use super::ui_pin::{PinSpec, PinType}; // Für die Pins

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
                // *** DEFINE Validation Logic (Anwendungsspezifisch!) ***
                // Diese Closure implementiert die spezifischen Regeln
                let link_validator: Box<LinkValidationCallback> = Box::new(
                    |start_pin_spec: &PinSpec,
                     end_pin_spec: &PinSpec,
                     _context: &NodesContext|
                     -> bool {
                        // --- VALIDATION LOGIC basierend auf relation_type und kind ---

                        // 1. Grundregel: Die Beziehungstypen müssen übereinstimmen.
                        if start_pin_spec.relation_type != end_pin_spec.relation_type {
                            // Optional: Log, warum die Validierung fehlschlägt
                            // bevy::log::trace!("Validation Fail: Mismatched relation types '{}' vs '{}'", start_pin_spec.relation_type, end_pin_spec.relation_type);
                            return false;
                        }

                        // 2. Richtungen (PinType/kind) müssen kompatibel sein.
                        let valid_direction = match (start_pin_spec.kind, end_pin_spec.kind) {
                            // Standard: Output verbindet sich mit Input
                            (PinType::Output, PinType::Input) => true,
                            (PinType::Input, PinType::Output) => true,

                            // Sonderfall: Erlaube Output <-> Output *nur* für "Friendship"
                            // (Da wir InOut visuell als Output behandeln)
                            (PinType::Output, PinType::Output) => {
                                if start_pin_spec.relation_type == "Friendship" {
                                    true // Erlaube Freundschaft zwischen zwei "Output" (InOut) Pins
                                } else {
                                    false // Verbiete Output <-> Output für andere Typen (z.B. Family)
                                }
                            }

                            // Alle anderen Kombinationen (Input<->Input, None->...) sind ungültig
                            _ => false,
                        };

                        if !valid_direction {
                            // Optional: Log
                            // bevy::log::trace!("Validation Fail: Incompatible PinTypes {:?} -> {:?}", start_pin_spec.kind, end_pin_spec.kind);
                            return false;
                        }

                        // Wenn alle Prüfungen für diesen Validator bestanden haben
                        true // Die Verbindung ist gemäß diesen Regeln erlaubt
                    },
                );

                // Rufe 'show' mit dem aktualisierten Validator auf
                self.nodes_context.show(
                    self.graph_data.nodes.clone(),
                    self.graph_data.links.clone(),
                    ui,
                    &*link_validator, // Übergebe eine Referenz auf die geboxte Closure
                );
            }

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
