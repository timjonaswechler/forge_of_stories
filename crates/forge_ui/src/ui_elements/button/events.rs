use bevy::prelude::*;

/// Event sent when a non-disabled button is clicked (Pressed state).
#[derive(Event, Debug, Clone)]
pub struct ButtonClickedEvent {
    /// The entity ID of the button that was clicked.
    pub button_entity: Entity,
}
