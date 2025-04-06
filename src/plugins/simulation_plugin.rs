// Neue Datei: src/plugins/simulation_plugin.rs
use bevy::prelude::*;
use bevy_rand::prelude::{Entropy, GlobalEntropy, WyRand};
use rand::Rng;

use super::setup_plugin::AppState; // Importiere AppState für run_if/OnEnter
use crate::builders::entity_builder::EntityBuilder;
use crate::components::attributes::{
    Attribute, AttributeGroup, MentalAttributes, PhysicalAttributes, SocialAttributes,
};
use crate::components::genetics::{Genotype, Phenotype, SpeciesGenes};
use crate::events::genetics_events::{EntityInitializedEvent, TemporaryAttributeModifierEvent};
use crate::resources::gene_library::GeneLibrary;
use crate::resources::genetics_generator::GeneticsGenerator;
use crate::systems::reproduction::reproduction_system;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Ressourcen, die für die Simulation gebraucht werden
            .insert_resource(GeneticsGenerator::default())
            // Startup-Systeme (laufen nur, wenn State erreicht wird)
            .add_systems(OnEnter(AppState::Running), spawn_initial_characters)
            // Update-Systeme (laufen nur im Running State)
            .add_systems(
                Update,
                (
                    send_entity_initialized_events,
                    handle_temporary_attribute_modifiers,
                    reproduction_system, // Dieses System war schon in systems::reproduction
                )
                    .run_if(in_state(AppState::Running)),
            );
    }
}

// --- Systeme, die vorher in main.rs waren ---

// Umbenannt von setup zu spawn_initial_characters
fn spawn_initial_characters(
    mut commands: Commands,
    gene_library: Res<GeneLibrary>,
    genetics_generator: Res<GeneticsGenerator>,
    mut rng_param: GlobalEntropy<WyRand>,
) {
    commands.spawn(Camera2dBundle::default());
    info!("AppState::Running erreicht. Erstelle Testcharaktere...");

    let rng: &mut Entropy<WyRand> = &mut *rng_param;

    if gene_library.attribute_distributions.is_empty() {
        error!("GeneLibrary ist leer, kann keine Entitäten erstellen! Asset-Laden fehlgeschlagen?");
        return;
    }

    let _mensch = create_initial_entity(
        &mut commands,
        &gene_library,
        &genetics_generator,
        "Mensch",
        rng,
    );
    let _elf = create_initial_entity(
        &mut commands,
        &gene_library,
        &genetics_generator,
        "Elf",
        rng,
    );
    let _ork = create_initial_entity(
        &mut commands,
        &gene_library,
        &genetics_generator,
        "Ork",
        rng,
    );

    info!("Testcharaktere erstellt!");
}

// Diese Funktion wird von spawn_initial_characters benötigt
fn create_initial_entity<Gen: Rng + ?Sized>(
    commands: &mut Commands,
    gene_library: &Res<GeneLibrary>,
    genetics_generator: &Res<GeneticsGenerator>,
    species: &str,
    rng: &mut Gen,
) -> Entity {
    if !gene_library.attribute_distributions.contains_key(species) {
        error!(
            "Spezies '{}' nicht in GeneLibrary gefunden! Erstellung könnte fehlschlagen oder Defaults verwenden.",
            species
        );
    }

    let genotype = genetics_generator.create_initial_genotype(gene_library, species, rng);
    EntityBuilder::create_entity_from_genotype(commands, genotype, vec![species.to_string()])
}

// System zum Senden von EntityInitializedEvents
fn send_entity_initialized_events(
    query: Query<(Entity, &SpeciesGenes), Added<Phenotype>>,
    mut entity_initialized_events: EventWriter<EntityInitializedEvent>,
) {
    for (entity, species_genes) in query.iter() {
        entity_initialized_events.send(EntityInitializedEvent {
            entity,
            species: species_genes.species.clone(),
        });
        info!(
            "Entity {:?} wurde initialisiert (Phenotype hinzugefügt, Spezies: {:?})",
            entity, species_genes.species
        );
    }
}

// handle_temporary_attribute_modifiers
fn handle_temporary_attribute_modifiers(
    _commands: Commands,
    _time: Res<Time>,
    mut temp_modifier_events: EventReader<TemporaryAttributeModifierEvent>,
    mut query: Query<(
        &mut PhysicalAttributes,
        &mut MentalAttributes,
        &mut SocialAttributes,
    )>,
) {
    for event in temp_modifier_events.read() {
        if let Ok((mut physical, mut mental, mut social)) = query.get_mut(event.entity) {
            let attribute_ref_option = physical
                .get_attribute_mut(event.attribute_id)
                .or_else(|| mental.get_attribute_mut(event.attribute_id))
                .or_else(|| social.get_attribute_mut(event.attribute_id));

            if let Some(attribute) = attribute_ref_option {
                let old_value = attribute.current_value;
                attribute.current_value += event.value_change;
                attribute.current_value = attribute.current_value.clamp(0.0, attribute.max_value);

                info!(
                    "TempMod angewendet auf Entität {:?}: Attribut '{}' ({:?}) geändert von {:.1} um {:+.1} -> Neuer Wert: {:.1} (Dauer: {:.1}s)",
                    event.entity,
                    attribute.name,
                    event.attribute_id,
                    old_value,
                    event.value_change,
                    attribute.current_value,
                    event.duration
                );
            } else {
                warn!(
                    "Attribut Enum '{:?}' für temporären Modifikator auf {:?} konnte in keiner Attributgruppe gefunden werden.",
                    event.attribute_id, event.entity
                );
            }
        } else {
            warn!("Entität {:?} für TempMod nicht gefunden.", event.entity);
        }
    }
}
