use aether_config::General;
use bevy::prelude::*;
use settings::SettingsArc;

fn apply_tickrate_initial(general: Res<SettingsArc<General>>, mut time_fixed: ResMut<Time<Fixed>>) {
    let hz = general.tick_rate;
    time_fixed.set_timestep_hz(hz);
    info!("Initial FixedUpdate tick rate set from settings: {} Hz", hz);
}

// reload handled by settings bevy adapter

pub struct AetherSettingsPlugin;
impl Plugin for AetherSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, apply_tickrate_initial);
    }
}
