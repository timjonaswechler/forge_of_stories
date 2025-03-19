use bevy::prelude::*;

pub mod time;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationState>()
            .init_resource::<SimulationSettings>()
            .add_systems(Update, update_simulation);
    }
}

#[derive(Resource, Default)]
pub struct SimulationState {
    /// Current simulation phase
    pub phase: SimulationPhase,
    /// Current simulation year
    pub year: i64,
    /// Is simulation currently running or paused
    pub running: bool,
    /// Speed multiplier for simulation
    pub speed: f32,
}

#[derive(Resource, Default)]
pub struct SimulationSettings {
    /// Years per simulation step during tectonic phase
    pub tectonic_years_per_step: i64,
    /// Years per simulation step during civilization phase
    pub civilization_years_per_step: i64,
    /// Years per simulation step during detailed history phase
    pub history_years_per_step: i64,
}

impl SimulationSettings {
    pub fn new() -> Self {
        Self {
            tectonic_years_per_step: 1_000_000, // 1 million years per step for tectonic simulation
            civilization_years_per_step: 100,   // 100 years per step for civilization formation
            history_years_per_step: 1,          // 1 year per step for detailed history
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum SimulationPhase {
    #[default]
    /// Initial setup phase, configuring simulation parameters
    Setup,
    /// Simulating tectonic plate movement, continent formation, etc. (millions of years)
    TectonicFormation,
    /// Simulating climate, erosion, and resource distribution
    ClimateAndResources,
    /// Simulating the rise of early civilizations
    CivilizationFormation,
    /// Simulating detailed history with characters, events, etc.
    DetailedHistory,
    /// Simulation is complete and ready for exploration/storytelling
    Complete,
}

fn update_simulation(
    time: Res<Time>,
    mut sim_state: ResMut<SimulationState>,
    sim_settings: Res<SimulationSettings>,
) {
    // Only update if simulation is running
    if !sim_state.running {
        return;
    }

    // Get the appropriate years per step based on current phase
    let years_per_step = match sim_state.phase {
        SimulationPhase::TectonicFormation => sim_settings.tectonic_years_per_step,
        SimulationPhase::CivilizationFormation => sim_settings.civilization_years_per_step,
        SimulationPhase::DetailedHistory => sim_settings.history_years_per_step,
        _ => 0, // No year advancement in other phases
    };

    // Advance years based on simulation speed
    if years_per_step > 0 {
        let years_to_advance =
            (years_per_step as f32 * sim_state.speed * time.delta_seconds()) as i64;
        sim_state.year += years_to_advance;
    }
}
