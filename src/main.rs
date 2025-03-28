use bevy::prelude::*;
use rand_distr::{Distribution, Normal};

mod components;
mod plugins;
mod resources;
mod systems;

use components::attributes::{MentalAttributes, PhysicalAttributes, SocialAttributes};
use components::genetics::{
    BodyStructure, ChromosomeType, GeneExpression, Genotype, Parent, Personality, Phenotype,
    SpeciesGenes, VisualTraits,
};
use plugins::genetics_plugin::GeneticsPlugin;
use resources::skin_color_palette::SkinColorPalette;

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
        .insert_resource(SkinColorPalette::default())
        .add_plugins(GeneticsPlugin)
        .add_systems(Startup, setup)
        // Debug-System, das Informationen über die erzeugten Entitäten ausgibt
        .add_systems(Update, debug_entities)
        .run();
}

fn setup(mut commands: Commands) {
    // Kamera
    commands.spawn(Camera2d);

    info!("Erstelle Testcharaktere...");

    // Ein menschlicher Charakter
    create_human_entity(&mut commands);

    // Ein elfischer Charakter
    create_elf_entity(&mut commands);

    info!("Setup abgeschlossen!");
}

// Debug-System, um die genetischen Informationen anzuzeigen
fn debug_entities(
    genotypes: Query<(Entity, &Genotype, &Phenotype, &VisualTraits, &SpeciesGenes)>,
    _time: Res<Time>,
    state: ResMut<AppState>,
) {
    // Ausgabe nur einmal zu Beginn
    if state.running {
        info!("Debugging genetische Informationen:");

        for (entity, genotype, phenotype, visual_traits, species_genes) in genotypes.iter() {
            info!("Entity {:?}", entity);

            info!("  Genotyp: {} Gene", genotype.gene_pairs.len());

            info!("  Phänotyp-Werte:");
            for (gene_id, value) in phenotype.attributes.iter() {
                info!("    {}: {:.2}", gene_id, value);
            }

            info!(
                "  Hautfarbe: RGB({:.3}, {:.3}, {:.3})",
                visual_traits.skin_color.0, visual_traits.skin_color.1, visual_traits.skin_color.2
            );

            info!("  Spezies: {:?}", species_genes.species);
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
    add_gene(
        &mut human_genotype,
        "gene_strength",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Dominant,
        ChromosomeType::Attributes,
    );
    add_gene(
        &mut human_genotype,
        "gene_agility",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
    add_gene(
        &mut human_genotype,
        "gene_focus",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
    add_gene(
        &mut human_genotype,
        "gene_skin_tone",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Codominant,
        ChromosomeType::VisualTraits,
    );
    add_gene(
        &mut human_genotype,
        "gene_height",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Codominant,
        ChromosomeType::BodyStructure,
    );

    // Spezies-Gene
    add_gene(
        &mut human_genotype,
        "gene_species_Mensch",
        1.0,
        1.0,
        GeneExpression::Dominant,
        ChromosomeType::Specialized,
    );

    // Erstelle die Entity
    let mut species_genes = SpeciesGenes::new();
    species_genes.species.push("Mensch".to_string());

    commands.spawn((
        human_genotype,
        Phenotype::new(),
        PhysicalAttributes::default(),
        MentalAttributes::default(),
        SocialAttributes::default(),
        VisualTraits {
            skin_color: SkinColorPalette::default().colors.get("Mensch").unwrap()[rand::random::<
                usize,
            >()
                % SkinColorPalette::default()
                    .colors
                    .get("Mensch")
                    .unwrap()
                    .len()],
            hair_color: (0.3, 0.2, 0.1),
            eye_color: (0.3, 0.5, 0.7),
        },
        species_genes,
        BodyStructure::humanoid(),
        Personality::default_traits(),
        Parent { children: vec![] },
    ));
}

fn create_elf_entity(commands: &mut Commands) {
    // Erstelle einen Genotyp für einen Elfen
    let mut elf_genotype = Genotype::new();
    let normal_distribution = Normal::new(0.250, 0.035).unwrap();

    // Füge einige Gene hinzu (vereinfachtes Beispiel)
    add_gene(
        &mut elf_genotype,
        "gene_strength",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Recessive,
        ChromosomeType::Attributes,
    );
    add_gene(
        &mut elf_genotype,
        "gene_agility",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Dominant,
        ChromosomeType::Attributes,
    );
    add_gene(
        &mut elf_genotype,
        "gene_focus",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Dominant,
        ChromosomeType::Attributes,
    );
    add_gene(
        &mut elf_genotype,
        "gene_skin_tone",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Dominant,
        ChromosomeType::VisualTraits,
    );
    add_gene(
        &mut elf_genotype,
        "gene_height",
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        normal_distribution.sample(&mut rand::thread_rng()) as f32,
        GeneExpression::Dominant,
        ChromosomeType::BodyStructure,
    );

    // Spezies-Gene
    add_gene(
        &mut elf_genotype,
        "gene_species_Elf",
        1.0,
        1.0,
        GeneExpression::Dominant,
        ChromosomeType::Specialized,
    );

    // Erstelle die Entity
    let mut species_genes = SpeciesGenes::new();
    species_genes.species.push("Elf".to_string());

    commands.spawn((
        elf_genotype,
        Phenotype::new(),
        PhysicalAttributes::default(),
        MentalAttributes::default(),
        SocialAttributes::default(),
        VisualTraits {
            // random skin color from SkinColorPalette
            skin_color: SkinColorPalette::default().colors.get("Elf").unwrap()[rand::random::<
                usize,
            >()
                % SkinColorPalette::default().colors.get("Elf").unwrap().len()],
            hair_color: (0.9, 0.9, 0.7),
            eye_color: (0.2, 0.7, 0.5),
        },
        species_genes,
        BodyStructure::humanoid(),
        Personality::default_traits(),
        Parent { children: vec![] },
    ));
}

// Hilfsfunktion zum Hinzufügen eines Gens
fn add_gene(
    genotype: &mut Genotype,
    gene_id: &str,
    maternal_value: f32,
    paternal_value: f32,
    expression: GeneExpression,
    chromosome_type: ChromosomeType,
) {
    genotype.add_gene_pair(
        gene_id,
        maternal_value,
        paternal_value,
        expression,
        chromosome_type,
    );
}
