// src/main.rs mit EntityBuilder-Integration
use bevy::prelude::*;

mod builders;
mod components;
mod plugins;
mod resources;
mod systems; // Neues Modul für den EntityBuilder

use builders::entity_builder::EntityBuilder;
use builders::genetics_helper::GeneticsHelper;
use components::attributes::{MentalAttributes, PhysicalAttributes, SocialAttributes};
use components::genetics::{Phenotype, SpeciesGenes};
use components::visual_traits::VisualTraits;
use plugins::genetics_plugin::GeneticsPlugin;
use resources::gene_library::GeneLibrary;

// Ressource, um das Programm am Laufen zu halten
#[derive(Resource)]
struct AppState {
    pub running: bool,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Forge of Stories".into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(AppState { running: true })
        .add_plugins(GeneticsPlugin)
        .add_systems(Startup, setup)
        // Füge GeneLibrary als Ressource hinzu
        .insert_resource(GeneLibrary::default())
        // Debug-System, das Informationen über die erzeugten Entitäten ausgibt
        .add_systems(Update, debug_entities)
        .run();
}

fn setup(mut commands: Commands, gene_library: Res<GeneLibrary>) {
    // Kamera
    commands.spawn(Camera2d);

    info!("Erstelle Testcharaktere...");

    // Erstelle verschiedene Charaktere mit dem EntityBuilder
    create_initial_entity(&mut commands, &gene_library, "Mensch");
    create_initial_entity(&mut commands, &gene_library, "Elf");
    create_initial_entity(&mut commands, &gene_library, "Ork");

    info!("Setup abgeschlossen!");
}

// Funktion zum Erstellen einer Entität mit dem EntityBuilder
fn create_initial_entity(
    commands: &mut Commands,
    gene_library: &Res<GeneLibrary>,
    species: &str,
) -> Entity {
    // Erstelle einen vollständigen Genotyp mit dem GeneticsHelper
    let genotype = GeneticsHelper::create_initial_genotype(gene_library, species);

    // Verwende den EntityBuilder, um die Entität zu erstellen
    EntityBuilder::create_entity_from_genotype(
        commands,
        genotype,
        vec![species.to_string()],
        gene_library,
    )
}

// Debug-System, das Informationen über die erzeugten Entitäten ausgibt
fn debug_entities(
    query: Query<(
        Entity,
        &components::genetics::Genotype,
        &Phenotype,
        &PhysicalAttributes,
        &MentalAttributes,
        &SocialAttributes,
        &VisualTraits,
        &SpeciesGenes,
        // &components::genetics::Personality,
    )>,
    // time: Res<Time>,
    mut state: ResMut<AppState>,
) {
    if state.running {
        // Erster Durchlauf: Markiere als bereit für Debug
        state.running = true;
    } else if !state.running {
        info!("=== DETAILLIERTE ENTITY-INFORMATIONEN ===");

        for (
            entity,
            genotype,
            phenotype,
            physical,
            mental,
            social,
            visual,
            species, /* personality */
        ) in query.iter()
        {
            info!("Entity: {:?}", entity);
            info!("----------------------------------------");

            // Genotyp-Informationen
            info!("GENOTYP: {} Gene", genotype.gene_pairs.len());
            for (gene_id, gene_pair) in &genotype.gene_pairs {
                info!(
                    "  Gen '{}': Maternal: value: {:.2}, Expression: {:?}, Paternal: value: {:.2}, Expression: {:?}",
                    gene_id,
                    gene_pair.maternal.value,
                    gene_pair.maternal.expression,
                    gene_pair.paternal.value,
                    gene_pair.paternal.expression
                );
            }

            // Phänotyp-Informationen
            info!("PHÄNOTYP:");
            for (chrom_type, attributes) in &phenotype.attribute_groups {
                info!("  Chromosomentyp: {:?}", chrom_type);
                for (attr_id, gene_value) in attributes {
                    info!(
                        "    {}: {:.2} (Expression: {:?})",
                        attr_id,
                        gene_value.value(),
                        gene_value.expression()
                    );
                }
            }

            // Physische Attribute
            info!("PHYSISCHE ATTRIBUTE:");
            info!("  Stärke: {:.1}", physical.strength.current_value);
            info!("  Beweglichkeit: {:.1}", physical.agility.current_value);
            info!(
                "  Widerstandsfähigkeit: {:.1}",
                physical.toughness.current_value
            );
            info!("  Ausdauer: {:.1}", physical.endurance.current_value);
            info!(
                "  Heilungsfähigkeit: {:.1}",
                physical.recuperation.current_value
            );
            info!(
                "  Krankheitsresistenz: {:.1}",
                physical.disease_resistance.current_value
            );

            // Visualtraits
            info!("VISUELLE MERKMALE:");
            info!(
                "  Hautfarbe: RGB({:.3}, {:.3}, {:.3})",
                visual.skin_color.0, visual.skin_color.1, visual.skin_color.2
            );
            info!(
                "  Haarfarbe: RGB({:.3}, {:.3}, {:.3})",
                visual.hair_color.0, visual.hair_color.1, visual.hair_color.2
            );
            info!(
                "  Augenfarbe: RGB({:.3}, {:.3}, {:.3})",
                visual.eye_color.0, visual.eye_color.1, visual.eye_color.2
            );

            // Spezies
            info!("SPEZIES: {:?}", species.species);

            info!("========================================\n");
        }

        state.running = false;
    }
}
