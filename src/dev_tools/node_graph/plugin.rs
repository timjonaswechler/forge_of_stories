use super::context::NodesContext;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin}; // Import EguiPlugin and EguiContexts
use egui::WidgetText;
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer}; // Import WidgetText for TabViewer implementation

// Resource to hold the DockState
#[derive(Resource, Deref, DerefMut)]
struct MyDockState(DockState<MyWindowType>);

// Enum defining the types of windows/tabs we can have
#[derive(Debug, Clone, PartialEq, Eq)] // Added Clone, PartialEq, Eq for DockState requirements
enum MyWindowType {
    GraphEditor,
    DetailsView,
}

// The TabViewer implementation
struct MyTabViewer<'a, 'b> {
    nodes_context: &'a mut NodesContext,
    world_cell: &'b mut World, // Use World directly for potential entity inspection
}

// --- Implement TabViewer for MyTabViewer ---
impl<'a, 'b> TabViewer for MyTabViewer<'a, 'b> {
    // --- ADDED: Define the Tab type ---
    type Tab = MyWindowType;

    // --- ADDED: Implement the title function ---
    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        // Return a title based on the tab type
        match tab {
            MyWindowType::GraphEditor => "Graph Editor".into(),
            MyWindowType::DetailsView => "Details".into(),
        }
    }

    // --- Keep the ui function as it was ---
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            MyWindowType::GraphEditor => {
                // Pass empty vectors for now, as we don't have real node/link data yet
                self.nodes_context.show(Vec::new(), Vec::new(), ui);
                // Optional: Add a label if show doesn't fill space or for testing
                // ui.label("Graph Editor Contents");
            }
            MyWindowType::DetailsView => {
                ui.label("Details View Placeholder");
                // TODO: Add logic to show details based on selection in NodesContext
                // You might need to access `self.world_cell` here to query components
                // let selected_nodes = self.nodes_context.get_selected_nodes();
                // if let Some(node_id) = selected_nodes.first() {
                //    // Query world_cell for data related to node_id (needs Entity mapping)
                // }
            }
        }
    }
}
// --- End of TabViewer Implementation ---

// The main plugin for the Node Graph UI
pub struct NodeGraphPlugin;

impl Plugin for NodeGraphPlugin {
    fn build(&self, app: &mut App) {
        // Ensure EguiPlugin is added (might be added elsewhere, but good practice)
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
            info!("EguiPlugin added by NodeGraphPlugin");
        }

        // Initialize the DockState with a default layout
        let mut initial_dock_state = DockState::new(vec![MyWindowType::GraphEditor]);
        let surface = initial_dock_state.main_surface_mut();
        // Split the main area, putting DetailsView on the right (taking 25% of space)
        surface.split_right(
            NodeIndex::root(),
            0.75, // Fraction for the left (GraphEditor) pane
            vec![MyWindowType::DetailsView],
        );

        app.insert_resource(MyDockState(initial_dock_state)); // Add the DockState resource
        app.init_resource::<NodesContext>(); // Add the NodesContext resource

        // Add the UI system to the update schedule
        app.add_systems(Update, graph_ui_system);

        info!("NodeGraphPlugin loaded and configured.");
    }
}

// Modified graph_ui_system to use proper Bevy system parameters
fn graph_ui_system(
    mut egui_contexts: EguiContexts,
    mut nodes_context: ResMut<NodesContext>,
    mut dock_state: ResMut<MyDockState>,
) {
    // Get the mutable egui context, usually for the primary window
    let ctx = egui_contexts.ctx_mut();

    // Clone the style to use it for the dock area
    let egui_style = ctx.style().clone();

    // Create the TabViewer instance with access to resources
    let mut tab_viewer = MyTabViewer {
        nodes_context: &mut *nodes_context,
        world_cell: &mut World::new(), // Temporary empty world as a placeholder
    };

    // Show the DockArea, passing the egui context and the tab viewer
    DockArea::new(&mut dock_state.0)
        .style(Style::from_egui(&egui_style))
        .show(ctx, &mut tab_viewer);
}
