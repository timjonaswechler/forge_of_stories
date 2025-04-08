// src/dev_ui/mod.rs
pub mod plugin;
pub mod simulation_graph; // DetailDisplayData ist in resources.rs

pub use plugin::DevUIPlugin;
pub use simulation_graph::handle_graph_changes_system;
pub use simulation_graph::provide_simulation_graph_data;
// === GEÄNDERTE/NEUE Exporte ===
pub use simulation_graph::update_selected_node_details;
