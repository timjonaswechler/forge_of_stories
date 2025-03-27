use bevy::prelude::*;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

use crate::components::attributes::{MentalAttributes, PhysicalAttributes, SocialAttributes};
use crate::components::genetics::{
    Allele, ChromosomePair, Fertility, GeneExpression, Genotype, Parent, Phenotype,
    SpeciesIdentity, VisualTraits,
};
use crate::resources::skin_color_palette::SkinColorPalette;

// System zur Berechnung des Phänotyps aus dem Genotyp
pub fn genotype_to_phenotype_system(mut query: Query<(&Genotype, &mut Phenotype)>) {
    for (genotype, mut phenotype) in query.iter_mut() {
        for (gene_id, chromosome_pair) in genotype.chromosome_pairs.iter() {
            // Bestimme den phänotypischen Wert basierend auf den Expressionen
            let value = match (
                chromosome_pair.maternal.expression,
                chromosome_pair.paternal.expression,
            ) {
                // Wenn beide dominant sind oder beide rezessiv, nimm den Durchschnitt
                (GeneExpression::Dominant, GeneExpression::Dominant)
                | (GeneExpression::Recessive, GeneExpression::Recessive) => {
                    (chromosome_pair.maternal.value + chromosome_pair.paternal.value) / 2.0
                }

                // Wenn eins dominant und eins rezessiv ist, nimm das dominante
                (GeneExpression::Dominant, GeneExpression::Recessive) => {
                    chromosome_pair.maternal.value
                }
                (GeneExpression::Recessive, GeneExpression::Dominant) => {
                    chromosome_pair.paternal.value
                }

                // Bei Kodominanz: gewichteter Durchschnitt
                (GeneExpression::Codominant, _) | (_, GeneExpression::Codominant) => {
                    (chromosome_pair.maternal.value + chromosome_pair.paternal.value) / 2.0
                }
            };

            phenotype.attributes.insert(gene_id.clone(), value);
        }
    }
}

// System zur Anwendung des Phänotyps auf die physischen Attribute
pub fn apply_physical_attributes_system(mut query: Query<(&Phenotype, &mut PhysicalAttributes)>) {
    for (phenotype, mut physical_attrs) in query.iter_mut() {
        // Beispiel für die Anwendung genetischer Werte auf Attribute
        if let Some(strength_value) = phenotype.attributes.get("gene_strength") {
            physical_attrs.strength.base_value = strength_value * 100.0;
            physical_attrs.strength.current_value = physical_attrs.strength.base_value;
        }

        if let Some(agility_value) = phenotype.attributes.get("gene_agility") {
            physical_attrs.agility.base_value = agility_value * 100.0;
            physical_attrs.agility.current_value = physical_attrs.agility.base_value;
        }

        // Weitere Attribute ähnlich umsetzen...
    }
}

// System zur Anwendung des Phänotyps auf die mentalen Attribute
pub fn apply_mental_attributes_system(mut query: Query<(&Phenotype, &mut MentalAttributes)>) {
    for (phenotype, mut mental_attrs) in query.iter_mut() {
        // Beispiel für die Anwendung genetischer Werte auf Attribute
        if let Some(focus_value) = phenotype.attributes.get("gene_focus") {
            mental_attrs.focus.base_value = focus_value * 100.0;
            mental_attrs.focus.current_value = mental_attrs.focus.base_value;
        }

        if let Some(creativity_value) = phenotype.attributes.get("gene_creativity") {
            mental_attrs.creativity.base_value = creativity_value * 100.0;
            mental_attrs.creativity.current_value = mental_attrs.creativity.base_value;
        }

        // Weitere Attribute ähnlich umsetzen...
    }
}

// System zur Anwendung des Phänotyps auf die sozialen Attribute
pub fn apply_social_attributes_system(mut query: Query<(&Phenotype, &mut SocialAttributes)>) {
    for (phenotype, mut social_attrs) in query.iter_mut() {
        // Beispiel für die Anwendung genetischer Werte auf Attribute
        if let Some(empathy_value) = phenotype.attributes.get("gene_empathy") {
            social_attrs.empathy.base_value = empathy_value * 100.0;
            social_attrs.empathy.current_value = social_attrs.empathy.base_value;
        }

        if let Some(leadership_value) = phenotype.attributes.get("gene_leadership") {
            social_attrs.leadership.base_value = leadership_value * 100.0;
            social_attrs.leadership.current_value = social_attrs.leadership.base_value;
        }

        // Weitere Attribute ähnlich umsetzen...
    }
}

// System zur Berechnung visueller Merkmale basierend auf Genen
pub fn update_visual_traits_system(
    mut query: Query<(&Phenotype, &mut VisualTraits, &SpeciesIdentity)>,
    skin_palette: Res<SkinColorPalette>,
) {
    for (phenotype, mut visual_traits, species_identity) in query.iter_mut() {
        // Bei gemischten Spezies mischen wir die Hautfarben basierend auf Genpool
        if phenotype.attributes.contains_key("gene_skin_r")
            && phenotype.attributes.contains_key("gene_skin_g")
            && phenotype.attributes.contains_key("gene_skin_b")
        {
            // Wenn wir spezifische Gene für RGB-Komponenten haben, verwenden wir diese
            visual_traits.skin_color = (
                *phenotype.attributes.get("gene_skin_r").unwrap(),
                *phenotype.attributes.get("gene_skin_g").unwrap(),
                *phenotype.attributes.get("gene_skin_b").unwrap(),
            );
        } else {
            // Andernfalls berechnen wir die Hautfarbe basierend auf den Spezies-Anteilen
            let mut skin_color = (0.0, 0.0, 0.0);
            let mut total_weight = 0.0;

            for (species, percentage) in species_identity.species_percentage.iter() {
                if let Some(species_colors) = skin_palette.colors.get(species) {
                    if !species_colors.is_empty() {
                        // Wähle eine Hautfarbe basierend auf einem genetischen Faktor
                        // Hier verwenden wir gene_skin_tone, falls vorhanden, oder generieren einen zufälligen Wert
                        let gene_value = phenotype.attributes.get("gene_skin_tone").unwrap_or(&0.5);
                        let color_index = ((gene_value * (species_colors.len() as f32 - 1.0))
                            as usize)
                            .min(species_colors.len() - 1);

                        let color = species_colors[color_index];

                        // Mische proportional zum Spezies-Anteil
                        skin_color.0 += color.0 * percentage;
                        skin_color.1 += color.1 * percentage;
                        skin_color.2 += color.2 * percentage;
                        total_weight += percentage;
                    }
                }
            }

            // Normalisiere die Farbe falls nötig
            if total_weight > 0.0 {
                skin_color.0 /= total_weight;
                skin_color.1 /= total_weight;
                skin_color.2 /= total_weight;
            }

            visual_traits.skin_color = skin_color;
        }

        // Größe berechnen (wie vorher)
        if let Some(height_base) = phenotype.attributes.get("gene_height_base") {
            visual_traits.height = 150.0 + (height_base * 50.0);
        }

        // Weitere visuelle Merkmale könnten hier ähnlich umgesetzt werden...
    }
}

// System zur genetischen Vererbung bei der Fortpflanzung
pub fn reproduction_system(
    mut commands: Commands,
    parent_query: Query<(Entity, &Genotype, &SpeciesIdentity), With<Parent>>,
) {
    // Dies ist nur eine Dummy-Implementierung, die noch keine tatsächliche Reproduktion durchführt
    // In einem realen System würde hier die Logik zur Bestimmung, welche Entitäten sich fortpflanzen, stehen

    let parents: Vec<(Entity, &Genotype, &SpeciesIdentity)> = parent_query.iter().collect();

    // Nur fortfahren, wenn wir mindestens 2 Eltern haben
    if parents.len() >= 2 {
        // Beispielsweise nehmen wir die ersten beiden Eltern
        let (parent1_entity, parent1_genotype, parent1_species) = &parents[0];
        let (parent2_entity, parent2_genotype, parent2_species) = &parents[1];

        info!(
            "Potentielle Fortpflanzung zwischen {:?} und {:?}",
            parent1_entity, parent2_entity
        );
        info!("  Erster Elternteil: {}", parent1_species.primary_species);
        info!("  Zweiter Elternteil: {}", parent2_species.primary_species);

        // Hier würde in einer vollständigen Implementierung die Berechnung des Nachwuchs-Genotyps erfolgen
        // und ein neues Kind-Entity erstellt werden

        // Kommentieren wir den tatsächlichen Reproduktionscode aus, bis wir ihn implementieren möchten
        /*
        // Implementierung würde hier folgen...
         */
    } else {
        info!(
            "Nicht genügend Elternteile für Reproduktion vorhanden: {}",
            parents.len()
        );
    }
}

// System zur Berechnung der Fruchtbarkeit basierend auf genetischer Kompatibilität
pub fn calculate_fertility_system(mut query: Query<(&SpeciesIdentity, &mut Fertility)>) {
    for (species_identity, mut fertility) in query.iter_mut() {
        // Grundsätzliche Fruchtbarkeitsrate basierend auf Spezies
        let base_fertility = match species_identity.primary_species.as_str() {
            "Mensch" => 0.8,
            "Elf" => 0.5, // Elfen haben eine niedrigere Geburtenrate
            "Zwerg" => 0.6,
            "Ork" => 0.9, // Orks haben eine hohe Geburtenrate
            _ => 0.7,
        };

        // Berücksichtige genetische Mischung
        let mut mixed_penalty = 0.0;

        // Je mehr verschiedene Spezies im Genpool, desto größer könnte die Einschränkung sein
        let diverse_species_count = species_identity.species_percentage.len();

        if diverse_species_count > 1 {
            mixed_penalty = (diverse_species_count as f32 - 1.0) * 0.05;
        }

        // Endgültige Fruchtbarkeitsrate berechnen
        fertility.fertility_rate = (base_fertility - mixed_penalty).max(0.1);

        // Kompatibilitätsmodifikatoren für andere Spezies
        // (vereinfachtes Beispiel)
        for (species, percentage) in species_identity.species_percentage.iter() {
            fertility.compatibility_modifiers.insert(
                species.clone(),
                1.0 - (1.0 - percentage) * 0.5, // Höhere Kompatibilität mit ähnlichen Spezies
            );
        }
    }
}
