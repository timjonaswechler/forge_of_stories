// Neue Datei: src/plugins/event_plugin.rs
use crate::simulation::events::{
    ChildBornEvent, EntityInitializedEvent, ReproduceRequestEvent, TemporaryAttributeModifierEvent,
};
use bevy::prelude::*;

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EntityInitializedEvent>()
            .add_event::<TemporaryAttributeModifierEvent>()
            .add_event::<ReproduceRequestEvent>()
            .add_event::<ChildBornEvent>();
        // Füge hier zukünftige Events hinzu
    }
}
