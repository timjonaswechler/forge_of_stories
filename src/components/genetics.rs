// src/components/genetics.rs

use bevy::prelude::*;
use std::collections::HashMap;

// Gen-Ausprägung (dominant, rezessiv, kodominant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeneExpression {
    Dominant,
    Recessive,
    Codominant,
}

// Chromosomen-Typ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChromosomeType {
    BodyStructure, // Körperbau
    Attributes,    // Attributwerte
    Personality,   // Persönlichkeit
    VisualTraits,  // Aussehen
    Specialized,   // Spezielle Fähigkeiten/Merkmale
}

// Gene: Blaupause für ein Gen in der Gendatenbank/dem Genpool der Welt
// Definiert die Eigenschaften und möglichen Werte, die ein Gen haben kann
#[derive(Component, Debug, Clone)]
pub struct Gene {
    pub id: String,                                // Eindeutiger Identifikator
    pub name: String,                              // Lesbarer Name (z.B. "Augenfarbe")
    pub description: String,                       // Kurze Beschreibung des Gens
    pub possible_expressions: Vec<GeneExpression>, // Mögliche Expressionen
    pub default_value: f32,                        // Standard/Ausgangswert
    pub value_range: (f32, f32),                   // Min/Max-Wertebereich
    pub mutation_rate: f32,                        // Wahrscheinlichkeit von Mutationen
    pub chromosome_type: ChromosomeType,           // Zuordnung zu einem Chromosomentyp
}

// GeneVariant: Eine spezifische Ausprägung (Allel) eines Gens in einem Individuum
// Jedes Individuum hat zwei GeneVariants für jedes Gen (von Mutter und Vater)
#[derive(Debug, Clone)]
pub struct GeneVariant {
    pub gene_id: String,
    pub value: f32,
    pub expression: GeneExpression,
}

// GenePair: Ein Paar von Genvarianten, das ein komplettes Gen in einem diploiden Organismus darstellt
// Beinhaltet sowohl das mütterliche als auch das väterliche Allel
#[derive(Debug, Clone)]
pub struct GenePair {
    pub maternal: GeneVariant,           // Von der Mutter
    pub paternal: GeneVariant,           // Vom Vater
    pub chromosome_type: ChromosomeType, // Art des Chromosoms
}

// Genpool eines Organismus
#[derive(Component, Debug, Clone)]
pub struct Genotype {
    pub gene_pairs: HashMap<String, GenePair>, // Gen-ID -> Genpaar
    pub chromosome_groups: HashMap<ChromosomeType, Vec<String>>, // Gruppierung nach Chromosomen-Typ
}

impl Genotype {
    pub fn new() -> Self {
        Self {
            gene_pairs: HashMap::new(),
            chromosome_groups: HashMap::new(),
        }
    }

    // Hilfsmethode zum Hinzufügen eines Genpaars
    pub fn add_gene_pair(
        &mut self,
        gene_id: &str,
        maternal_value: f32,
        paternal_value: f32,
        expression: GeneExpression,
        chromosome_type: ChromosomeType,
    ) {
        let gene_pair = GenePair {
            maternal: GeneVariant {
                gene_id: gene_id.to_string(),
                value: maternal_value,
                expression,
            },
            paternal: GeneVariant {
                gene_id: gene_id.to_string(),
                value: paternal_value,
                expression,
            },
            chromosome_type,
        };

        self.gene_pairs.insert(gene_id.to_string(), gene_pair);

        // Zum entsprechenden Chromosomen-Typ hinzufügen
        self.chromosome_groups
            .entry(chromosome_type)
            .or_insert_with(Vec::new)
            .push(gene_id.to_string());
    }
}

// Phänotyp (die sichtbaren/wirksamen Eigenschaften)
#[derive(Component, Debug, Clone)]
pub struct Phenotype {
    pub attributes: HashMap<String, f32>, // Gen-ID -> Phänotyp-Wert
    pub attribute_groups: HashMap<ChromosomeType, HashMap<String, f32>>, // Gruppierung nach Chromosomen-Typ
}

impl Phenotype {
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
            attribute_groups: HashMap::new(),
        }
    }
}

// Körperbaustein (für hierarchische Körperstruktur)
#[derive(Debug, Clone)]
pub struct BodyComponent {
    pub id: String,
    pub name: String,
    pub properties: HashMap<String, f32>, // Eigenschaften wie Größe, Form, etc.
    pub children: Vec<BodyComponent>,     // Unterkomponenten
}

impl BodyComponent {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            properties: HashMap::new(),
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: BodyComponent) {
        self.children.push(child);
    }

    pub fn set_property(&mut self, key: &str, value: f32) {
        self.properties.insert(key.to_string(), value);
    }
}

// Körperstruktur
#[derive(Component, Debug, Clone)]
pub struct BodyStructure {
    pub root: BodyComponent,
}

impl BodyStructure {
    pub fn new() -> Self {
        // Erstelle eine leere Becken-Komponente als Wurzel
        Self {
            root: BodyComponent::new("pelvis", "Becken"),
        }
    }

    // Erstellt eine Standard-Humanoid-Körperstruktur
    pub fn humanoid() -> Self {
        let mut body = Self::new();

        // Kopf und Hals
        let mut neck = BodyComponent::new("neck", "Hals");
        let mut head = BodyComponent::new("head", "Kopf");

        // Kopfkomponenten
        let mouth = BodyComponent::new("mouth", "Mund");
        let left_eye = BodyComponent::new("left_eye", "Linkes Auge");
        let right_eye = BodyComponent::new("right_eye", "Rechtes Auge");
        let left_ear = BodyComponent::new("left_ear", "Linkes Ohr");
        let right_ear = BodyComponent::new("right_ear", "Rechtes Ohr");

        head.add_child(mouth);
        head.add_child(left_eye);
        head.add_child(right_eye);
        head.add_child(left_ear);
        head.add_child(right_ear);
        neck.add_child(head);

        // Brustkorb
        let mut chest = BodyComponent::new("chest", "Brustkorb");
        let heart = BodyComponent::new("heart", "Herz");
        let lungs = BodyComponent::new("lungs", "Lunge");

        // Schultern und Arme
        let mut left_shoulder = BodyComponent::new("left_shoulder", "Linke Schulter");
        let mut left_upper_arm = BodyComponent::new("left_upper_arm", "Linker Oberarm");
        let mut left_lower_arm = BodyComponent::new("left_lower_arm", "Linker Unterarm");
        let mut left_hand = BodyComponent::new("left_hand", "Linke Hand");

        // Finger der linken Hand
        for i in 1..=5 {
            left_hand.add_child(BodyComponent::new(
                &format!("left_finger_{}", i),
                &format!("Linker Finger {}", i),
            ));
        }

        left_lower_arm.add_child(left_hand);
        left_upper_arm.add_child(left_lower_arm);
        left_shoulder.add_child(left_upper_arm);

        // Rechte Schulter und Arm (ähnlich wie links)
        let mut right_shoulder = BodyComponent::new("right_shoulder", "Rechte Schulter");
        let mut right_upper_arm = BodyComponent::new("right_upper_arm", "Rechter Oberarm");
        let mut right_lower_arm = BodyComponent::new("right_lower_arm", "Rechter Unterarm");
        let mut right_hand = BodyComponent::new("right_hand", "Rechte Hand");

        // Finger der rechten Hand
        for i in 1..=5 {
            right_hand.add_child(BodyComponent::new(
                &format!("right_finger_{}", i),
                &format!("Rechter Finger {}", i),
            ));
        }

        right_lower_arm.add_child(right_hand);
        right_upper_arm.add_child(right_lower_arm);
        right_shoulder.add_child(right_upper_arm);

        chest.add_child(heart);
        chest.add_child(lungs);
        chest.add_child(left_shoulder);
        chest.add_child(right_shoulder);

        // Bauch
        let mut abdomen = BodyComponent::new("abdomen", "Bauch");
        let stomach = BodyComponent::new("stomach", "Magen");
        let intestines = BodyComponent::new("intestines", "Darm");
        let liver = BodyComponent::new("liver", "Leber");
        let kidneys = BodyComponent::new("kidneys", "Nieren");

        abdomen.add_child(stomach);
        abdomen.add_child(intestines);
        abdomen.add_child(liver);
        abdomen.add_child(kidneys);

        // Beine
        let mut left_thigh = BodyComponent::new("left_thigh", "Linker Oberschenkel");
        let mut left_calf = BodyComponent::new("left_calf", "Linker Unterschenkel");
        let mut left_foot = BodyComponent::new("left_foot", "Linker Fuß");

        // Zehen des linken Fußes
        for i in 1..=5 {
            left_foot.add_child(BodyComponent::new(
                &format!("left_toe_{}", i),
                &format!("Linker Zeh {}", i),
            ));
        }

        left_calf.add_child(left_foot);
        left_thigh.add_child(left_calf);

        // Rechtes Bein (ähnlich wie links)
        let mut right_thigh = BodyComponent::new("right_thigh", "Rechter Oberschenkel");
        let mut right_calf = BodyComponent::new("right_calf", "Rechter Unterschenkel");
        let mut right_foot = BodyComponent::new("right_foot", "Rechter Fuß");

        // Zehen des rechten Fußes
        for i in 1..=5 {
            right_foot.add_child(BodyComponent::new(
                &format!("right_toe_{}", i),
                &format!("Rechter Zeh {}", i),
            ));
        }

        right_calf.add_child(right_foot);
        right_thigh.add_child(right_calf);

        // Zusammensetzen des Körpers
        body.root.add_child(neck);
        body.root.add_child(chest);
        body.root.add_child(abdomen);
        body.root.add_child(left_thigh);
        body.root.add_child(right_thigh);

        body
    }
}

// Zusätzliche Gene für spezifische visuelle Merkmale
// #[derive(Component, Debug, Clone)]
// pub struct VisualTraits {
//     pub skin_color: (f32, f32, f32), // RGB-Werte für die Hautfarbe
//     pub hair_color: (f32, f32, f32), // RGB-Werte für die Haarfarbe
//     pub eye_color: (f32, f32, f32),  // RGB-Werte für die Augenfarbe
// }

// Persönlichkeitsmerkmale
#[derive(Component, Debug, Clone)]
pub struct Personality {
    pub traits: HashMap<String, f32>, // Persönlichkeitsmerkmal -> Wert (0.0-1.0)
}

impl Personality {
    pub fn new() -> Self {
        Self {
            traits: HashMap::new(),
        }
    }

    // Standardpersönlichkeit mit typischen Merkmalen
    pub fn default_traits() -> Self {
        let mut traits = HashMap::new();

        // Grundlegende Persönlichkeitsmerkmale (Big Five + einige Fantasy-relevante)
        traits.insert("openness".to_string(), 0.5); // Offenheit für Erfahrungen
        traits.insert("conscientiousness".to_string(), 0.5); // Gewissenhaftigkeit
        traits.insert("extraversion".to_string(), 0.5); // Extraversion
        traits.insert("agreeableness".to_string(), 0.5); // Verträglichkeit
        traits.insert("neuroticism".to_string(), 0.5); // Neurotizismus

        // Fantasy-spezifische Merkmale
        traits.insert("courage".to_string(), 0.5); // Mut
        traits.insert("honor".to_string(), 0.5); // Ehre
        traits.insert("curiosity".to_string(), 0.5); // Neugier
        traits.insert("spirituality".to_string(), 0.5); // Spiritualität
        traits.insert("greed".to_string(), 0.5); // Gier

        Self { traits }
    }
}

// Komponente für Spezieszugehörigkeit
#[derive(Component, Debug, Clone)]
pub struct SpeciesGenes {
    pub species: Vec<String>, // Liste aller Spezies, die in dem Genpool vorkommen
}

impl SpeciesGenes {
    pub fn new() -> Self {
        Self {
            species: Vec::new(),
        }
    }
}

// Komponente, die anzeigt, dass dieses Wesen ein Elternteil ist
#[derive(Component, Debug)]
pub struct Parent {
    pub children: Vec<Entity>,
}

#[derive(Component, Debug, Clone)]
pub struct VisualTraits {
    pub skin_color: (f32, f32, f32),
    pub hair_color: (f32, f32, f32),
    pub eye_color: (f32, f32, f32),
}

// Komponente, die auf die Eltern verweist
#[derive(Component, Debug)]
pub struct Ancestry {
    pub mother: Option<Entity>,
    pub father: Option<Entity>,
    pub generation: u32, // Generationszähler für evolutionäre Analyse
}

// Komponente für die Fruchtbarkeit und Fortpflanzungsfähigkeit
#[derive(Component, Debug, Clone)]
pub struct Fertility {
    pub fertility_rate: f32, // Grundlegende Fruchtbarkeitsrate (0.0-1.0)
    pub reproduction_cooldown: Option<f32>, // Abklingzeit nach Fortpflanzung
    pub compatibility_modifiers: HashMap<String, f32>, // Kompatibilität mit verschiedenen Spezies
    pub maturity: bool,      // Ist das Wesen fortpflanzungsfähig?
}

impl Fertility {
    pub fn new(fertility_rate: f32) -> Self {
        Self {
            fertility_rate,
            reproduction_cooldown: None,
            compatibility_modifiers: HashMap::new(),
            maturity: false,
        }
    }
}
