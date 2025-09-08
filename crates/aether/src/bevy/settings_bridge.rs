use aether_config::bevy::GeneralRes;
use bevy::prelude::*;

fn apply_tickrate_initial(general: Res<GeneralRes>, mut time_fixed: ResMut<Time<Fixed>>) {
    let hz = general.tick_rate;
    time_fixed.set_timestep_hz(hz);
    info!("Initial FixedUpdate tick rate set from settings: {} Hz", hz);
}

fn apply_tickrate_from_general(general: Res<GeneralRes>, mut time_fixed: ResMut<Time<Fixed>>) {
    if general.is_changed() {
        let hz = general.tick_rate;
        time_fixed.set_timestep_hz(hz);
        info!("Applied FixedUpdate tick rate from settings: {} Hz", hz);
    }
}

// reload handled by settings bevy adapter

pub struct AetherSettingsPlugin;
impl Plugin for AetherSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, apply_tickrate_initial)
            .add_systems(Update, apply_tickrate_from_general);
    }
}
