// Declare the modules within ui_elements::button
pub mod builder;
pub mod components;
pub mod enums;
pub mod events;
pub mod style;
pub mod systems;

// Re-export the public API
pub use builder::ButtonBuilder;
pub use components::ButtonMarker; // Marker kann nützlich sein
pub use components::OnClick; // Wenn externe Callback-Komponente erwünscht
pub use enums::{ButtonSize, ButtonVariant};
pub use events::ButtonClickedEvent;

pub use systems::{handle_button_clicks_event, handle_button_clicks_fn, update_button_visuals};
