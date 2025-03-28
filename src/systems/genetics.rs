// src/systems/genetics.rs
use bevy::prelude::*;
use rand::Rng;
use std::collections::{HashMap, HashSet};

use crate::components::attributes::{MentalAttributes, PhysicalAttributes, SocialAttributes};
use crate::components::genetics::{
    Ancestry, BodyComponent, BodyStructure, ChromosomeType, Fertility, GeneExpression, Genotype,
    Parent, Personality, Phenotype, SpeciesGenes, VisualTraits,
};
use crate::resources::skin_color_palette::SkinColorPalette;

// Fortpflanzungssystem zur Kombination genetischer Information zweier Elternteile
pub fn reproduction_system(
    mut commands: Commands,
    _time: Res<Time>,
    // Query für fortpflanzungsfähige Entitäten
    mut query: Query<(Entity, &Genotype, &mut Fertility, &SpeciesGenes)>,
    mut parent_query: Query<&mut Parent>,
) {
    // Sammeln potentieller Elternteile
    let mut potential_parents = Vec::new();

    // Sammle fortpflanzungsfähige Entitäten
    for (entity, genotype, mut fertility, species_genes) in query.iter_mut() {
        // Überprüfen, ob das Wesen fortpflanzungsfähig ist
        if fertility.maturity && fertility.reproduction_cooldown.is_none() {
            potential_parents.push((entity, genotype.clone(), species_genes.clone()));
            // Setze sofort Abklingzeit
            fertility.reproduction_cooldown = Some(30.0);
        }
    }

    // Prüfe, ob wir genügend Eltern für Fortpflanzung haben
    if potential_parents.len() < 2 {
        return;
    }

    let mut rng = rand::thread_rng();

    // Wähle Eltern aus
    let parent1_idx = rng.gen_range(0..potential_parents.len());
    let (parent1_entity, parent1_genotype, parent1_species) = potential_parents.remove(parent1_idx);

    let parent2_idx = rng.gen_range(0..potential_parents.len());
    let (parent2_entity, parent2_genotype, parent2_species) = &potential_parents[parent2_idx];

    // Erstelle das Kind (neuer Organismus)
    let child_entity = create_child(
        &mut commands,
        parent1_entity,
        *parent2_entity,
        &parent1_genotype,
        parent2_genotype,
        &parent1_species,
        parent2_species,
    );

    // Aktualisiere die Eltern-Kind-Beziehungen
    if let Ok(mut parent1) = parent_query.get_mut(parent1_entity) {
        parent1.children.push(child_entity);
    }

    if let Ok(mut parent2) = parent_query.get_mut(*parent2_entity) {
        parent2.children.push(child_entity);
    }
}

// Hilfsfunktion zur Erzeugung eines neuen Organismus aus zwei Elternteilen
fn create_child(
    commands: &mut Commands,
    parent1_entity: Entity,
    parent2_entity: Entity,
    parent1_genotype: &Genotype,
    parent2_genotype: &Genotype,
    parent1_species: &SpeciesGenes,
    parent2_species: &SpeciesGenes,
) -> Entity {
    // Erstelle einen neuen Genotyp durch Kombination der Eltern-Genotypen
    let mut child_genotype = Genotype::new();

    // Kombiniere Gene von beiden Eltern
    // Wir gehen durch alle Gene, die in mindestens einem Elternteil vorhanden sind
    let all_gene_ids: HashSet<String> = parent1_genotype
        .gene_pairs
        .keys()
        .chain(parent2_genotype.gene_pairs.keys())
        .cloned()
        .collect();

    let mut rng = rand::thread_rng();

    for gene_id in all_gene_ids {
        let parent1_gene = parent1_genotype.gene_pairs.get(&gene_id);
        let parent2_gene = parent2_genotype.gene_pairs.get(&gene_id);

        match (parent1_gene, parent2_gene) {
            // Wenn beide Eltern das Gen haben, kombiniere sie
            (Some(p1_gene), Some(p2_gene)) => {
                // Wähle zufällig ein Allel von jedem Elternteil
                let maternal = if rng.gen_bool(0.5) {
                    &p1_gene.maternal
                } else {
                    &p1_gene.paternal
                };

                let paternal = if rng.gen_bool(0.5) {
                    &p2_gene.maternal
                } else {
                    &p2_gene.paternal
                };

                // Füge das neue Genpaar zum Kind hinzu
                child_genotype.add_gene_pair(
                    &gene_id,
                    maternal.value,
                    paternal.value,
                    maternal.expression, // Vereinfachung: Expression des mütterlichen Gens
                    p1_gene.chromosome_type,
                );
            }
            // Wenn nur ein Elternteil das Gen hat, gib es mit 50% Wahrscheinlichkeit weiter
            (Some(p1_gene), None) => {
                if rng.gen_bool(0.5) {
                    let allele = if rng.gen_bool(0.5) {
                        &p1_gene.maternal
                    } else {
                        &p1_gene.paternal
                    };

                    // Für das zweite Allel nehmen wir eine Kopie des ersten (vereinfacht)
                    child_genotype.add_gene_pair(
                        &gene_id,
                        allele.value,
                        allele.value,
                        allele.expression,
                        p1_gene.chromosome_type,
                    );
                }
            }
            (None, Some(p2_gene)) => {
                if rng.gen_bool(0.5) {
                    let allele = if rng.gen_bool(0.5) {
                        &p2_gene.maternal
                    } else {
                        &p2_gene.paternal
                    };

                    child_genotype.add_gene_pair(
                        &gene_id,
                        allele.value,
                        allele.value,
                        allele.expression,
                        p2_gene.chromosome_type,
                    );
                }
            }
            (None, None) => {} // Sollte nicht vorkommen, aber zur Sicherheit
        }
    }

    // Kleine Chance auf Mutation
    apply_mutations(&mut child_genotype);

    // Erstelle Spezies-Liste für das Kind
    let mut child_species = SpeciesGenes::new();

    // Vereinige die Spezies beider Eltern
    let mut combined_species = parent1_species.species.clone();
    for species in &parent2_species.species {
        if !combined_species.contains(species) {
            combined_species.push(species.clone());
        }
    }
    child_species.species = combined_species;

    // Erzeuge die neue Entity mit allen notwendigen Komponenten
    let child = commands
        .spawn((
            child_genotype,
            Phenotype::new(),
            PhysicalAttributes::default(),
            MentalAttributes::default(),
            SocialAttributes::default(),
            VisualTraits {
                skin_color: (0.8, 0.65, 0.55), // Default, wird später durch Systeme angepasst
                hair_color: (0.3, 0.2, 0.1),   // Default
                eye_color: (0.3, 0.5, 0.7),    // Default
            },
            child_species,
            BodyStructure::humanoid(),
            Personality::default_traits(),
            Parent { children: vec![] },
            Ancestry {
                mother: Some(parent1_entity),
                father: Some(parent2_entity),
                generation: 1, // Generation könnte später von den Eltern abgeleitet werden
            },
            Fertility {
                fertility_rate: 0.0, // Startet unfruchtbar (muss erst wachsen)
                reproduction_cooldown: None,
                compatibility_modifiers: HashMap::new(),
                maturity: false,
            },
        ))
        .id();

    return child;
}

// Hilfsfunktion zum Anwenden zufälliger Mutationen
fn apply_mutations(genotype: &mut Genotype) {
    // Geringe Chance auf Mutation pro Gen
    const MUTATION_CHANCE: f32 = 0.01; // 1% Mutationswahrscheinlichkeit
    let mut rng = rand::thread_rng();

    for (_, gene_pair) in genotype.gene_pairs.iter_mut() {
        // Maternal-Mutation
        if rng.gen::<f32>() < MUTATION_CHANCE {
            let mut new_value = gene_pair.maternal.value + rng.gen_range(-0.1..0.1);
            // Begrenze den Wert auf den gültigen Bereich
            new_value = new_value.max(0.0).min(1.0);
            gene_pair.maternal.value = new_value;
        }

        // Paternal-Mutation
        if rng.gen::<f32>() < MUTATION_CHANCE {
            let mut new_value = gene_pair.paternal.value + rng.gen_range(-0.1..0.1);
            // Begrenze den Wert auf den gültigen Bereich
            new_value = new_value.max(0.0).min(1.0);
            gene_pair.paternal.value = new_value;
        }
    }
}
// System zur Berechnung des Phänotyps aus dem Genotyp
pub fn genotype_to_phenotype_system(mut query: Query<(&Genotype, &mut Phenotype)>) {
    for (genotype, mut phenotype) in query.iter_mut() {
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

            // Speichere den Wert im allgemeinen Phänotyp
            phenotype.attributes.insert(gene_id.clone(), value);

            // Speichere den Wert auch in der entsprechenden Chromosomen-Gruppe
            phenotype
                .attribute_groups
                .entry(gene_pair.chromosome_type)
                .or_insert_with(HashMap::new)
                .insert(gene_id.clone(), value);
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
            if let Some(strength_value) = attribute_values.get("gene_strength") {
                physical_attrs.strength.base_value = strength_value * 100.0;
                physical_attrs.strength.current_value = physical_attrs.strength.base_value;
            }

            if let Some(agility_value) = attribute_values.get("gene_agility") {
                physical_attrs.agility.base_value = agility_value * 100.0;
                physical_attrs.agility.current_value = physical_attrs.agility.base_value;
            }

            if let Some(toughness_value) = attribute_values.get("gene_toughness") {
                physical_attrs.toughness.base_value = toughness_value * 100.0;
                physical_attrs.toughness.current_value = physical_attrs.toughness.base_value;
            }

            if let Some(endurance_value) = attribute_values.get("gene_endurance") {
                physical_attrs.endurance.base_value = endurance_value * 100.0;
                physical_attrs.endurance.current_value = physical_attrs.endurance.base_value;
            }

            if let Some(recuperation_value) = attribute_values.get("gene_recuperation") {
                physical_attrs.recuperation.base_value = recuperation_value * 100.0;
                physical_attrs.recuperation.current_value = physical_attrs.recuperation.base_value;
            }

            if let Some(disease_resistance_value) = attribute_values.get("gene_disease_resistance")
            {
                physical_attrs.disease_resistance.base_value = disease_resistance_value * 100.0;
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
            if let Some(focus_value) = attribute_values.get("gene_focus") {
                mental_attrs.focus.base_value = focus_value * 100.0;
                mental_attrs.focus.current_value = mental_attrs.focus.base_value;
            }

            if let Some(creativity_value) = attribute_values.get("gene_creativity") {
                mental_attrs.creativity.base_value = creativity_value * 100.0;
                mental_attrs.creativity.current_value = mental_attrs.creativity.base_value;
            }

            if let Some(willpower_value) = attribute_values.get("gene_willpower") {
                mental_attrs.willpower.base_value = willpower_value * 100.0;
                mental_attrs.willpower.current_value = mental_attrs.willpower.base_value;
            }

            if let Some(analytical_ability_value) = attribute_values.get("gene_analytical_ability")
            {
                mental_attrs.analytical_ability.base_value = analytical_ability_value * 100.0;
                mental_attrs.analytical_ability.current_value =
                    mental_attrs.analytical_ability.base_value;
            }

            if let Some(intuition_value) = attribute_values.get("gene_intuition") {
                mental_attrs.intuition.base_value = intuition_value * 100.0;
                mental_attrs.intuition.current_value = mental_attrs.intuition.base_value;
            }

            if let Some(memory_value) = attribute_values.get("gene_memory") {
                mental_attrs.memory.base_value = memory_value * 100.0;
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
            if let Some(empathy_value) = attribute_values.get("gene_empathy") {
                social_attrs.empathy.base_value = empathy_value * 100.0;
                social_attrs.empathy.current_value = social_attrs.empathy.base_value;
            }

            if let Some(leadership_value) = attribute_values.get("gene_leadership") {
                social_attrs.leadership.base_value = leadership_value * 100.0;
                social_attrs.leadership.current_value = social_attrs.leadership.base_value;
            }

            if let Some(social_awareness_value) = attribute_values.get("gene_social_awareness") {
                social_attrs.social_awareness.base_value = social_awareness_value * 100.0;
                social_attrs.social_awareness.current_value =
                    social_attrs.social_awareness.base_value;
            }

            if let Some(linguistic_ability_value) = attribute_values.get("gene_linguistic_ability")
            {
                social_attrs.linguistic_ability.base_value = linguistic_ability_value * 100.0;
                social_attrs.linguistic_ability.current_value =
                    social_attrs.linguistic_ability.base_value;
            }

            if let Some(negotiation_value) = attribute_values.get("gene_negotiation") {
                social_attrs.negotiation.base_value = negotiation_value * 100.0;
                social_attrs.negotiation.current_value = social_attrs.negotiation.base_value;
            }
        }
    }
}

// System zur Anwendung des Phänotyps auf die Persönlichkeitsmerkmale
// TODO: Implementiere fehlt noch
#[allow(dead_code)]
pub fn apply_personality_system(mut query: Query<(&Phenotype, &mut Personality)>) {
    for (phenotype, mut personality) in query.iter_mut() {
        // Holen der Persönlichkeitswerte aus der Persönlichkeits-Chromosomen-Gruppe
        if let Some(personality_values) =
            phenotype.attribute_groups.get(&ChromosomeType::Personality)
        {
            for (trait_id, value) in personality_values.iter() {
                // Strip off the "gene_" prefix for cleaner trait names
                let trait_name = trait_id.strip_prefix("gene_").unwrap_or(trait_id);
                personality.traits.insert(trait_name.to_string(), *value);
            }
        }
    }
}

// System zur Aktualisierung der Körperstruktur basierend auf Genen
pub fn update_body_structure_system(mut query: Query<(&Phenotype, &mut BodyStructure)>) {
    for (phenotype, mut body_structure) in query.iter_mut() {
        // Holen der Körperstrukturwerte aus der entsprechenden Chromosomen-Gruppe
        if let Some(body_values) = phenotype
            .attribute_groups
            .get(&ChromosomeType::BodyStructure)
        {
            // Rekursive Hilfsfunktion zum Aktualisieren von Körperteilen basierend auf Genen
            fn update_body_part(body_part: &mut BodyComponent, body_values: &HashMap<String, f32>) {
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

            // Starte mit der Wurzelkomponente
            update_body_part(&mut body_structure.root, body_values);
        }
    }
}

// System zur Berechnung visueller Merkmale basierend auf Genen
pub fn update_visual_traits_system(
    mut query: Query<(&Phenotype, &mut VisualTraits, &SpeciesGenes)>,
    skin_palette: Res<SkinColorPalette>,
) {
    for (phenotype, mut visual_traits, species_genes) in query.iter_mut() {
        // Für visuelle Merkmale verwenden wir primär die VisualTraits-Chromosomengruppe
        let visual_genes = phenotype
            .attribute_groups
            .get(&ChromosomeType::VisualTraits);

        // Hautfarbe berechnen
        if let Some(visual_values) = visual_genes {
            // Wenn wir spezifische Gene für RGB-Komponenten haben, verwenden wir diese
            if visual_values.contains_key("gene_skin_r")
                && visual_values.contains_key("gene_skin_g")
                && visual_values.contains_key("gene_skin_b")
            {
                visual_traits.skin_color = (
                    *visual_values.get("gene_skin_r").unwrap(),
                    *visual_values.get("gene_skin_g").unwrap(),
                    *visual_values.get("gene_skin_b").unwrap(),
                );
            }
            // Andernfalls prüfen wir auf einen allgemeinen Hautton-Wert
            else if visual_values.contains_key("gene_skin_tone") {
                let skin_tone = *visual_values.get("gene_skin_tone").unwrap();

                // Hautfarbe direkt aus dem Farbpaletten-Wert ableiten
                // Wir nutzen den ersten gefundenen Speziestyp, falls vorhanden
                if !species_genes.species.is_empty() {
                    let primary_species = &species_genes.species[0];

                    if let Some(species_colors) = skin_palette.colors.get(primary_species) {
                        if !species_colors.is_empty() {
                            // Wähle eine Hautfarbe basierend auf dem genetischen Hautton
                            let color_index = ((skin_tone * (species_colors.len() as f32 - 1.0))
                                as usize)
                                .min(species_colors.len() - 1);

                            visual_traits.skin_color = species_colors[color_index];
                        }
                    }
                } else {
                    // Fallback: Einfacher Grauwert basierend auf dem Hautton
                    visual_traits.skin_color = (skin_tone, skin_tone, skin_tone);
                }
            }
        } else {
            // Fallback: Verwende Standardwerte oder behalte aktuelle Werte bei
        }

        // Weitere visuelle Merkmale wie Größe und Körperbau aktualisieren
        // if let Some(visual_values) = visual_genes {}
    }
}
