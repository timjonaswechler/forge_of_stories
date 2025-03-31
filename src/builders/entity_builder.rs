// src/builders/entity_builder.rs
use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::attributes::{MentalAttributes, PhysicalAttributes, SocialAttributes};
use crate::components::genetics::{
    BodyStructure, ChromosomeType, Fertility, GeneExpression, Genotype, Parent, Personality,
    Phenotype, SpeciesGenes, VisualTraits,
};
use crate::components::phenotype_gene::PhenotypeGene;
use crate::resources::gene_library::GeneLibrary;

/// Builder für genetisch definierte Entitäten
pub struct EntityBuilder;

impl EntityBuilder {
    /// Erstellt eine vollständige Entität basierend auf einem Genotyp
    pub fn create_entity_from_genotype(
        commands: &mut Commands,
        genotype: Genotype,
        species_names: Vec<String>,
        gene_library: &Res<GeneLibrary>,
    ) -> Entity {
        // Berechne Phänotyp aus Genotyp
        let mut phenotype = Phenotype::new();
        Self::calculate_phenotype(&genotype, &mut phenotype);

        // Berechne die visuellen Eigenschaften aus dem Genotyp
        let visual_traits =
            Self::calculate_visual_traits(&genotype, &phenotype, &species_names, gene_library);

        // Spawn Entity mit allen Komponenten
        commands
            .spawn((
                genotype.clone(),
                phenotype.clone(),
                Self::calculate_physical_attributes(&phenotype),
                Self::calculate_mental_attributes(&phenotype),
                Self::calculate_social_attributes(&phenotype),
                visual_traits,
                SpeciesGenes {
                    species: species_names,
                },
                Self::calculate_body_structure(&phenotype),
                Self::calculate_personality(&phenotype),
                Parent { children: vec![] },
                Fertility {
                    fertility_rate: 0.5,
                    reproduction_cooldown: None,
                    compatibility_modifiers: HashMap::new(),
                    maturity: true,
                },
            ))
            .id()
    }

    /// Berechnet den Phänotyp aus dem Genotyp
    fn calculate_phenotype(genotype: &Genotype, phenotype: &mut Phenotype) {
        // Leere die Phänotyp-Gruppen
        phenotype.attribute_groups.clear();

        for (gene_id, gene_pair) in genotype.gene_pairs.iter() {
            // Bestimme den phänotypischen Wert basierend auf den Expressionen
            let value = match (gene_pair.maternal.expression, gene_pair.paternal.expression) {
                // Wenn beide dominant sind oder beide rezessiv, nimm den Durchschnitt
                (GeneExpression::Dominant, GeneExpression::Dominant)
                | (GeneExpression::Recessive, GeneExpression::Recessive) => {
                    (gene_pair.maternal.value + gene_pair.paternal.value) / 2.0
                }

                // Wenn eins dominant und eins rezessiv ist, nimm das dominante
                (GeneExpression::Dominant, GeneExpression::Recessive) => gene_pair.maternal.value,
                (GeneExpression::Recessive, GeneExpression::Dominant) => gene_pair.paternal.value,

                // Bei Kodominanz: gewichteter Durchschnitt
                (GeneExpression::Codominant, _) | (_, GeneExpression::Codominant) => {
                    (gene_pair.maternal.value + gene_pair.paternal.value) / 2.0
                }
            };

            // Bestimme die Expression für das Phänotyp-Gen
            // Verwende die dominantere Expression der beiden Allele
            let expression = if gene_pair.maternal.expression == GeneExpression::Dominant
                || gene_pair.paternal.expression == GeneExpression::Dominant
            {
                GeneExpression::Dominant
            } else if gene_pair.maternal.expression == GeneExpression::Codominant
                || gene_pair.paternal.expression == GeneExpression::Codominant
            {
                GeneExpression::Codominant
            } else {
                GeneExpression::Recessive
            };

            // Erstelle ein PhenotypeGene mit Wert und Expression
            let phenotype_gene = PhenotypeGene::new(value, expression);

            // Speichere den Wert im allgemeinen Phänotyp
            phenotype.attributes.insert(gene_id.clone(), phenotype_gene);

            // Speichere den Wert auch in der entsprechenden Chromosomen-Gruppe
            phenotype
                .attribute_groups
                .entry(gene_pair.chromosome_type)
                .or_insert_with(HashMap::new)
                .insert(gene_id.clone(), phenotype_gene);
        }
    }

    /// Berechnet die physischen Attribute aus dem Phänotyp
    fn calculate_physical_attributes(phenotype: &Phenotype) -> PhysicalAttributes {
        let mut physical_attrs = PhysicalAttributes::default();

        // Holen der Attributwerte aus der Attribut-Chromosomen-Gruppe
        if let Some(attribute_values) = phenotype.attribute_groups.get(&ChromosomeType::Attributes)
        {
            // Beispiel für Anwendung genetischer Werte auf Attribute
            if let Some(strength_value) = attribute_values.get("gene_strength") {
                physical_attrs.strength.base_value = strength_value.value() * 100.0;
                physical_attrs.strength.current_value = physical_attrs.strength.base_value;
            }

            if let Some(agility_value) = attribute_values.get("gene_agility") {
                physical_attrs.agility.base_value = agility_value.value() * 100.0;
                physical_attrs.agility.current_value = physical_attrs.agility.base_value;
            }

            if let Some(toughness_value) = attribute_values.get("gene_toughness") {
                physical_attrs.toughness.base_value = toughness_value.value() * 100.0;
                physical_attrs.toughness.current_value = physical_attrs.toughness.base_value;
            }

            if let Some(endurance_value) = attribute_values.get("gene_endurance") {
                physical_attrs.endurance.base_value = endurance_value.value() * 100.0;
                physical_attrs.endurance.current_value = physical_attrs.endurance.base_value;
            }

            if let Some(recuperation_value) = attribute_values.get("gene_recuperation") {
                physical_attrs.recuperation.base_value = recuperation_value.value() * 100.0;
                physical_attrs.recuperation.current_value = physical_attrs.recuperation.base_value;
            }

            if let Some(disease_resistance_value) = attribute_values.get("gene_disease_resistance")
            {
                physical_attrs.disease_resistance.base_value =
                    disease_resistance_value.value() * 100.0;
                physical_attrs.disease_resistance.current_value =
                    physical_attrs.disease_resistance.base_value;
            }
        }

        physical_attrs
    }

    /// Berechnet die mentalen Attribute aus dem Phänotyp
    fn calculate_mental_attributes(phenotype: &Phenotype) -> MentalAttributes {
        let mut mental_attrs = MentalAttributes::default();

        // Holen der Attributwerte aus der Attribut-Chromosomen-Gruppe
        if let Some(attribute_values) = phenotype.attribute_groups.get(&ChromosomeType::Attributes)
        {
            // Anwendung genetischer Werte auf Attribute
            if let Some(focus_value) = attribute_values.get("gene_focus") {
                mental_attrs.focus.base_value = focus_value.value() * 100.0;
                mental_attrs.focus.current_value = mental_attrs.focus.base_value;
            }

            if let Some(creativity_value) = attribute_values.get("gene_creativity") {
                mental_attrs.creativity.base_value = creativity_value.value() * 100.0;
                mental_attrs.creativity.current_value = mental_attrs.creativity.base_value;
            }

            if let Some(willpower_value) = attribute_values.get("gene_willpower") {
                mental_attrs.willpower.base_value = willpower_value.value() * 100.0;
                mental_attrs.willpower.current_value = mental_attrs.willpower.base_value;
            }

            if let Some(analytical_ability_value) = attribute_values.get("gene_analytical_ability")
            {
                mental_attrs.analytical_ability.base_value =
                    analytical_ability_value.value() * 100.0;
                mental_attrs.analytical_ability.current_value =
                    mental_attrs.analytical_ability.base_value;
            }

            if let Some(intuition_value) = attribute_values.get("gene_intuition") {
                mental_attrs.intuition.base_value = intuition_value.value() * 100.0;
                mental_attrs.intuition.current_value = mental_attrs.intuition.base_value;
            }

            if let Some(memory_value) = attribute_values.get("gene_memory") {
                mental_attrs.memory.base_value = memory_value.value() * 100.0;
                mental_attrs.memory.current_value = mental_attrs.memory.base_value;
            }
        }

        mental_attrs
    }

    /// Berechnet die sozialen Attribute aus dem Phänotyp
    fn calculate_social_attributes(phenotype: &Phenotype) -> SocialAttributes {
        let mut social_attrs = SocialAttributes::default();

        // Holen der Attributwerte aus der Attribut-Chromosomen-Gruppe
        if let Some(attribute_values) = phenotype.attribute_groups.get(&ChromosomeType::Attributes)
        {
            // Anwendung genetischer Werte auf Attribute
            if let Some(empathy_value) = attribute_values.get("gene_empathy") {
                social_attrs.empathy.base_value = empathy_value.value() * 100.0;
                social_attrs.empathy.current_value = social_attrs.empathy.base_value;
            }

            if let Some(leadership_value) = attribute_values.get("gene_leadership") {
                social_attrs.leadership.base_value = leadership_value.value() * 100.0;
                social_attrs.leadership.current_value = social_attrs.leadership.base_value;
            }

            if let Some(social_awareness_value) = attribute_values.get("gene_social_awareness") {
                social_attrs.social_awareness.base_value = social_awareness_value.value() * 100.0;
                social_attrs.social_awareness.current_value =
                    social_attrs.social_awareness.base_value;
            }

            if let Some(linguistic_ability_value) = attribute_values.get("gene_linguistic_ability")
            {
                social_attrs.linguistic_ability.base_value =
                    linguistic_ability_value.value() * 100.0;
                social_attrs.linguistic_ability.current_value =
                    social_attrs.linguistic_ability.base_value;
            }

            if let Some(negotiation_value) = attribute_values.get("gene_negotiation") {
                social_attrs.negotiation.base_value = negotiation_value.value() * 100.0;
                social_attrs.negotiation.current_value = social_attrs.negotiation.base_value;
            }
        }

        social_attrs
    }

    /// Berechnet die visuellen Eigenschaften aus dem Genotyp und Phänotyp
    fn calculate_visual_traits(
        genotype: &Genotype,
        phenotype: &Phenotype,
        species_names: &[String],
        gene_library: &Res<GeneLibrary>,
    ) -> VisualTraits {
        let mut visual_traits = VisualTraits {
            skin_color: (0.5, 0.5, 0.5), // Neutrale Defaults
            hair_color: (0.5, 0.5, 0.5),
            eye_color: (0.5, 0.5, 0.5),
        };

        // Hautfarbe direkt aus dem Genotyp extrahieren
        if let (Some(r_pair), Some(g_pair), Some(b_pair)) = (
            genotype.gene_pairs.get("gene_skin_r"),
            genotype.gene_pairs.get("gene_skin_g"),
            genotype.gene_pairs.get("gene_skin_b"),
        ) {
            // Einfache Berechnung für die Demo - später kann hier die vollständige Logik
            // aus update_visual_traits_system implementiert werden
            let skin_r = (r_pair.maternal.value + r_pair.paternal.value) / 2.0;
            let skin_g = (g_pair.maternal.value + g_pair.paternal.value) / 2.0;
            let skin_b = (b_pair.maternal.value + b_pair.paternal.value) / 2.0;

            visual_traits.skin_color = (skin_r, skin_g, skin_b);
        } else if !species_names.is_empty() && phenotype.attributes.contains_key("gene_skin_tone") {
            // Alternativ aus dem skin_tone und der Spezies
            let skin_tone = phenotype.attributes["gene_skin_tone"].value();
            let primary_species = &species_names[0];

            if let Some(colors) = gene_library.skin_colors.get(primary_species) {
                if !colors.is_empty() {
                    let color_index =
                        ((skin_tone * (colors.len() as f32 - 1.0)) as usize).min(colors.len() - 1);
                    visual_traits.skin_color = colors[color_index];
                }
            }
        }

        // Haarfarbe
        if let (Some(r_pair), Some(g_pair), Some(b_pair)) = (
            genotype.gene_pairs.get("gene_hair_r"),
            genotype.gene_pairs.get("gene_hair_g"),
            genotype.gene_pairs.get("gene_hair_b"),
        ) {
            let hair_r = (r_pair.maternal.value + r_pair.paternal.value) / 2.0;
            let hair_g = (g_pair.maternal.value + g_pair.paternal.value) / 2.0;
            let hair_b = (b_pair.maternal.value + b_pair.paternal.value) / 2.0;

            visual_traits.hair_color = (hair_r, hair_g, hair_b);
        }

        // Augenfarbe
        if let (Some(r_pair), Some(g_pair), Some(b_pair)) = (
            genotype.gene_pairs.get("gene_eye_r"),
            genotype.gene_pairs.get("gene_eye_g"),
            genotype.gene_pairs.get("gene_eye_b"),
        ) {
            let eye_r = (r_pair.maternal.value + r_pair.paternal.value) / 2.0;
            let eye_g = (g_pair.maternal.value + g_pair.paternal.value) / 2.0;
            let eye_b = (b_pair.maternal.value + b_pair.paternal.value) / 2.0;

            visual_traits.eye_color = (eye_r, eye_g, eye_b);
        }

        visual_traits
    }

    /// Berechnet die Persönlichkeit aus dem Phänotyp
    fn calculate_personality(phenotype: &Phenotype) -> Personality {
        let mut personality = Personality::new();

        // Holen der Persönlichkeitswerte aus der Persönlichkeits-Chromosomen-Gruppe
        if let Some(personality_values) =
            phenotype.attribute_groups.get(&ChromosomeType::Personality)
        {
            for (trait_id, gene_value) in personality_values.iter() {
                // Strip off the "gene_" prefix for cleaner trait names
                let trait_name = trait_id.strip_prefix("gene_").unwrap_or(trait_id);
                personality
                    .traits
                    .insert(trait_name.to_string(), gene_value.value());
            }
        } else {
            // Fallback auf Standardpersönlichkeit
            personality = Personality::default_traits();
        }

        personality
    }

    /// Berechnet die Körperstruktur aus dem Phänotyp
    fn calculate_body_structure(phenotype: &Phenotype) -> BodyStructure {
        let mut body_structure = BodyStructure::humanoid();

        // Wenn wir Body-Struktur-Gene haben, wenden wir sie an
        if let Some(body_values) = phenotype
            .attribute_groups
            .get(&ChromosomeType::BodyStructure)
        {
            // Rekursive Hilfsfunktion zum Aktualisieren von Körperteilen basierend auf Genen
            fn update_body_part(
                body_part: &mut crate::components::genetics::BodyComponent,
                body_values: &HashMap<String, f32>,
            ) {
                // Aktualisiere Eigenschaften für dieses Körperteil
                let gene_prefix = format!("gene_body_{}_", body_part.id);

                for (gene_id, value) in body_values.iter() {
                    if gene_id.starts_with(&gene_prefix) {
                        let property_name = gene_id.strip_prefix(&gene_prefix).unwrap_or(gene_id);
                        body_part
                            .properties
                            .insert(property_name.to_string(), *value);
                    }
                }

                // Rekursiv für alle Kinder
                for child in &mut body_part.children {
                    update_body_part(child, body_values);
                }
            }

            // Konvertiere body_values zu HashMap<String, f32>
            let body_values_f32: HashMap<String, f32> = body_values
                .iter()
                .map(|(k, v)| (k.clone(), v.value()))
                .collect();

            // Starte mit der Wurzelkomponente
            update_body_part(&mut body_structure.root, &body_values_f32);
        }

        body_structure
    }
}
