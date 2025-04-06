// src/simulation/systems/character_spawner.rs
use crate::simulation::builders::entity_builder::EntityBuilder;
use crate::simulation::resources::gene_library::GeneLibrary;
use crate::simulation::resources::genetics_generator::GeneticsGenerator;
use bevy::prelude::*;
use bevy_rand::prelude::{Entropy, GlobalEntropy, WyRand};
use rand::Rng;

// Umbenannt von setup zu spawn_initial_characters
pub fn spawn_initial_characters(
    mut commands: Commands,
    gene_library: Res<GeneLibrary>,
    genetics_generator: Res<GeneticsGenerator>,
    mut rng_param: GlobalEntropy<WyRand>,
) {
    commands.spawn(Camera2d::default());
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
