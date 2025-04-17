// src/attributes/mod.rs
mod components;
mod events;
mod plugin;
mod systems;
// ----- Spezifische Re-Exports -----

// Aus components: Typen und der definierende Trait
pub use components::{
    Attribute,
    AttributeCategory,
    AttributeDistribution,
    AttributeGroup, // Der Trait wird hier definiert und exportiert
    AttributeType,
    MentalAttributes,
    PhysicalAttributes,
    SocialAttributes,
};

pub use events::AttributeUsedEvent;
pub use plugin::AttributesPlugin;

// Aus systems: Die öffentlichen Systemfunktionen
// Wichtig: Wir exportieren den AttributeGroup Trait NICHT erneut aus systems
pub use systems::{
    calculate_effective_attribute_values,
    update_attribute_rust,
    update_attribute_usage,
    // Die (noch) leeren Update-Systeme
    update_mental_attributes,
    update_physical_attributes,
    update_social_attributes,
};

// Hinweis: Wenn 'systems' weitere öffentliche Elemente hätte,
// müssten diese hier auch explizit aufgeführt werden.
