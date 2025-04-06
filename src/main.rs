// src/main.rs
use bevy::log::LogPlugin;
use bevy::prelude::*;

use bevy_rand::prelude::{Entropy, EntropyPlugin, GlobalEntropy, WyRand};
use rand::Rng;
use std::ops::Not;
use std::str::FromStr; // Nötig für GeneType::from_str im Debug-System

mod builders;
mod components;
mod config;
mod events;
mod plugins;
mod resources;
mod systems;

// Gezielte Imports
use crate::builders::entity_builder::EntityBuilder;
use crate::components::attributes::{
    Attribute,
    AttributeGroup,
    MentalAttributes,
    PhysicalAttributes,
    SocialAttributes, // Import Attribute und AttributeGroup hier
};
use crate::components::gene_types::GeneType; // GeneType importieren
use crate::components::genetics::{Genotype, Phenotype, SpeciesGenes}; // Genotype importieren
use crate::components::visual_traits::VisualTraits;
use crate::events::genetics_events::{
    ChildBornEvent, EntityInitializedEvent, ReproduceRequestEvent, TemporaryAttributeModifierEvent,
};
use crate::plugins::genetics_plugin::{GeneticsPlugin, GeneticsSystemSet}; // Plugin + Set
use crate::plugins::setup_plugin::{AppState, SetupPlugin};
use crate::resources::gene_library::GeneLibrary;
use crate::resources::genetics_generator::GeneticsGenerator;
use crate::systems::reproduction::reproduction_system;

const FIXED_SEED: u64 = 1234567890;
const USE_FIXED_SEED: bool = true; // true für festen Seed, false für System-Seed

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Forge of Stories".into(),
                    ..default()
                }),
                ..default()
            })
            // Optional: Konfiguriere Logging Level für weniger Spam
            .set(LogPlugin {
                level: bevy::log::Level::INFO, // oder WARN
                filter: "wgpu=error,naga=warn,bevy_render=info,bevy_app=info".to_string(), // Beispiel-Filter
                ..default()
            }),
    );

    if USE_FIXED_SEED {
        app.add_plugins(EntropyPlugin::<WyRand>::with_seed(FIXED_SEED.to_le_bytes()));
        info!("Using fixed RNG seed: {}", FIXED_SEED);
    } else {
        app.add_plugins(EntropyPlugin::<WyRand>::default()); // Standard: Seed aus System Entropy
        info!("Using system entropy for RNG seed.");
    }

    app.add_plugins((
        SetupPlugin, // Lädt Assets und verwaltet den State
        GeneticsPlugin, // Fügt Genetik-Systeme hinzu (laufen nur in AppState::Running)
                     // Füge hier zukünftige Plugins hinzu, z.B. UiPlugin
    ));

    // --- Ressourcen (die nicht im SetupPlugin initialisiert werden) ---
    // GeneLibrary wird jetzt im SetupPlugin initialisiert!
    // GeneLibrary::default() wird nicht mehr benötigt.
    // GeneticsGenerator: Überprüfe, ob es noch gebraucht wird.
    // Wenn ja, kann es hier bleiben oder in ein Plugin verschoben werden.
    app.insert_resource(GeneticsGenerator::default()); // Behalte vorerst

    // --- Events registrieren (kann hier bleiben) ---
    app.add_event::<EntityInitializedEvent>()
        .add_event::<TemporaryAttributeModifierEvent>()
        .add_event::<ReproduceRequestEvent>()
        .add_event::<ChildBornEvent>();

    // --- Systeme hinzufügen ---
    // Startup-Systeme, die erst nach dem Laden laufen sollen:
    app.add_systems(OnEnter(AppState::Running), setup); // <- Neues System, OnEnter

    // Update-Systeme, die nur im Running-State laufen sollen:
    app.add_systems(
        Update,
        (
            send_entity_initialized_events,
            handle_temporary_attribute_modifiers,
            reproduction_system,
            debug_entities, // Debug nach allem anderen
        )
            .run_if(in_state(AppState::Running)), // <- Wichtig!
    );

    app.run();
}

fn setup(
    mut commands: Commands,
    gene_library: Res<GeneLibrary>, // GeneLibrary ist jetzt gefüllt
    genetics_generator: Res<GeneticsGenerator>,
    mut rng_param: GlobalEntropy<WyRand>,
) {
    commands.spawn(Camera2dBundle::default()); // Verwende Camera2dBundle
    info!("AppState::Running erreicht. Erstelle Testcharaktere...");

    let rng: &mut Entropy<WyRand> = &mut *rng_param;

    // Stelle sicher, dass GeneLibrary Daten hat, bevor du create aufrufst
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

fn create_initial_entity<Gen: Rng + ?Sized>(
    commands: &mut Commands,
    gene_library: &Res<GeneLibrary>,
    genetics_generator: &Res<GeneticsGenerator>,
    species: &str,
    rng: &mut Gen,
) -> Entity {
    // Füge eine Überprüfung hinzu, ob die Spezies in der Library existiert
    if !gene_library.attribute_distributions.contains_key(species) {
        error!("Spezies '{}' nicht in GeneLibrary gefunden! Erstellung könnte fehlschlagen oder Defaults verwenden.", species);
        // Optional: Frühzeitig abbrechen oder mit Defaults fortfahren
        // return Entity::PLACEHOLDER; // Beispiel für Abbruch (Entity::PLACEHOLDER existiert nicht, alternative wählen)
    }

    let genotype = genetics_generator.create_initial_genotype(gene_library, species, rng);
    EntityBuilder::create_entity_from_genotype(commands, genotype, vec![species.to_string()])
}

// System zum Senden von EntityInitializedEvents für neue Entitäten
// Wird jetzt ausgelöst, wenn Phenotype *hinzugefügt* wird (normalerweise 1 Frame nach dem Spawnen)
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

// handle_temporary_attribute_modifiers (Jetzt ohne boolean 'attribute_found')
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
            // Verwende Option direkt, um den mutablen Referenz zu speichern
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

// Debug-System (Verwendet jetzt auch GeneType::from_str)
fn debug_entities(
    query: Query<(
        Entity,
        &Genotype,
        &Phenotype,
        &PhysicalAttributes,
        &MentalAttributes,
        &SocialAttributes,
        &VisualTraits, // <- Query verwendet jetzt VisualTraits
        &SpeciesGenes,
    )>,
    mut ran_once: Local<bool>,
    app_state: Res<State<AppState>>, // Zugriff auf State zum Debuggen
) {
    // Füge eine zusätzliche Prüfung hinzu, um sicherzustellen, dass wir im Running State sind
    if *app_state != AppState::Running {
        return;
    }
    if !*ran_once {
        info!(
            "=== DETAILLIERTE ENTITY-INFORMATIONEN (State: {:?}) ===",
            app_state.get()
        );
        for (entity, genotype, phenotype, physical, mental, social, visual, species) in query.iter()
        {
            info!("Entity: {:?}", entity);
            info!("----------------------------------------");
            info!("GENOTYP: {} Gene", genotype.gene_pairs.len());
            info!("PHÄNOTYP:");
            for (chrom_type, attributes) in &phenotype.attribute_groups {
                info!("  Chromosomentyp: {:?}", chrom_type);
                for (attr_id_str, gene_value) in attributes {
                    // Parse für schönere Ausgabe
                    let gene_type_str = GeneType::from_str(attr_id_str).map_or_else(
                        // <-- Use hinzugefügt
                        |_| format!("Unbekannt: '{}'", attr_id_str),
                        |gt| format!("{:?}", gt),
                    );
                    info!(
                        "    {:<30}: Pheno-Wert: {:<6.3} (Expression: {:?})",
                        gene_type_str,
                        gene_value.value(),
                        gene_value.expression()
                    );
                }
            }
            info!("PHYSISCHE ATTRIBUTE:");
            debug_attribute(&physical.strength);
            debug_attribute(&physical.agility);
            debug_attribute(&physical.toughness);
            debug_attribute(&physical.endurance);
            debug_attribute(&physical.recuperation);
            debug_attribute(&physical.disease_resistance);

            info!("MENTALE ATTRIBUTE:");
            debug_attribute(&mental.analytical_ability);
            debug_attribute(&mental.focus);
            debug_attribute(&mental.willpower);
            debug_attribute(&mental.creativity);
            debug_attribute(&mental.intuition);
            debug_attribute(&mental.patience);
            debug_attribute(&mental.memory);
            debug_attribute(&mental.spatial_sense);

            info!("SOZIALE ATTRIBUTE:");
            debug_attribute(&social.empathy);
            debug_attribute(&social.social_awareness);
            debug_attribute(&social.linguistic_ability);
            debug_attribute(&social.musicality);
            debug_attribute(&social.leadership);
            debug_attribute(&social.negotiation);

            info!("VISUELLE MERKMALE:");
            // Konvertiere Color zurück zu sRGBA-Komponenten für die Ausgabe
            let skin_srgba = visual.skin_color.to_srgba();
            info!(
                "  Hautfarbe (sRGB): R={:.3} G={:.3} B={:.3} (A={:.3})",
                skin_srgba.red, // <- Zugriff auf Felder der Srgba-Struktur
                skin_srgba.green,
                skin_srgba.blue,
                skin_srgba.alpha // Optional Alpha ausgeben
            );

            let hair_srgba = visual.hair_color.to_srgba();
            info!(
                "  Haarfarbe (sRGB): R={:.3} G={:.3} B={:.3} (A={:.3})",
                hair_srgba.red, // <- Zugriff auf Felder
                hair_srgba.green,
                hair_srgba.blue,
                hair_srgba.alpha
            );

            let eye_srgba = visual.eye_color.to_srgba();
            info!(
                "  Augenfarbe (sRGB): R={:.3} G={:.3} B={:.3} (A={:.3})",
                eye_srgba.red, // <- Zugriff auf Felder
                eye_srgba.green,
                eye_srgba.blue,
                eye_srgba.alpha
            );
            info!("SPEZIES: {:?}", species.species);
            info!("========================================\n");
        }
        *ran_once = true;
    }
}

// Debug-Hilfsfunktion (unverändert)
fn debug_attribute(attribute: &Attribute) {
    info!(
        "  {:<20} ({:<20}): Base: {:<7.1}, Current: {:<7.1}, Effective: {:<7.1} (Max: {:.0}, Rust: {:?})",
        attribute.name,
        format!("{:?}", attribute.id),
        attribute.base_value,
        attribute.current_value,
        attribute.effective_value,
        attribute.max_value,
        attribute.rust_level
    );
}
