// src/systems/genetics.rs
use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::attributes::{MentalAttributes, PhysicalAttributes, SocialAttributes};
use crate::components::genetics::{
    ChromosomeType, GeneExpression, Genotype, Phenotype, PhenotypeGene, SpeciesGenes,
};
use crate::components::visual_traits::EyeColor;
use crate::resources::eye_color_inheritance::EyeColorInheritance;

// System zur Berechnung des Phänotyps aus dem Genotyp
pub fn genotype_to_phenotype_system(
    mut query: Query<(&Genotype, &mut Phenotype, &SpeciesGenes)>,
    eye_inheritance: Res<EyeColorInheritance>,
) {
    for (genotype, mut phenotype, species_genes) in query.iter_mut() {
        // Leere die Phänotyp-Gruppen
        phenotype.attribute_groups.clear();
        phenotype.attributes.clear();

        // Bestimme die primäre Spezies (verwende die erste, falls vorhanden)
        let primary_species = species_genes
            .species
            .first()
            .map(|s| s.as_str())
            .unwrap_or("Mensch"); // Fallback auf Mensch

        for (gene_id, gene_pair) in genotype.gene_pairs.iter() {
            // Spezielle Verarbeitung für Augenfarben
            if gene_id == "gene_eye_color" {
                let maternal_eye_color = EyeColor::from_f32(gene_pair.maternal.value);
                let paternal_eye_color = EyeColor::from_f32(gene_pair.paternal.value);

                // Verwende die Spezies-spezifische Vererbungslogik
                let resulting_eye_color = if maternal_eye_color == paternal_eye_color {
                    maternal_eye_color
                } else {
                    eye_inheritance.inherit_eye_color(
                        primary_species,
                        maternal_eye_color,
                        paternal_eye_color,
                    )
                };

                let phenotype_gene = PhenotypeGene {
                    value: resulting_eye_color.to_f32(),
                    expression: GeneExpression::Codominant,
                };

                phenotype.attributes.insert(gene_id.clone(), phenotype_gene);

                phenotype
                    .attribute_groups
                    .entry(gene_pair.chromosome_type)
                    .or_insert_with(|| HashMap::new())
                    .insert(gene_id.clone(), phenotype_gene);

                continue; // Skip standard processing for this gene
            }

            // Standardverarbeitung für alle anderen Gene
            let (value, expression) =
                match (gene_pair.maternal.expression, gene_pair.paternal.expression) {
                    // ... bestehender Code ...
                    // Wenn beide dominant sind oder beide rezessiv, nimm den Durchschnitt und behalte die Expression
                    (GeneExpression::Dominant, GeneExpression::Dominant) => (
                        (gene_pair.maternal.value + gene_pair.paternal.value) / 2.0,
                        GeneExpression::Dominant,
                    ),
                    (GeneExpression::Recessive, GeneExpression::Recessive) => (
                        (gene_pair.maternal.value + gene_pair.paternal.value) / 2.0,
                        GeneExpression::Recessive,
                    ),

                    // Wenn eins dominant und eins rezessiv ist, nimm das dominante
                    (GeneExpression::Dominant, GeneExpression::Recessive) => {
                        (gene_pair.maternal.value, GeneExpression::Dominant)
                    }
                    (GeneExpression::Recessive, GeneExpression::Dominant) => {
                        (gene_pair.paternal.value, GeneExpression::Dominant)
                    }

                    // Bei Kodominanz: gewichteter Durchschnitt und Codominante Expression
                    (GeneExpression::Codominant, _) | (_, GeneExpression::Codominant) => (
                        (gene_pair.maternal.value + gene_pair.paternal.value) / 2.0,
                        GeneExpression::Codominant,
                    ),
                };

            // Phänotyp-Gen erstellen
            let phenotype_gene = PhenotypeGene { value, expression };

            // Speichere den Wert im allgemeinen Phänotyp
            phenotype
                .attributes
                .insert(gene_id.clone(), phenotype_gene.clone());

            // Speichere den Wert auch in der entsprechenden Chromosomen-Gruppe
            phenotype
                .attribute_groups
                .entry(gene_pair.chromosome_type)
                .or_insert_with(|| HashMap::new()) // Korrigiert mit einem Closure
                .insert(gene_id.clone(), phenotype_gene);
        }
    }
}
// System zur Anwendung des Phänotyps auf die physischen Attribute
pub fn apply_physical_attributes_system(mut query: Query<(&Phenotype, &mut PhysicalAttributes)>) {
    for (phenotype, mut physical_attrs) in query.iter_mut() {
        // Holen der Attributwerte aus der Attribut-Chromosomen-Gruppe
        if let Some(attribute_values) = phenotype.attribute_groups.get(&ChromosomeType::Attributes)
        {
            // Beispiel für die Anwendung genetischer Werte auf Attribute
            if let Some(strength_gene) = attribute_values.get("gene_strength") {
                physical_attrs.strength.base_value = strength_gene.value * 100.0;
                physical_attrs.strength.current_value = physical_attrs.strength.base_value;
                // Hier könnte man auch mit der Expression arbeiten, wenn nötig
            }

            if let Some(agility_gene) = attribute_values.get("gene_agility") {
                physical_attrs.agility.base_value = agility_gene.value * 100.0;
                physical_attrs.agility.current_value = physical_attrs.agility.base_value;
            }

            if let Some(toughness_gene) = attribute_values.get("gene_toughness") {
                physical_attrs.toughness.base_value = toughness_gene.value * 100.0;
                physical_attrs.toughness.current_value = physical_attrs.toughness.base_value;
            }

            if let Some(endurance_gene) = attribute_values.get("gene_endurance") {
                physical_attrs.endurance.base_value = endurance_gene.value * 100.0;
                physical_attrs.endurance.current_value = physical_attrs.endurance.base_value;
            }

            if let Some(recuperation_gene) = attribute_values.get("gene_recuperation") {
                physical_attrs.recuperation.base_value = recuperation_gene.value * 100.0;
                physical_attrs.recuperation.current_value = physical_attrs.recuperation.base_value;
            }

            if let Some(disease_resistance_gene) = attribute_values.get("gene_disease_resistance") {
                physical_attrs.disease_resistance.base_value =
                    disease_resistance_gene.value * 100.0;
                physical_attrs.disease_resistance.current_value =
                    physical_attrs.disease_resistance.base_value;
            }
        }
    }
}

// System zur Anwendung des Phänotyps auf die mentalen Attribute
pub fn apply_mental_attributes_system(mut query: Query<(&Phenotype, &mut MentalAttributes)>) {
    for (phenotype, mut mental_attrs) in query.iter_mut() {
        // Holen der Attributwerte aus der Attribut-Chromosomen-Gruppe
        if let Some(attribute_values) = phenotype.attribute_groups.get(&ChromosomeType::Attributes)
        {
            // Beispiel für die Anwendung genetischer Werte auf Attribute
            if let Some(focus_gene) = attribute_values.get("gene_focus") {
                mental_attrs.focus.base_value = focus_gene.value * 100.0;
                mental_attrs.focus.current_value = mental_attrs.focus.base_value;
            }

            if let Some(creativity_gene) = attribute_values.get("gene_creativity") {
                mental_attrs.creativity.base_value = creativity_gene.value * 100.0;
                mental_attrs.creativity.current_value = mental_attrs.creativity.base_value;
            }

            if let Some(willpower_gene) = attribute_values.get("gene_willpower") {
                mental_attrs.willpower.base_value = willpower_gene.value * 100.0;
                mental_attrs.willpower.current_value = mental_attrs.willpower.base_value;
            }

            if let Some(analytical_ability_gene) = attribute_values.get("gene_analytical_ability") {
                mental_attrs.analytical_ability.base_value = analytical_ability_gene.value * 100.0;
                mental_attrs.analytical_ability.current_value =
                    mental_attrs.analytical_ability.base_value;
            }

            if let Some(intuition_gene) = attribute_values.get("gene_intuition") {
                mental_attrs.intuition.base_value = intuition_gene.value * 100.0;
                mental_attrs.intuition.current_value = mental_attrs.intuition.base_value;
            }

            if let Some(memory_gene) = attribute_values.get("gene_memory") {
                mental_attrs.memory.base_value = memory_gene.value * 100.0;
                mental_attrs.memory.current_value = mental_attrs.memory.base_value;
            }
        }
    }
}

// System zur Anwendung des Phänotyps auf die sozialen Attribute
pub fn apply_social_attributes_system(mut query: Query<(&Phenotype, &mut SocialAttributes)>) {
    for (phenotype, mut social_attrs) in query.iter_mut() {
        // Holen der Attributwerte aus der Attribut-Chromosomen-Gruppe
        if let Some(attribute_values) = phenotype.attribute_groups.get(&ChromosomeType::Attributes)
        {
            // Beispiel für die Anwendung genetischer Werte auf Attribute
            if let Some(empathy_gene) = attribute_values.get("gene_empathy") {
                social_attrs.empathy.base_value = empathy_gene.value * 100.0;
                social_attrs.empathy.current_value = social_attrs.empathy.base_value;
            }

            if let Some(leadership_gene) = attribute_values.get("gene_leadership") {
                social_attrs.leadership.base_value = leadership_gene.value * 100.0;
                social_attrs.leadership.current_value = social_attrs.leadership.base_value;
            }

            if let Some(social_awareness_gene) = attribute_values.get("gene_social_awareness") {
                social_attrs.social_awareness.base_value = social_awareness_gene.value * 100.0;
                social_attrs.social_awareness.current_value =
                    social_attrs.social_awareness.base_value;
            }

            if let Some(linguistic_ability_gene) = attribute_values.get("gene_linguistic_ability") {
                social_attrs.linguistic_ability.base_value = linguistic_ability_gene.value * 100.0;
                social_attrs.linguistic_ability.current_value =
                    social_attrs.linguistic_ability.base_value;
            }

            if let Some(negotiation_gene) = attribute_values.get("gene_negotiation") {
                social_attrs.negotiation.base_value = negotiation_gene.value * 100.0;
                social_attrs.negotiation.current_value = social_attrs.negotiation.base_value;
            }
        }
    }
}
