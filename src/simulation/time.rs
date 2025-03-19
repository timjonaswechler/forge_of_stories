use super::{SimulationPhase, SimulationState};
use bevy::prelude::*;

// UI commands for controlling the simulation
pub struct TimeControlCommands;

impl Plugin for TimeControlCommands {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_time_control_input);
    }
}

fn handle_time_control_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut sim_state: ResMut<SimulationState>,
) {
    // Pause/resume simulation
    if keyboard_input.just_pressed(KeyCode::Space) {
        sim_state.running = !sim_state.running;

        let status = if sim_state.running {
            "resumed"
        } else {
            "paused"
        };
        info!("Simulation {}", status);
    }

    // Increase/decrease simulation speed
    if keyboard_input.just_pressed(KeyCode::Up) {
        sim_state.speed *= 2.0;
        info!("Simulation speed: {}", sim_state.speed);
    }

    if keyboard_input.just_pressed(KeyCode::Down) {
        sim_state.speed *= 0.5;
        info!("Simulation speed: {}", sim_state.speed);
    }

    // Advance simulation phase
    if keyboard_input.just_pressed(KeyCode::Right) {
        advance_simulation_phase(&mut sim_state);
    }

    // Reset to beginning
    if keyboard_input.just_pressed(KeyCode::R) {
        *sim_state = SimulationState::default();
        info!("Simulation reset");
    }
}

fn advance_simulation_phase(sim_state: &mut SimulationState) {
    let next_phase = match sim_state.phase {
        SimulationPhase::Setup => SimulationPhase::TectonicFormation,
        SimulationPhase::TectonicFormation => SimulationPhase::ClimateAndResources,
        SimulationPhase::ClimateAndResources => SimulationPhase::CivilizationFormation,
        SimulationPhase::CivilizationFormation => SimulationPhase::DetailedHistory,
        SimulationPhase::DetailedHistory => SimulationPhase::Complete,
        SimulationPhase::Complete => SimulationPhase::Complete, // No further phases
    };

    sim_state.phase = next_phase;
    info!("Advanced to simulation phase: {:?}", sim_state.phase);
}
