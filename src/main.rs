use bevy::prelude::*;

mod components;
mod plugins;
mod resources;
mod systems;

use components::attributes::{MentalAttributes, PhysicalAttributes, SocialAttributes};
use components::genetics::{
    BodyStructure, ChromosomeType, GeneExpression, GenePair, Genotype, Parent, Personality,
    Phenotype, SpeciesGenes, VisualTraits,
};
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

    // Erstelle verschiedene Charaktere
    create_initial_entity(&mut commands, &gene_library, "Mensch");
    create_initial_entity(&mut commands, &gene_library, "Elf");
    create_initial_entity(&mut commands, &gene_library, "Ork");

    info!("Setup abgeschlossen!");
}

fn create_initial_entity(commands: &mut Commands, gene_library: &Res<GeneLibrary>, species: &str) {
    let mut genotype = Genotype::new();

    // Visuelle Gene (Haut, Haare, Augen) aus der GeneLibrary hinzufügen
    add_visual_genes(&mut genotype, gene_library, species);

    // Attribute-Gene mit zufälligen Werten für "fiktive Eltern" hinzufügen
    add_attribute_genes(&mut genotype);

    // Körperstruktur-Gene hinzufügen
    add_body_structure_genes(&mut genotype);

    // Persönlichkeits-Gene hinzufügen
    add_personality_genes(&mut genotype);

    // Die Entität mit allen Komponenten spawnen
    commands.spawn((
        genotype,
        Phenotype::new(), // Leerer Phänotyp, wird vom System gefüllt
        PhysicalAttributes::default(),
        MentalAttributes::default(),
        SocialAttributes::default(),
        VisualTraits {
            skin_color: (0.8, 0.65, 0.55), // Standardwerte, werden vom System überschrieben
            hair_color: (0.3, 0.2, 0.1),
            eye_color: (0.3, 0.5, 0.7),
        },
        SpeciesGenes {
            species: vec![species.to_string()],
        },
        BodyStructure::humanoid(),
        Personality::default_traits(),
        Parent { children: vec![] },
    ));
}

// Debug-System, das Informationen über die erzeugten Entitäten ausgibt
fn debug_entities(
    query: Query<(
        Entity,
        &Genotype,
        &Phenotype,
        &PhysicalAttributes,
        &MentalAttributes,
        &SocialAttributes,
        &VisualTraits,
        &SpeciesGenes,
        &Personality,
    )>,
    _time: Res<Time>,
    mut state: ResMut<AppState>,
) {
    if state.running {
        // Erster Durchlauf: Markiere als bereit für Debug
        state.running = false;
    } else if !state.running {
        info!("=== DETAILLIERTE ENTITY-INFORMATIONEN ===");

        for (entity, genotype, phenotype, physical, mental, social, visual, species, personality) in
            query.iter()
        {
            info!("Entity: {:?}", entity);
            info!("----------------------------------------");

            // Genotyp ausgeben
            info!("GENOTYP: {} Gene", genotype.gene_pairs.len());
            for (gene_id, gene_pair) in &genotype.gene_pairs {
                info!(
                    "  Gen '{}': Maternal: {:.2}, Paternal: {:.2}, Expression: {:?}",
                    gene_id,
                    gene_pair.maternal.value,
                    gene_pair.paternal.value,
                    gene_pair.maternal.expression
                );
            }

            // Phänotyp ausgeben
            info!("PHÄNOTYP:");
            for (chromosome_type, traits) in &phenotype.expressed_traits {
                info!("  Chromosomentyp: {:?}", chromosome_type);
                for (gene_id, value) in traits {
                    info!("    {}: {:.2}", gene_id, value);
                }
            }

            // Rest der Ausgabe...
            // ...
        }
        state.running = true;
    }
}

// In main.rs hinzufügen
fn add_visual_genes(genotype: &mut Genotype, gene_library: &Res<GeneLibrary>, species: &str) {
    // Hautfarben-Gene
    if let Some((gene_r, gene_g, gene_b)) = gene_library.create_skin_color_genes(species) {
        genotype
            .gene_pairs
            .insert("gene_skin_r".to_string(), gene_r);
        genotype
            .gene_pairs
            .insert("gene_skin_g".to_string(), gene_g);
        genotype
            .gene_pairs
            .insert("gene_skin_b".to_string(), gene_b);

        genotype
            .chromosome_groups
            .entry(ChromosomeType::VisualTraits)
            .or_insert_with(Vec::new)
            .append(&mut vec![
                "gene_skin_r".to_string(),
                "gene_skin_g".to_string(),
                "gene_skin_b".to_string(),
            ]);
    }

    // Haarfarben-Gene
    if let Some((gene_r, gene_g, gene_b)) = gene_library.create_hair_color_genes(species) {
        genotype
            .gene_pairs
            .insert("gene_hair_r".to_string(), gene_r);
        genotype
            .gene_pairs
            .insert("gene_hair_g".to_string(), gene_g);
        genotype
            .gene_pairs
            .insert("gene_hair_b".to_string(), gene_b);

        genotype
            .chromosome_groups
            .entry(ChromosomeType::VisualTraits)
            .or_insert_with(Vec::new)
            .append(&mut vec![
                "gene_hair_r".to_string(),
                "gene_hair_g".to_string(),
                "gene_hair_b".to_string(),
            ]);
    }

    // Augenfarben-Gene
    if let Some((gene_r, gene_g, gene_b)) = gene_library.create_eye_color_genes(species) {
        genotype.gene_pairs.insert("gene_eye_r".to_string(), gene_r);
        genotype.gene_pairs.insert("gene_eye_g".to_string(), gene_g);
        genotype.gene_pairs.insert("gene_eye_b".to_string(), gene_b);

        genotype
            .chromosome_groups
            .entry(ChromosomeType::VisualTraits)
            .or_insert_with(Vec::new)
            .append(&mut vec![
                "gene_eye_r".to_string(),
                "gene_eye_g".to_string(),
                "gene_eye_b".to_string(),
            ]);
    }
}

fn add_attribute_genes(genotype: &mut Genotype) {
    // Physische Attribute
    genotype.add_gene_pair(
        "gene_strength",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
    genotype.add_gene_pair(
        "gene_agility",
        0.6,
        0.6,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
    genotype.add_gene_pair(
        "gene_toughness",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
    genotype.add_gene_pair(
        "gene_endurance",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
    genotype.add_gene_pair(
        "gene_recuperation",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
    genotype.add_gene_pair(
        "gene_disease_resistance",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );

    // Mentale Attribute
    genotype.add_gene_pair(
        "gene_focus",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
    genotype.add_gene_pair(
        "gene_creativity",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
    genotype.add_gene_pair(
        "gene_willpower",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
    genotype.add_gene_pair(
        "gene_analytical_ability",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
    genotype.add_gene_pair(
        "gene_intuition",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
    genotype.add_gene_pair(
        "gene_memory",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );

    // Soziale Attribute
    genotype.add_gene_pair(
        "gene_empathy",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
    genotype.add_gene_pair(
        "gene_leadership",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
    genotype.add_gene_pair(
        "gene_social_awareness",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
    genotype.add_gene_pair(
        "gene_linguistic_ability",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
    genotype.add_gene_pair(
        "gene_negotiation",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );
}

fn add_body_structure_genes(genotype: &mut Genotype) {
    // Grundlegende Körperstruktur-Gene
    genotype.add_gene_pair(
        "gene_body_pelvis_size",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::BodyStructure,
    );
    genotype.add_gene_pair(
        "gene_body_neck_length",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::BodyStructure,
    );
    genotype.add_gene_pair(
        "gene_body_head_size",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::BodyStructure,
    );
}

fn add_personality_genes(genotype: &mut Genotype) {
    // Grundlegende Persönlichkeits-Gene (Big Five)
    genotype.add_gene_pair(
        "gene_openness",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Personality,
    );
    genotype.add_gene_pair(
        "gene_conscientiousness",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Personality,
    );
    genotype.add_gene_pair(
        "gene_extraversion",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Personality,
    );
    genotype.add_gene_pair(
        "gene_agreeableness",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Personality,
    );
    genotype.add_gene_pair(
        "gene_neuroticism",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Personality,
    );

    // Fantasy-spezifische Traits
    genotype.add_gene_pair(
        "gene_courage",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Personality,
    );
    genotype.add_gene_pair(
        "gene_honor",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Personality,
    );
    genotype.add_gene_pair(
        "gene_curiosity",
        0.5,
        0.5,
        GeneExpression::Codominant,
        ChromosomeType::Personality,
    );
}
