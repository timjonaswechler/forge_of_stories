// src/dev_tools/node_graph/mod.rs
pub mod context;
pub mod plugin;
pub mod resources; // <<< NEU
pub mod systems; // <<< NEU
pub mod ui_data;
pub mod ui_link;
pub mod ui_node;
pub mod ui_pin;
pub mod ui_style;

// Re-exports für einfachen Zugriff
pub use context::NodesContext;
pub use plugin::NodeGraphPlugin;
pub use resources::GraphUIData;

pub use ui_data::*;
pub use ui_link::LinkSpec;
pub use ui_node::NodeSpec;
pub use ui_pin::PinSpec;
