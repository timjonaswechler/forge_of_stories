// src/simulation/mod.rs
pub mod builders;
pub mod events;
pub mod plugin;
pub mod resources; // Verweist auf den Ordner resources/
pub mod systems;

// Re-exportiere Typen
pub use builders::entity_builder::EntityBuilder;
pub use events::{
    // Re-export aus events.rs (das die Events enthält)
    ChildBornEvent,
    EntityInitializedEvent,
    ReproduceRequestEvent,
    TemporaryAttributeModifierEvent,
};
pub use plugin::SimulationPlugin;

pub use resources::gene_library::GeneLibrary; // Re-exportiere GeneLibrary
pub use resources::genetics_generator::GeneticsGenerator; // Re-exportiere SpeciesData
