use bevy::ecs::resource::Resource;
use bevy_settings::Settings;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Resource)]
pub struct Network {
    port: u16,
    label: String,
}
impl Default for Network {
    fn default() -> Self {
        Self {
            port: 5000,
            label: "main".into(),
        }
    }
}
impl Settings for Network {
    const SECTION: &'static str = "network";
}
