use bevy::prelude::*;
use rand_distr::{Distribution, Normal};
use std::collections::HashMap;

mod components;
mod plugins;
mod resources;
mod systems; // Neues Modul

use components::attributes::{MentalAttributes, PhysicalAttributes, SocialAttributes};
use components::genetics::{
    Allele, ChromosomePair, GeneExpression, Genotype, Parent, Phenotype, VisualTraits,
};
use plugins::genetics_plugin::GeneticsPlugin;
use resources::skin_color_palette::SkinColorPalette; // Import der Hautfarben-Ressource

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
        .insert_resource(SkinColorPalette::default()) // Hautfarbenpalette hinzufügen
        .add_plugins(GeneticsPlugin)
        .add_systems(Startup, setup)
        // Debug-System, das Informationen über die erzeugten Entitäten ausgibt
        .add_systems(Update, debug_entities)
        .run();
}

fn setup(mut commands: Commands) {
    // Kamera (mit aktualisierten API)
    commands.spawn(Camera2dBundle::default());

    info!("Erstelle Testcharaktere...");

    // Ein menschlicher Charakter
    create_human_entity(&mut commands);

    // Ein elfischer Charakter
    create_elf_entity(&mut commands);

    info!("Setup abgeschlossen!");
}

// Debug-System, um die genetischen Informationen anzuzeigen
fn debug_entities(
    genotypes: Query<(Entity, &Genotype, &Phenotype, &VisualTraits)>,
    time: Res<Time>,
    mut state: ResMut<AppState>,
) {
    // Ausgabe nur einmal zu Beginn
    if state.running {
        info!("Debugging genetische Informationen:");

        for (entity, genotype, phenotype, visual_traits) in genotypes.iter() {
            info!("Entity {:?}", entity);

            info!("  Genotyp: {} Gene", genotype.chromosome_pairs.len());

            info!("  Phänotyp-Werte:");
            for (gene_id, value) in phenotype.attributes.iter() {
                info!("    {}: {:.2}", gene_id, value);
            }

            info!(
                "  Hautfarbe: RGB({:.3}, {:.3}, {:.3})",
                visual_traits.skin_color.0, visual_traits.skin_color.1, visual_traits.skin_color.2
            );
        }

        // Bei einem reinen Backend-System könnten wir hier die Simulation beenden
        // state.running = false;
    }
}

fn create_human_entity(commands: &mut Commands) {
    // Erstelle einen Genotyp für einen Menschen
    let mut human_genotype = Genotype::new();
    let normal_distribution = Normal::new(0.250, 0.035).unwrap();

    // Füge einige Gene hinzu (vereinfachtes Beispiel)
    add_gene_pair(
        &mut human_genotype,
        "gene_strength",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Dominant,
    );
    add_gene_pair(
        &mut human_genotype,
        "gene_agility",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Codominant,
    );
    add_gene_pair(
        &mut human_genotype,
        "gene_focus",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Codominant,
    );
    add_gene_pair(
        &mut human_genotype,
        "gene_skin_tone",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Codominant,
    ); // Hautton im Spektrum
    add_gene_pair(
        &mut human_genotype,
        "gene_height_base",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Codominant,
    );

    // Erstelle die Entity
    commands.spawn((
        human_genotype,
        Phenotype::new(),
        PhysicalAttributes::default(),
        MentalAttributes::default(),
        SocialAttributes::default(),
        VisualTraits {
            skin_color: (0.8, 0.65, 0.55), // Wird durch System ersetzt
            hair_color: (0.3, 0.2, 0.1),
            eye_color: (0.3, 0.5, 0.7),
        },
        Parent { children: vec![] },
    ));
}

fn create_elf_entity(commands: &mut Commands) {
    // Erstelle einen Genotyp für einen Elfen
    let mut elf_genotype = Genotype::new();
    let normal_distribution = Normal::new(0.250, 0.035).unwrap();

    // Füge einige Gene hinzu (vereinfachtes Beispiel)
    add_gene_pair(
        &mut elf_genotype,
        "gene_strength",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Recessive,
    );
    add_gene_pair(
        &mut elf_genotype,
        "gene_agility",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Dominant,
    );
    add_gene_pair(
        &mut elf_genotype,
        "gene_focus",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Dominant,
    );
    add_gene_pair(
        &mut elf_genotype,
        "gene_skin_tone",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Dominant,
    ); // Hellere Haut
    add_gene_pair(
        &mut elf_genotype,
        "gene_height_base",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Dominant,
    );

    // Erstelle die Entity
    commands.spawn((
        elf_genotype,
        Phenotype::new(),
        PhysicalAttributes::default(),
        MentalAttributes::default(),
        SocialAttributes::default(),
        VisualTraits {
            skin_color: (0.9, 0.8, 0.75), // Wird durch System ersetzt
            hair_color: (0.9, 0.9, 0.7),
            eye_color: (0.2, 0.7, 0.5),
        },
        Parent { children: vec![] },
    ));
}

// Hilfsfunktion zum Hinzufügen eines Genpaars
fn add_gene_pair(
    genotype: &mut Genotype,
    gene_id: &str,
    maternal_value: f32,
    paternal_value: f32,
    expression: GeneExpression,
) {
    let chromosome_pair = ChromosomePair {
        maternal: Allele {
            gene_id: gene_id.to_string(),
            value: maternal_value,
            expression,
        },
        paternal: Allele {
            gene_id: gene_id.to_string(),
            value: paternal_value,
            expression,
        },
    };

    genotype
        .chromosome_pairs
        .insert(gene_id.to_string(), chromosome_pair);
}
