// src/systems/reproduction.rs
use bevy::prelude::*;
use rand::Rng; // Für gen()/gen_range()

// KORRIGIERT: Import aus bevy_rand::prelude
use bevy_rand::prelude::{Entropy, GlobalEntropy, WyRand};

use crate::genetics::components::gene_types::{GeneType, VisualGene};
use crate::genetics::components::genetics::{
    GeneExpression, GenePair, GeneVariant, Genotype, SpeciesGenes,
};
use crate::simulation::builders::entity_builder::EntityBuilder;
use crate::simulation::events::{ChildBornEvent, ReproduceRequestEvent};
use crate::visuals::components::EyeColor;
use crate::visuals::resources::EyeColorInheritance; // Stelle Pfad sicher

pub fn reproduction_system(
    mut commands: Commands,
    mut reproduce_requests: EventReader<ReproduceRequestEvent>,
    mut child_born_events: EventWriter<ChildBornEvent>,
    parent_query: Query<(&Genotype, &SpeciesGenes)>,
    eye_inheritance: Res<EyeColorInheritance>,
    // Korrekter SystemParam
    mut rng_param: GlobalEntropy<WyRand>,
) {
    // Korrektes Dereferenzieren
    let rng: &mut Entropy<WyRand> = &mut *rng_param;

    for event in reproduce_requests.read() {
        let parent1_data = parent_query.get(event.parent1);
        let parent2_data = parent_query.get(event.parent2);

        if let (Ok((p1_genotype, p1_species)), Ok((p2_genotype, p2_species))) =
            (parent1_data, parent2_data)
        {
            let (child_genotype, child_species) = create_child_genotype(
                p1_genotype,
                p2_genotype,
                p1_species,
                p2_species,
                &eye_inheritance,
                rng, // <- RNG weitergeben
            );

            // Spawne Kind Entity (kein RNG hier direkt nötig)
            let child_entity = EntityBuilder::create_entity_from_genotype(
                &mut commands,
                child_genotype,
                child_species.species,
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

/// Erstellt den Genotyp und die Spezies für ein Kind.
/// Akzeptiert jetzt einen RNG Parameter.
fn create_child_genotype<Gen: Rng + ?Sized>(
    p1_genotype: &Genotype,
    p2_genotype: &Genotype,
    _p1_species: &SpeciesGenes, // KORRIGIERT: Unused markiert
    _p2_species: &SpeciesGenes, // KORRIGIERT: Unused markiert
    eye_inheritance: &Res<EyeColorInheritance>,
    rng: &mut Gen,
) -> (Genotype, SpeciesGenes) {
    let mut child_genotype = Genotype::new();
    // Korrekte Spezies-Logik
    let mut combined_species: Vec<String> = _p1_species
        .species
        .iter()
        .chain(_p2_species.species.iter())
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

        match (p1_gene_pair_opt, p2_gene_pair_opt) {
            (Some(p1_gene_pair), Some(p2_gene_pair)) => {
                let mut child_maternal_allele: GeneVariant; // Deklariere hier
                let mut child_paternal_allele: GeneVariant; // Deklariere hier

                let eye_color_gene_id = GeneType::Visual(VisualGene::EyeColor).to_string();
                if gene_id == eye_color_gene_id {
                    // Augenfarbe: Wähle je ein Allel von jedem Elternteil
                    let p1_chosen_value = if rng.gen::<bool>() {
                        // <- Nutze RNG
                        p1_gene_pair.maternal.value
                    } else {
                        p1_gene_pair.paternal.value
                    };
                    let p2_chosen_value = if rng.gen::<bool>() {
                        // <- Nutze RNG
                        p2_gene_pair.maternal.value
                    } else {
                        p2_gene_pair.paternal.value
                    };
                    let p1_chosen_eye = EyeColor::from_f32(p1_chosen_value);
                    let p2_chosen_eye = EyeColor::from_f32(p2_chosen_value);

                    // Vererbe die Augenfarbe mit der Vererbungsmatrix und dem RNG
                    let child_resulting_eye = eye_inheritance.inherit_eye_color(
                        primary_species,
                        p1_chosen_eye,
                        p2_chosen_eye,
                        rng, // <- RNG weitergeben
                    );
                    let child_resulting_value = child_resulting_eye.to_f32();

                    // Setze beide Allele des Kindes auf das Ergebnis (da Augenfarbe komplexer ist)
                    // TODO: Überdenken, ob hier wirklich beide gleich sein sollen oder
                    // ob man das Vererbungsmodell anpasst, um zwei Allele zu liefern.
                    // Aktuell simuliert dies eine direkte Phänotyp-Bestimmung für das Kind-Allel.
                    child_maternal_allele = GeneVariant {
                        gene_id: gene_id.clone(),
                        value: child_resulting_value,
                        expression: GeneExpression::Codominant, // Expression vom Parent? Oder fix?
                    };
                    child_paternal_allele = GeneVariant {
                        gene_id: gene_id.clone(),
                        value: child_resulting_value,
                        expression: GeneExpression::Codominant, // Expression vom Parent? Oder fix?
                    };
                } else {
                    // Standard-Vererbung: Wähle zufällig ein Allel von jedem Elternteil
                    child_maternal_allele = if rng.gen::<bool>() {
                        // <- Nutze RNG
                        p1_gene_pair.maternal.clone()
                    } else {
                        p1_gene_pair.paternal.clone()
                    };
                    child_paternal_allele = if rng.gen::<bool>() {
                        // <- Nutze RNG
                        p2_gene_pair.maternal.clone()
                    } else {
                        p2_gene_pair.paternal.clone()
                    };

                    // --- TODO 3 hier implementieren für Haut/Haarfarbe ---
                    // Statt obiger Standard Vererbung:
                    // if gene_id matches SkinColorR/G/B or HairColorR/G/B:
                    //    color1 = get_color_from_parent(p1_genotype, VisualGene::SkinColorR/G/B, rng) // Holt R,G,B für die Farbe von P1
                    //    color2 = get_color_from_parent(p2_genotype, VisualGene::SkinColorR/G/B, rng) // Holt R,G,B für die Farbe von P2
                    //    mixed_color = mix_colors(color1, color2)
                    //    child_maternal_allele.value = mixed_color.R (für R Gen) etc.
                    //    child_paternal_allele.value = mixed_color.R (für R Gen) etc.
                    // ----------------------------------------------------
                }

                // TODO 5 Mutation: Hier nach der Auswahl und *vor* dem Einfügen mutieren
                let mutation_chance = 0.01; // 1% Chance pro Allel
                if rng.gen::<f32>() < mutation_chance {
                    let change = rng.gen_range(-0.05..=0.05);
                    child_maternal_allele.value =
                        (child_maternal_allele.value + change).clamp(0.0, 1.0);
                    // Optional: Log mutation
                }
                if rng.gen::<f32>() < mutation_chance {
                    let change = rng.gen_range(-0.05..=0.05);
                    child_paternal_allele.value =
                        (child_paternal_allele.value + change).clamp(0.0, 1.0);
                    // Optional: Log mutation
                }

                let child_gene_pair = GenePair {
                    maternal: child_maternal_allele, // Jetzt verwenden
                    paternal: child_paternal_allele, // Jetzt verwenden
                    chromosome_type: p1_gene_pair.chromosome_type,
                };
                child_genotype
                    .gene_pairs
                    .insert(gene_id.clone(), child_gene_pair);
                child_genotype
                    .chromosome_groups
                    .entry(p1_gene_pair.chromosome_type)
                    .or_default()
                    .push(gene_id); // Füge zum Chromosom hinzu
            }
            (Some(p1_gene_pair), None) => {
                // Nur Parent 1 hat Gen - Logik TODO 6
                warn!("TODO 6: Gen '{}' nur in P1. Kind erhält Kopie.", gene_id);
                let allele1 = if rng.gen::<bool>() {
                    p1_gene_pair.maternal.clone()
                } else {
                    p1_gene_pair.paternal.clone()
                };
                let allele2 = if rng.gen::<bool>() {
                    p1_gene_pair.maternal.clone()
                } else {
                    p1_gene_pair.paternal.clone()
                }; // Nochmals von P1

                // Hier könnte Mutation auch angewendet werden
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
                // Nur Parent 2 hat Gen - Logik TODO 6
                warn!("TODO 6: Gen '{}' nur in P2. Kind erhält Kopie.", gene_id);
                let allele1 = if rng.gen::<bool>() {
                    p2_gene_pair.maternal.clone()
                } else {
                    p2_gene_pair.paternal.clone()
                };
                let allele2 = if rng.gen::<bool>() {
                    p2_gene_pair.maternal.clone()
                } else {
                    p2_gene_pair.paternal.clone()
                };

                // Hier könnte Mutation auch angewendet werden
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
            (None, None) => { /* Sollte nicht vorkommen */ }
        }
    }

    // Stelle sicher, dass jede Chromosomengruppe im child_genotype unique ist
    for gene_list in child_genotype.chromosome_groups.values_mut() {
        gene_list.sort();
        gene_list.dedup();
    }

    (child_genotype, child_species)
}
