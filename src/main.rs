// src/main.rs
use bevy::prelude::*;

mod builders;
mod components;
mod events;
mod plugins;
mod resources;
mod systems;

use crate::plugins::genetics_plugin::GeneticsSystemSet;
use builders::entity_builder::EntityBuilder;
use components::attributes::{MentalAttributes, PhysicalAttributes, SocialAttributes};
use components::genetics::{Phenotype, SpeciesGenes};
use components::visual_traits::VisualTraits;
use events::genetics_events::{EntityInitializedEvent, TemporaryAttributeModifierEvent};
use plugins::genetics_plugin::GeneticsPlugin;
use resources::gene_library::GeneLibrary;
use resources::genetics_generator::GeneticsGenerator;
use systems::attributes::AttributeGroup; // Hinzugefügt für den Trait

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Forge of Stories".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GeneticsPlugin)
        .add_systems(Startup, setup)
        .insert_resource(GeneLibrary::default())
        .insert_resource(GeneticsGenerator::default())
        // Events registrieren
        .add_event::<EntityInitializedEvent>()
        .add_event::<TemporaryAttributeModifierEvent>()
        // System zum Senden von Events nach der Initialisierung hinzufügen
        .add_systems(
            Update,
            (
                send_entity_initialized_events.after(GeneticsSystemSet::PhysicalTraits),
                handle_temporary_attribute_modifiers,
                debug_entities.after(GeneticsSystemSet::PhysicalTraits),
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    gene_library: Res<GeneLibrary>,
    genetics_generator: Res<GeneticsGenerator>,
) {
    // Kamera
    commands.spawn(Camera2d);

    info!("Erstelle Testcharaktere...");

    // Erstelle verschiedene Charaktere mit dem EntityBuilder
    create_initial_entity(&mut commands, &gene_library, &genetics_generator, "Mensch");
    create_initial_entity(&mut commands, &gene_library, &genetics_generator, "Elf");
    create_initial_entity(&mut commands, &gene_library, &genetics_generator, "Ork");

    info!("Setup abgeschlossen!");
}

// Funktion zum Erstellen einer Entität mit dem EntityBuilder
fn create_initial_entity(
    commands: &mut Commands,
    gene_library: &Res<GeneLibrary>,
    genetics_generator: &Res<GeneticsGenerator>,
    species: &str,
) -> Entity {
    // Erstelle einen vollständigen Genotyp mit dem GeneticsGenerator
    let genotype = genetics_generator.create_initial_genotype(gene_library, species);

    // Verwende den EntityBuilder, um die Entität zu erstellen
    EntityBuilder::create_entity_from_genotype(commands, genotype, vec![species.to_string()])
}

// System zum Senden von EntityInitializedEvents für neue Entitäten
fn send_entity_initialized_events(
    query: Query<(Entity, &SpeciesGenes), Added<Phenotype>>,
    mut entity_initialized_events: EventWriter<EntityInitializedEvent>,
) {
    for (entity, species_genes) in query.iter() {
        // Ein Event senden, dass diese Entität initialisiert wurde
        entity_initialized_events.send(EntityInitializedEvent {
            entity,
            species: species_genes.species.clone(),
        });

        info!(
            "Entität {:?} wurde vollständig initialisiert (Spezies: {:?})",
            entity, species_genes.species
        );
    }
}

// System zur Verarbeitung temporärer Attribut-Modifikatoren
fn handle_temporary_attribute_modifiers(
    mut commands: Commands,
    time: Res<Time>,
    mut temp_modifier_events: EventReader<TemporaryAttributeModifierEvent>,
    mut query: Query<(
        &mut PhysicalAttributes,
        &mut MentalAttributes,
        &mut SocialAttributes,
    )>,
) {
    for event in temp_modifier_events.read() {
        if let Ok((mut physical, mut mental, mut social)) = query.get_mut(event.entity) {
            // Versuche zuerst physische Attribute
            let mut attribute_found = false;

            // Physische Attribute prüfen
            if let Some(attribute) = physical.get_attribute_mut(&event.attribute_id) {
                attribute.current_value += event.value_change;
                attribute_found = true;

                // Man könnte hier eine Komponente mit Timer hinzufügen, um den Effekt später rückgängig zu machen
                // Für dieses Beispiel lassen wir das der Einfachheit halber weg
                info!(
                    "Temporärer Modifikator auf {:?} angewendet: {} um {:.1} für {:.1} Sekunden",
                    event.entity, event.attribute_id, event.value_change, event.duration
                );
            }

            // Wenn nicht in physischen Attributen, prüfe mentale
            if !attribute_found {
                if let Some(attribute) = mental.get_attribute_mut(&event.attribute_id) {
                    attribute.current_value += event.value_change;
                    attribute_found = true;
                }
            }

            // Wenn immer noch nicht gefunden, prüfe soziale
            if !attribute_found {
                if let Some(attribute) = social.get_attribute_mut(&event.attribute_id) {
                    attribute.current_value += event.value_change;
                }
            }
        }
    }
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
    )>,
    mut ran_once: Local<bool>,
) {
    if !*ran_once {
        info!("=== DETAILLIERTE ENTITY-INFORMATIONEN ===");

        for (entity, genotype, phenotype, physical, mental, social, visual, species) in query.iter()
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

        *ran_once = true
    }
}
