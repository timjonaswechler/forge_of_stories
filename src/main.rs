use bevy::prelude::*;
use std::collections::HashMap;

mod components;
mod plugins;
mod systems;

use components::attributes::{MentalAttributes, PhysicalAttributes, SocialAttributes};
use components::genetics::{
    Allele, ChromosomePair, Fertility, GeneExpression, Genotype, Parent, Phenotype,
    SpeciesIdentity, VisualTraits,
};
use plugins::genetics_plugin::GeneticsPlugin;

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
    genotypes: Query<(Entity, &Genotype, &Phenotype, &SpeciesIdentity)>,
    time: Res<Time>,
    mut state: ResMut<AppState>,
) {
    // Ausgabe nur einmal zu Beginn
    if state.running {
        info!("Debugging genetische Informationen:");

        for (entity, genotype, phenotype, species) in genotypes.iter() {
            info!("Entity {:?}: Spezies: {}", entity, species.primary_species);

            info!("  Genotyp: {} Gene", genotype.chromosome_pairs.len());

            info!("  Phänotyp-Werte:");
            for (gene_id, value) in phenotype.attributes.iter() {
                info!("    {}: {:.2}", gene_id, value);
            }

            info!("  Spezies-Anteile:");
            for (species_name, percentage) in species.species_percentage.iter() {
                info!("    {}: {:.1}%", species_name, percentage * 100.0);
            }
        }

        // Bei einem reinen Backend-System könnten wir hier die Simulation beenden
        // state.running = false;
    }
}

fn create_human_entity(commands: &mut Commands) {
    // Erstelle einen Genotyp für einen Menschen
    let mut human_genotype = Genotype::new();

    // Füge einige Gene hinzu (vereinfachtes Beispiel)
    add_gene_pair(
        &mut human_genotype,
        "gene_strength",
        0.7,
        0.65,
        GeneExpression::Dominant,
    );
    add_gene_pair(
        &mut human_genotype,
        "gene_agility",
        0.6,
        0.55,
        GeneExpression::Codominant,
    );
    add_gene_pair(
        &mut human_genotype,
        "gene_focus",
        0.7,
        0.75,
        GeneExpression::Codominant,
    );
    add_gene_pair(
        &mut human_genotype,
        "gene_skin_r",
        0.8,
        0.85,
        GeneExpression::Codominant,
    );
    add_gene_pair(
        &mut human_genotype,
        "gene_skin_g",
        0.65,
        0.7,
        GeneExpression::Codominant,
    );
    add_gene_pair(
        &mut human_genotype,
        "gene_skin_b",
        0.55,
        0.6,
        GeneExpression::Codominant,
    );
    add_gene_pair(
        &mut human_genotype,
        "gene_height_base",
        0.6,
        0.65,
        GeneExpression::Codominant,
    );

    // Spezies-Identität (100% Mensch)
    let mut species_percentage = HashMap::new();
    species_percentage.insert("Mensch".to_string(), 1.0);

    // Erstelle die Entity
    commands.spawn((
        human_genotype,
        Phenotype::new(),
        PhysicalAttributes::default(),
        MentalAttributes::default(),
        SocialAttributes::default(),
        VisualTraits {
            skin_color: (0.8, 0.65, 0.55),
            hair_color: (0.3, 0.2, 0.1),
            eye_color: (0.3, 0.5, 0.7),
            height: 175.0,
            build: 0.5,
            special_features: vec![],
        },
        SpeciesIdentity {
            primary_species: "Mensch".to_string(),
            species_percentage,
        },
        Fertility {
            fertility_rate: 0.8,
            compatibility_modifiers: HashMap::new(),
        },
        Parent { children: vec![] },
    ));
}

fn create_elf_entity(commands: &mut Commands) {
    // Erstelle einen Genotyp für einen Elfen
    let mut elf_genotype = Genotype::new();

    // Füge einige Gene hinzu (vereinfachtes Beispiel)
    add_gene_pair(
        &mut elf_genotype,
        "gene_strength",
        0.5,
        0.45,
        GeneExpression::Recessive,
    );
    add_gene_pair(
        &mut elf_genotype,
        "gene_agility",
        0.85,
        0.9,
        GeneExpression::Dominant,
    );
    add_gene_pair(
        &mut elf_genotype,
        "gene_focus",
        0.8,
        0.85,
        GeneExpression::Dominant,
    );
    add_gene_pair(
        &mut elf_genotype,
        "gene_skin_r",
        0.9,
        0.95,
        GeneExpression::Codominant,
    );
    add_gene_pair(
        &mut elf_genotype,
        "gene_skin_g",
        0.8,
        0.85,
        GeneExpression::Codominant,
    );
    add_gene_pair(
        &mut elf_genotype,
        "gene_skin_b",
        0.75,
        0.8,
        GeneExpression::Codominant,
    );
    add_gene_pair(
        &mut elf_genotype,
        "gene_height_base",
        0.8,
        0.85,
        GeneExpression::Dominant,
    );

    // Spezies-Identität (100% Elf)
    let mut species_percentage = HashMap::new();
    species_percentage.insert("Elf".to_string(), 1.0);

    // Erstelle die Entity
    commands.spawn((
        elf_genotype,
        Phenotype::new(),
        PhysicalAttributes::default(),
        MentalAttributes::default(),
        SocialAttributes::default(),
        VisualTraits {
            skin_color: (0.9, 0.8, 0.75),
            hair_color: (0.9, 0.9, 0.7),
            eye_color: (0.2, 0.7, 0.5),
            height: 190.0,
            build: 0.3,
            special_features: vec!["Spitze Ohren".to_string()],
        },
        SpeciesIdentity {
            primary_species: "Elf".to_string(),
            species_percentage,
        },
        Fertility {
            fertility_rate: 0.5,
            compatibility_modifiers: HashMap::new(),
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
