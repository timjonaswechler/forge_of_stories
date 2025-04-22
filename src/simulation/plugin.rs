// src/simulation/plugin.rs
use bevy::prelude::*;
// Importiere notwendige Typen und Traits
use crate::{
    attributes::{
        AttributeDistribution, // Diese Struktur ist Teil von SpeciesTemplate, Import evtl. redundant
        AttributeGroup,        // Der Trait zum Setzen von Attributen
        AttributeType,         // Der Enum für die Attribut-Typen
        MentalAttributes,      // Struktur für mentale Attribute
        PhysicalAttributes,    // Struktur für physische Attribute
        SocialAttributes,      // Struktur für soziale Attribute
    },
    initialization::{AppState, GameAssets, SpeciesTemplate},
    // Assets und State
};
// Importiere RNG-bezogene Dinge

use bevy_rand::prelude::GlobalEntropy;
use bevy_rand::prelude::WyRand;

use rand::Rng; // Wichtig für .sample()
use rand_distr::{Distribution as RandDistribution, Normal};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugins(EntropyPlugin::<WyRand>::default());

        // EntropyPlugin NICHT hier registrieren! (Wird im CorePlugin gemacht)
        app.add_systems(OnEnter(AppState::Running), setup_simulation);
    }
}

// --- System zum Setup der Simulation ---
fn setup_simulation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_assets: Res<GameAssets>,
    template_assets: Res<Assets<SpeciesTemplate>>,
    mut rng: GlobalEntropy<WyRand>,
) {
    info!("Setting up simulation entities...");
    // todo!("Implement the setup logic for simulation entities");
    let Some(template_handle) = game_assets.species_templates.first() else {
        error!("No species templates loaded or found in GameAssets!");
        return;
    };

    let Some(species_template) = template_assets.get(template_handle) else {
        error!("Failed to get SpeciesTemplate data from handle for the first species!");
        return;
    };

    info!(
        "Initializing character based on species: {}",
        species_template.species_name
    );

    let mut physical_attributes = PhysicalAttributes::new();
    let mut mental_attributes = MentalAttributes::new();
    let mut social_attributes = SocialAttributes::new();
    let att = physical_attributes.agility.clone();

    for (attribute_type, distribution) in &species_template.attribute_distributions {
        let normal_dist = match Normal::new(distribution.mean, distribution.std_dev) {
            Ok(dist) => dist,
            Err(err) => {
                error!(
                    "Invalid distribution parameters for {:?}: mean={}, std_dev={}. Error: {}",
                    attribute_type, distribution.mean, distribution.std_dev, err
                );
                continue;
            }
        };

        // Korrektur: sample erwartet &mut impl Rng
        let generated_value = normal_dist.sample(rng.as_mut()).max(0.0);

        let mut attribute_found = false;
        if let Some(attribute) = physical_attributes.get_attribute_mut(*attribute_type) {
            attribute.base_value = generated_value;
            attribute.current_value = generated_value;
            attribute_found = true;
        }
        if !attribute_found {
            if let Some(attribute) = mental_attributes.get_attribute_mut(*attribute_type) {
                attribute.base_value = generated_value;
                attribute.current_value = generated_value;
                attribute_found = true;
            }
        }
        if !attribute_found {
            if let Some(attribute) = social_attributes.get_attribute_mut(*attribute_type) {
                attribute.base_value = generated_value;
                attribute.current_value = generated_value;
            }
        }
    }

    let character_entity = commands
        .spawn((
            physical_attributes,
            mental_attributes,
            social_attributes,
            Name::new(format!(
                "{} #{}",
                species_template.species_name,
                rng.as_mut().gen_range(1000..9999)
            )),
        ))
        .id();

    info!(
        "Spawned Character: {:?} ({})",
        character_entity, species_template.species_name
    );
    info!("Attriute: {:?}", att);
    info!("Finished setting up simulation entities.");
}
