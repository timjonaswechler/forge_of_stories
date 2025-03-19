use crate::civilization::{Culture, Settlement};
use crate::simulation::SimulationPhase;
use bevy::prelude::*;

mod memory;
mod personality;
mod relationship;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_character_simulation.run_if(in_state(SimulationPhase::DetailedHistory)),
                update_relationships.run_if(in_state(SimulationPhase::DetailedHistory)),
                update_character_actions.run_if(in_state(SimulationPhase::DetailedHistory)),
            ),
        );
    }
}

/// Components for characters and their attributes
#[derive(Component)]
pub struct Character {
    pub name: String,
    pub birth_year: i64,
    pub death_year: Option<i64>,
    pub culture: Entity,
    pub home_settlement: Entity,
    pub current_settlement: Entity,
    pub importance: f32, // Historical importance (0.0 to 1.0)
    pub gender: Gender,
    pub roles: Vec<CharacterRole>,
}

#[derive(Component)]
pub struct Personality {
    pub ambition: f32,     // Drive to achieve goals
    pub courage: f32,      // Willingness to face danger
    pub compassion: f32,   // Empathy for others
    pub loyalty: f32,      // Commitment to groups/individuals
    pub intellect: f32,    // Problem-solving ability
    pub spirituality: f32, // Connection to religious/mystical
    pub extroversion: f32, // Social energy
    pub traits: Vec<PersonalityTrait>,
}

#[derive(Component)]
pub struct CharacterMemory {
    pub significant_events: Vec<(Entity, f32)>, // Event and emotional impact
    pub known_characters: Vec<(Entity, f32)>,   // Character and relationship strength
    pub known_locations: Vec<Entity>,           // Places the character has been
    pub traumas: Vec<Entity>,                   // Traumatic events experienced
    pub achievements: Vec<Entity>,              // Major accomplishments
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharacterRole {
    Ruler,
    Noble,
    Military,
    Religious,
    Merchant,
    Artisan,
    Scholar,
    Peasant,
    Outcast,
    Explorer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonalityTrait {
    Brave,
    Cautious,
    Curious,
    Vengeful,
    Forgiving,
    Ambitious,
    Content,
    Honest,
    Deceitful,
    Generous,
    Greedy,
    // Many more possible traits
}

#[derive(Component)]
pub struct Relationship {
    pub character_a: Entity,
    pub character_b: Entity,
    pub relationship_type: RelationshipType,
    pub strength: f32, // -1.0 (hate) to 1.0 (love/devotion)
    pub start_year: i64,
    pub end_year: Option<i64>,
    pub significant_events: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationshipType {
    Family,
    Friendship,
    Romantic,
    Professional,
    Rivalry,
    Enmity,
    Mentorship,
}

/// System to simulate character life cycles and development
fn update_character_simulation() {
    // This will handle character lifecycle events:
    // 1. Birth and death of significant characters
    // 2. Character development and personality changes
    // 3. Career progression and role changes
    // 4. Formation of memories based on experienced events
}

/// System to update relationships between characters
fn update_relationships() {
    // This will handle interpersonal dynamics:
    // 1. Creating new relationships based on proximity and compatibility
    // 2. Evolving existing relationships based on interactions
    // 3. Forming alliances and rivalries
    // 4. Creating family trees and dynasties
}

/// System to generate character actions based on personality and goals
fn update_character_actions() {
    // This will handle character agency:
    // 1. Determining character goals based on personality and context
    // 2. Planning and executing actions to achieve goals
    // 3. Reacting to events and other characters' actions
    // 4. Creating conflicts and collaborations between characters
}
