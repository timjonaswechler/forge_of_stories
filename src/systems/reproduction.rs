// src/systems/reproduction.rs
use bevy::prelude::*;
use rand::prelude::*;
// use bevy_prng::ChaCha8Rng; // TODO: Wenn bevy_prng verwendet wird
// use bevy_rand::prelude::{GlobalEntropy, ForkableRng}; // TODO: Wenn bevy_rand verwendet wird

use crate::builders::entity_builder::EntityBuilder;
use crate::components::gene_types::{GeneType, VisualGene}; // Für Gen-Typ-Prüfung
use crate::components::genetics::{GeneExpression, GenePair, GeneVariant, Genotype, SpeciesGenes}; // GeneExpression hinzugefügt
use crate::components::visual_traits::EyeColor;
use crate::events::genetics_events::{ChildBornEvent, ReproduceRequestEvent};
use crate::resources::eye_color_inheritance::EyeColorInheritance;

// System, das auf Reproduktionsanfragen reagiert
pub fn reproduction_system(
    mut commands: Commands,
    mut reproduce_requests: EventReader<ReproduceRequestEvent>,
    mut child_born_events: EventWriter<ChildBornEvent>,
    parent_query: Query<(&Genotype, &SpeciesGenes)>, // Zum Abrufen der Elterndaten
    eye_inheritance: Res<EyeColorInheritance>,
    // TODO: rng: ResMut<GlobalEntropy<ChaCha8Rng>>, // Beispiel für bevy_rand RNG
) {
    // TODO: RNG als Resource oder Parameter verwenden statt thread_rng()
    let mut rng = rand::thread_rng();

    for event in reproduce_requests.read() {
        let parent1_data = parent_query.get(event.parent1);
        let parent2_data = parent_query.get(event.parent2);

        if let (Ok((p1_genotype, p1_species)), Ok((p2_genotype, p2_species))) =
            (parent1_data, parent2_data)
        {
            // Erzeuge den Genotyp und Spezies-Info des Kindes
            let (child_genotype, child_species) = create_child_genotype(
                p1_genotype,
                p2_genotype,
                p1_species,
                p2_species,
                &eye_inheritance,
                &mut rng, // TODO: Verwende RNG Resource
            );

            // Spawne die Entität für das Kind
            let child_entity = EntityBuilder::create_entity_from_genotype(
                &mut commands,
                child_genotype,
                child_species.species, // Gib die kombinierten Spezies weiter
            );

            info!(
                "Kind {:?} wurde von {:?} und {:?} geboren.",
                child_entity, event.parent1, event.parent2
            );

            child_born_events.send(ChildBornEvent {
                child: child_entity,
                parent1: event.parent1,
                parent2: event.parent2,
            });
        } else {
            warn!(
                "Konnte Reproduktions-Partner nicht finden oder Daten fehlen für Event: {:?}",
                event
            );
        }
    }
}

/// Erstellt den Genotyp und die Spezies-Information für ein Kind aus zwei Eltern.
/// Enthält eine robustere Gen-Iterationslogik.
fn create_child_genotype<R: Rng + ?Sized>(
    p1_genotype: &Genotype,
    p2_genotype: &Genotype,
    p1_species: &SpeciesGenes,
    p2_species: &SpeciesGenes,
    eye_inheritance: &Res<EyeColorInheritance>,
    rng: &mut R,
) -> (Genotype, SpeciesGenes) {
    let mut child_genotype = Genotype::new();

    // --- Kind Spezies bestimmen ---
    let mut combined_species: Vec<String> = p1_species
        .species
        .iter()
        .chain(p2_species.species.iter())
        .cloned()
        .collect();
    combined_species.sort();
    combined_species.dedup();
    let child_species = SpeciesGenes {
        species: combined_species,
    };
    let primary_species = child_species
        .species
        .first()
        .map(|s| s.as_str())
        .unwrap_or("Mensch");

    // --- Gene vererben (Robustere Methode) ---
    // 1. Sammle alle einzigartigen Gen-IDs beider Elternteile
    let mut all_gene_ids: Vec<String> = p1_genotype
        .gene_pairs
        .keys()
        .chain(p2_genotype.gene_pairs.keys())
        .cloned()
        .collect();
    all_gene_ids.sort();
    all_gene_ids.dedup();

    // 2. Iteriere über alle einzigartigen Gen-IDs
    for gene_id in all_gene_ids {
        let p1_gene_pair_opt = p1_genotype.gene_pairs.get(&gene_id);
        let p2_gene_pair_opt = p2_genotype.gene_pairs.get(&gene_id);

        // Bestimme die Allele für das Kind
        match (p1_gene_pair_opt, p2_gene_pair_opt) {
            (Some(p1_gene_pair), Some(p2_gene_pair)) => {
                // Beide Eltern haben das Gen
                let child_maternal_allele: GeneVariant;
                let child_paternal_allele: GeneVariant;

                // Spezielle Augenfarbe Logik
                let eye_color_gene_id = GeneType::Visual(VisualGene::EyeColor).to_string();
                if gene_id == eye_color_gene_id {
                    let p1_chosen_value = if rng.gen() {
                        p1_gene_pair.maternal.value
                    } else {
                        p1_gene_pair.paternal.value
                    };
                    let p2_chosen_value = if rng.gen() {
                        p2_gene_pair.maternal.value
                    } else {
                        p2_gene_pair.paternal.value
                    };
                    let p1_chosen_eye = EyeColor::from_f32(p1_chosen_value);
                    let p2_chosen_eye = EyeColor::from_f32(p2_chosen_value);

                    // TODO: Verwende RNG Resource statt internem rng in inherit_eye_color
                    let child_resulting_eye = eye_inheritance.inherit_eye_color(
                        primary_species,
                        p1_chosen_eye,
                        p2_chosen_eye,
                    );
                    let child_resulting_value = child_resulting_eye.to_f32();

                    child_maternal_allele = GeneVariant {
                        gene_id: gene_id.clone(),
                        value: child_resulting_value,
                        expression: GeneExpression::Codominant,
                    };
                    child_paternal_allele = GeneVariant {
                        gene_id: gene_id.clone(),
                        value: child_resulting_value,
                        expression: GeneExpression::Codominant,
                    };
                } else {
                    // Standard-Vererbung
                    child_maternal_allele = if rng.gen() {
                        p1_gene_pair.maternal.clone()
                    } else {
                        p1_gene_pair.paternal.clone()
                    };
                    child_paternal_allele = if rng.gen() {
                        p2_gene_pair.maternal.clone()
                    } else {
                        p2_gene_pair.paternal.clone()
                    };
                }

                let child_gene_pair = GenePair {
                    maternal: child_maternal_allele,
                    paternal: child_paternal_allele,
                    chromosome_type: p1_gene_pair.chromosome_type, // Nimm von P1 an
                };
                child_genotype
                    .gene_pairs
                    .insert(gene_id.clone(), child_gene_pair);
                child_genotype
                    .chromosome_groups
                    .entry(p1_gene_pair.chromosome_type)
                    .or_default()
                    .push(gene_id);
            }
            (Some(p1_gene_pair), None) => {
                // Nur Parent 1 hat das Gen -> Kind bekommt zwei Kopien von P1's Allelen
                warn!(
                    "Gen '{}' nur in Parent 1 gefunden. Kind erhält Kopie.",
                    gene_id
                );
                let allele1 = if rng.gen() {
                    p1_gene_pair.maternal.clone()
                } else {
                    p1_gene_pair.paternal.clone()
                };
                let allele2 = if rng.gen() {
                    p1_gene_pair.maternal.clone()
                } else {
                    p1_gene_pair.paternal.clone()
                }; // Nochmals von P1 wählen

                let child_gene_pair = GenePair {
                    maternal: allele1,
                    paternal: allele2,
                    chromosome_type: p1_gene_pair.chromosome_type,
                };
                child_genotype
                    .gene_pairs
                    .insert(gene_id.clone(), child_gene_pair);
                child_genotype
                    .chromosome_groups
                    .entry(p1_gene_pair.chromosome_type)
                    .or_default()
                    .push(gene_id);
            }
            (None, Some(p2_gene_pair)) => {
                // Nur Parent 2 hat das Gen -> Kind bekommt zwei Kopien von P2's Allelen
                warn!(
                    "Gen '{}' nur in Parent 2 gefunden. Kind erhält Kopie.",
                    gene_id
                );
                let allele1 = if rng.gen() {
                    p2_gene_pair.maternal.clone()
                } else {
                    p2_gene_pair.paternal.clone()
                };
                let allele2 = if rng.gen() {
                    p2_gene_pair.maternal.clone()
                } else {
                    p2_gene_pair.paternal.clone()
                };

                let child_gene_pair = GenePair {
                    maternal: allele1,
                    paternal: allele2,
                    chromosome_type: p2_gene_pair.chromosome_type,
                };
                child_genotype
                    .gene_pairs
                    .insert(gene_id.clone(), child_gene_pair);
                child_genotype
                    .chromosome_groups
                    .entry(p2_gene_pair.chromosome_type)
                    .or_default()
                    .push(gene_id);
            }
            (None, None) => {
                // Sollte nicht vorkommen, wenn wir über vereinigte Schlüssel iterieren
                error!(
                    "Unerwarteter Fall: Gen '{}' in keinem Elternteil gefunden.",
                    gene_id
                );
            }
        }
    }

    // TODO: Mutation hinzufügen (nach der Gen-Selektion, vor dem Einfügen?)

    (child_genotype, child_species)
}
