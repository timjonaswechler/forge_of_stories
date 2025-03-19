use crate::simulation::SimulationPhase;
use bevy::prelude::*;

mod climate;
mod plates;
mod resources;
mod terrain;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldSettings>().add_systems(
            Update,
            (
                update_tectonic_simulation.run_if(in_state(SimulationPhase::TectonicFormation)),
                update_climate_simulation.run_if(in_state(SimulationPhase::ClimateAndResources)),
            ),
        );
    }
}

#[derive(Resource, Default)]
pub struct WorldSettings {
    /// World size in tiles
    pub width: usize,
    pub height: usize,
    /// Number of tectonic plates
    pub num_plates: usize,
    /// Sea level (0.0 to 1.0)
    pub sea_level: f32,
    /// Random seed for world generation
    pub seed: u64,
}

impl WorldSettings {
    pub fn new() -> Self {
        Self {
            width: 512,
            height: 256,
            num_plates: 12,
            sea_level: 0.5,
            seed: rand::random(),
        }
    }
}

/// Components for the world map and tiles
#[derive(Component)]
pub struct WorldMap;

#[derive(Component)]
pub struct Tile {
    pub x: usize,
    pub y: usize,
    pub elevation: f32,
    pub temperature: f32,
    pub rainfall: f32,
    pub biome: Biome,
    pub plate_id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Biome {
    Ocean,
    Desert,
    Savanna,
    TropicalRainforest,
    Grassland,
    Woodland,
    Forest,
    Taiga,
    Tundra,
    Mountain,
}

impl Default for Biome {
    fn default() -> Self {
        Biome::Ocean
    }
}

/// System to simulate tectonic plate movement and continent formation
fn update_tectonic_simulation() {
    // This will handle the geological simulation of plate tectonics
    // Implementation would involve:
    // 1. Calculating plate movement vectors
    // 2. Handling collision and subduction
    // 3. Updating elevation based on plate interactions
    // 4. Generating mountain ranges at convergent boundaries
}

/// System to simulate climate patterns and biome formation
fn update_climate_simulation() {
    // This will handle climate simulation:
    // 1. Generating global wind patterns
    // 2. Simulating ocean currents
    // 3. Calculating rainfall patterns
    // 4. Determining temperature distributions
    // 5. Assigning biomes based on temperature and rainfall
}
