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
    create_entity_with_genes(&mut commands, &gene_library, "Mensch");
    create_entity_with_genes(&mut commands, &gene_library, "Elf");
    create_entity_with_genes(&mut commands, &gene_library, "Ork");

    info!("Setup abgeschlossen!");
}

// In einem passenden System (z.B. in src/systems/creation.rs)
fn create_entity_with_genes(
    commands: &mut Commands,
    gene_library: &Res<GeneLibrary>,
    species: &str,
) {
    // Erstelle einen neuen Genotyp
    let mut genotype = Genotype::new();

    // Füge Hautfarben-Gene hinzu (jetzt RGB Komponenten)
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

        // Füge die Gene auch zu den entsprechenden Chromosomengruppen hinzu
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
        // Füge die Gene auch zu den entsprechenden Chromosomengruppen hinzu
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

    if let Some((gene_r, gene_g, gene_b)) = gene_library.create_eye_color_genes(species) {
        genotype.gene_pairs.insert("gene_eye_r".to_string(), gene_r);
        genotype.gene_pairs.insert("gene_eye_g".to_string(), gene_g);
        genotype.gene_pairs.insert("gene_eye_b".to_string(), gene_b);
        // Füge die Gene auch zu den entsprechenden Chromosomengruppen hinzu
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

    genotype.add_gene_pair(
        "gene_strength", // Gen-ID
        0.5,             // Maternaler Wert (0.0-1.0)
        0.5,             // Paternaler Wert
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );

    // Auch für Beweglichkeit
    genotype.add_gene_pair(
        "gene_agility",
        0.6,
        0.6,
        GeneExpression::Codominant,
        ChromosomeType::Attributes,
    );

    genotype
        .chromosome_groups
        .entry(ChromosomeType::Attributes)
        .or_insert_with(Vec::new)
        .push("gene_strength".to_string());

    genotype
        .chromosome_groups
        .entry(ChromosomeType::Attributes)
        .or_insert_with(Vec::new)
        .push("gene_agility".to_string());

    // Erstelle die Entity mit allen notwendigen Komponenten
    commands.spawn((
        genotype,
        Phenotype::new(),
        PhysicalAttributes::default(),
        MentalAttributes::default(),
        SocialAttributes::default(),
        VisualTraits {
            skin_color: (0.8, 0.65, 0.55), // Default, wird später durch Systeme angepasst
            hair_color: (0.3, 0.2, 0.1),   // Default, wird später durch Systeme angepasst
            eye_color: (0.3, 0.5, 0.7),    // Default, wird später durch Systeme angepasst
        },
        SpeciesGenes {
            species: vec![species.to_string()],
        },
        BodyStructure::humanoid(),
        Personality::default_traits(),
        Parent { children: vec![] },
        //TODO: implementieren eines Fertility-Systems
        // Fertility {
        //     fertility_rate: 0.5,
        //     reproduction_cooldown: None,
        //     compatibility_modifiers: HashMap::new(),
        //     maturity: true,
        // },
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
            for (chrom_type, attributes) in &phenotype.attribute_groups {
                info!("  Chromosomentyp: {:?}", chrom_type);
                for (attr_id, value) in attributes {
                    info!("    {}: {:.2}", attr_id, value);
                }
            }
            // Genotyp-Informationen
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

            // Phänotyp-Informationen
            info!("PHÄNOTYP: {} Attribute", phenotype.attributes.len());
            for (attr_id, value) in &phenotype.attributes {
                info!("  {}: {:.2}", attr_id, value);
            }

            // Physische Attribute
            info!("PHYSISCHE ATTRIBUTE:");
            info!("  Stärke: {:.1}", physical.strength.current_value);
            info!("  Ausdauer: {:.1}", physical.endurance.current_value);
            // usw.

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

            // Spezies
            info!("SPEZIES: {:?}", species.species);

            // Persönlichkeit
            info!("PERSÖNLICHKEIT:");
            for (trait_name, trait_value) in &personality.traits {
                info!("  {}: {:.2}", trait_name, trait_value);
            }

            info!("========================================\n");
        }

        state.running = true;
    }
}
