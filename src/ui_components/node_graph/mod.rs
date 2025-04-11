// src/ui_components/node_graph/mod.rs
pub mod context;
pub mod coords;
pub mod drawing;
pub mod hover;
pub mod identity;
pub mod interaction;
pub mod plugin;
pub mod resources;
pub mod settings;
pub mod state;
pub mod storage;
pub mod systems;
pub mod ui_data;
pub mod ui_link;
pub mod ui_node;
pub mod ui_pin;
pub mod ui_style;

// Re-exports für einfachen Zugriff

pub use plugin::NodeGraphPlugin;
pub use resources::GraphUIData;

pub use ui_data::*;
pub use ui_link::LinkSpec;
pub use ui_node::NodeSpec;
pub use ui_pin::PinSpec;
