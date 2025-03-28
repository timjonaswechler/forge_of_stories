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

    // Füge Haarfarben-Gene hinzu (wenn du diese Methode in der GeneLibrary implementierst)
    // if let Some((hair_r, hair_g, hair_b)) = gene_library.create_hair_color_genes(species) {
    //     genotype.gene_pairs.insert("gene_hair_r".to_string(), hair_r);
    //     genotype.gene_pairs.insert("gene_hair_g".to_string(), hair_g);
    //     genotype.gene_pairs.insert("gene_hair_b".to_string(), hair_b);
    //
    //     // Chromosomengruppen aktualisieren
    //     // ...
    // }

    // Füge Augenfarben-Gene hinzu
    // ...

    // Erstelle die Entity mit allen notwendigen Komponenten
    commands.spawn((
        genotype,
        Phenotype::new(),
        PhysicalAttributes::default(),
        MentalAttributes::default(),
        SocialAttributes::default(),
        VisualTraits {
            skin_color: (0.8, 0.65, 0.55), // Default, wird später durch Systeme angepasst
            hair_color: (0.3, 0.2, 0.1),   // Default
            eye_color: (0.3, 0.5, 0.7),    // Default
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
