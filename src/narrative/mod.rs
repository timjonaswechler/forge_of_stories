use crate::character::Character;
use crate::simulation::SimulationPhase;
use bevy::prelude::*;

mod artifact;
mod event;
mod story;

pub struct NarrativePlugin;

impl Plugin for NarrativePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                generate_historical_events.run_if(in_state(SimulationPhase::DetailedHistory)),
                identify_emergent_stories.run_if(in_state(SimulationPhase::DetailedHistory)),
                create_artifacts.run_if(in_state(SimulationPhase::DetailedHistory)),
            ),
        );
    }
}

/// Components for narrative elements
#[derive(Component)]
pub struct Event {
    pub name: String,
    pub description: String,
    pub year: i64,
    pub location: Entity,          // Where the event happened
    pub participants: Vec<Entity>, // Characters involved
    pub importance: f32,           // Historical significance (0.0 to 1.0)
    pub event_type: EventType,
    pub causes: Vec<Entity>,       // Events that led to this one
    pub consequences: Vec<Entity>, // Events caused by this one
}

#[derive(Component)]
pub struct Story {
    pub title: String,
    pub description: String,
    pub start_year: i64,
    pub end_year: Option<i64>,         // None if ongoing
    pub major_events: Vec<Entity>,     // Key events in this story
    pub major_characters: Vec<Entity>, // Key characters in this story
    pub story_type: StoryType,
    pub themes: Vec<StoryTheme>,
    pub importance: f32, // Historical significance (0.0 to 1.0)
}

#[derive(Component)]
pub struct Artifact {
    pub name: String,
    pub description: String,
    pub creation_year: i64,
    pub creator: Option<Entity>,
    pub current_location: Entity,
    pub current_owner: Option<Entity>,
    pub powers: Vec<ArtifactPower>,
    pub historical_events: Vec<Entity>, // Events this artifact was involved in
    pub significance: f32,              // Cultural/historical importance (0.0 to 1.0)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventType {
    Battle,
    Coronation,
    Treaty,
    Discovery,
    Disaster,
    Migration,
    Founding,
    Religious,
    Cultural,
    Technological,
    Personal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoryType {
    War,
    Succession,
    Romance,
    Quest,
    Rivalry,
    Rise,
    Fall,
    Transformation,
    Discovery,
    Revenge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoryTheme {
    PowerCorrupts,
    Redemption,
    Sacrifice,
    Justice,
    Identity,
    Loyalty,
    Betrayal,
    Hubris,
    Love,
    Loss,
    Survival,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactPower {
    None,
    Symbolic,  // Important for cultural/religious reasons
    Political, // Grants legitimate authority
    Military,  // Provides tactical advantage
    Knowledge, // Contains or grants special knowledge
    Magical,   // Has supernatural properties
}

/// System to generate significant historical events
fn generate_historical_events() {
    // This system will:
    // 1. Identify potential significant moments (battles, deaths of rulers, etc.)
    // 2. Create event entities with appropriate details
    // 3. Connect events to characters and locations
    // 4. Establish causal relationships between events
}

/// System to identify and track emergent story arcs
fn identify_emergent_stories() {
    // This system will:
    // 1. Analyze patterns of connected events
    // 2. Identify narrative structures (rise and fall, quest, etc.)
    // 3. Recognize key characters in each story
    // 4. Assign themes based on event patterns
    // 5. Track ongoing stories and determine when they conclude
}

/// System to create and track significant artifacts
fn create_artifacts() {
    // This system will:
    // 1. Create significant items during important events
    // 2. Track ownership and location changes over time
    // 3. Accumulate historical significance based on involvement in events
    // 4. Generate lore and descriptions for artifacts
}
