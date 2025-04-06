// Neue Datei: src/resources/eye_color_inheritance.rs
use crate::components::visual_traits::EyeColor;
use bevy::ecs::system::Resource;
use bevy::prelude::warn;
use rand::Rng;
use std::collections::HashMap;

#[derive(Resource)]
pub struct EyeColorInheritance {
    inheritance_matrices: HashMap<String, HashMap<(EyeColor, EyeColor), Vec<(EyeColor, f32)>>>,
    default_matrix: HashMap<(EyeColor, EyeColor), Vec<(EyeColor, f32)>>,
}

impl EyeColorInheritance {
    pub fn new() -> Self {
        let mut inheritance_matrices = HashMap::new();

        // Menschliche Vererbungsmatrix erstellen
        let human_matrix = Self::create_human_inheritance();
        inheritance_matrices.insert("Mensch".to_string(), human_matrix);

        // Elfische Vererbungsmatrix erstellen (basierend auf anderen Wahrscheinlichkeiten)
        let elf_matrix = Self::create_elf_inheritance();
        inheritance_matrices.insert("Elf".to_string(), elf_matrix);

        // Orkische Vererbungsmatrix erstellen
        let orc_matrix = Self::create_orc_inheritance();
        inheritance_matrices.insert("Ork".to_string(), orc_matrix);

        Self {
            inheritance_matrices,
            default_matrix: Self::create_human_inheritance(), // Menschen als Standard
        }
    }

    // Erstellt die menschliche Vererbungsmatrix (wie im Bild gezeigt)
    fn create_human_inheritance() -> HashMap<(EyeColor, EyeColor), Vec<(EyeColor, f32)>> {
        let mut matrix = HashMap::new();

        // Beide Eltern haben blaue Augen
        matrix.insert(
            (EyeColor::Blue, EyeColor::Blue),
            vec![(EyeColor::Blue, 0.99), (EyeColor::Green, 0.01)],
        );

        // Beide Eltern haben braune Augen
        matrix.insert(
            (EyeColor::Brown, EyeColor::Brown),
            vec![
                (EyeColor::Brown, 0.74),
                (EyeColor::Green, 0.19),
                (EyeColor::Blue, 0.07),
            ],
        );

        // Beide Eltern haben graue Augen
        matrix.insert(
            (EyeColor::Gray, EyeColor::Gray),
            vec![
                (EyeColor::Gray, 0.66),
                (EyeColor::Blue, 0.17),
                (EyeColor::Green, 0.17),
            ],
        );

        // Beide Eltern haben grüne Augen
        matrix.insert(
            (EyeColor::Green, EyeColor::Green),
            vec![(EyeColor::Blue, 0.75), (EyeColor::Green, 0.25)],
        );

        // Ein Elternteil hat blaue Augen, anderer braune
        matrix.insert(
            (EyeColor::Blue, EyeColor::Brown),
            vec![(EyeColor::Blue, 0.5), (EyeColor::Brown, 0.5)],
        );

        // Ein Elternteil hat blaue Augen, anderer grüne
        matrix.insert(
            (EyeColor::Blue, EyeColor::Green),
            vec![(EyeColor::Blue, 0.5), (EyeColor::Green, 0.5)],
        );

        // Ein Elternteil hat blaue Augen, anderer graue
        matrix.insert(
            (EyeColor::Blue, EyeColor::Gray),
            vec![(EyeColor::Gray, 0.8), (EyeColor::Blue, 0.2)],
        );

        // Ein Elternteil hat grüne Augen, anderer graue
        matrix.insert(
            (EyeColor::Green, EyeColor::Gray),
            vec![(EyeColor::Gray, 0.75), (EyeColor::Green, 0.25)],
        );

        // Ein Elternteil hat grüne Augen, anderer braune
        matrix.insert(
            (EyeColor::Green, EyeColor::Brown),
            vec![
                (EyeColor::Brown, 0.5),
                (EyeColor::Green, 0.38),
                (EyeColor::Blue, 0.12),
            ],
        );

        // Ein Elternteil hat graue Augen, anderer braune
        matrix.insert(
            (EyeColor::Gray, EyeColor::Brown),
            vec![(EyeColor::Gray, 0.35), (EyeColor::Brown, 0.65)],
        );

        // Symmetrie der Matrix sicherstellen (A,B) = (B,A)
        let keys: Vec<(EyeColor, EyeColor)> = matrix.keys().cloned().collect();
        for key in keys {
            let value = matrix[&key].clone();
            let reverse_key = (key.1, key.0);

            if !matrix.contains_key(&reverse_key) {
                matrix.insert(reverse_key, value);
            }
        }

        matrix
    }

    // Erstellt die elfische Vererbungsmatrix (Beispiel - kann angepasst werden)
    fn create_elf_inheritance() -> HashMap<(EyeColor, EyeColor), Vec<(EyeColor, f32)>> {
        let mut matrix = HashMap::new();

        // Elfen haben häufiger blaue und grüne Augen
        matrix.insert(
            (EyeColor::Blue, EyeColor::Blue),
            vec![(EyeColor::Blue, 1.0)],
        );

        matrix.insert(
            (EyeColor::Green, EyeColor::Green),
            vec![(EyeColor::Green, 0.9), (EyeColor::Blue, 0.1)],
        );

        matrix.insert(
            (EyeColor::Blue, EyeColor::Green),
            vec![(EyeColor::Blue, 0.6), (EyeColor::Green, 0.4)],
        );

        // Bei weiteren Farben entsprechend ergänzen...

        // Symmetrie der Matrix sicherstellen (A,B) = (B,A)
        let keys: Vec<(EyeColor, EyeColor)> = matrix.keys().cloned().collect();
        for key in keys {
            let value = matrix[&key].clone();
            let reverse_key = (key.1, key.0);

            if !matrix.contains_key(&reverse_key) {
                matrix.insert(reverse_key, value);
            }
        }

        matrix
    }

    // Erstellt die orkische Vererbungsmatrix (Beispiel - kann angepasst werden)
    fn create_orc_inheritance() -> HashMap<(EyeColor, EyeColor), Vec<(EyeColor, f32)>> {
        let mut matrix = HashMap::new();

        // Orks haben häufiger gelbe, rote und schwarze Augen
        matrix.insert(
            (EyeColor::Red, EyeColor::Red),
            vec![(EyeColor::Red, 0.8), (EyeColor::Yellow, 0.2)],
        );

        matrix.insert(
            (EyeColor::Yellow, EyeColor::Yellow),
            vec![(EyeColor::Yellow, 0.7), (EyeColor::Red, 0.3)],
        );

        matrix.insert(
            (EyeColor::Red, EyeColor::Yellow),
            vec![(EyeColor::Red, 0.5), (EyeColor::Yellow, 0.5)],
        );

        // Bei weiteren Farben entsprechend ergänzen...

        // Symmetrie der Matrix sicherstellen (A,B) = (B,A)
        let keys: Vec<(EyeColor, EyeColor)> = matrix.keys().cloned().collect();
        for key in keys {
            let value = matrix[&key].clone();
            let reverse_key = (key.1, key.0);

            if !matrix.contains_key(&reverse_key) {
                matrix.insert(reverse_key, value);
            }
        }

        matrix
    }

    // Bestimmt die Augenfarbe eines Kindes basierend auf den Eltern und ihrer Spezies
    pub fn inherit_eye_color<R: Rng + ?Sized>(
        &self,
        species: &str,
        parent1: EyeColor,
        parent2: EyeColor,
        rng: &mut R,
    ) -> EyeColor {
        let matrix = self
            .inheritance_matrices
            .get(species)
            .unwrap_or(&self.default_matrix);

        if let Some(probabilities) = matrix.get(&(parent1, parent2)) {
            let random_value = rng.gen::<f32>(); // <- gen()
            let mut cumulative_prob = 0.0;

            for (color, prob) in probabilities {
                cumulative_prob += prob;
                if random_value <= cumulative_prob {
                    return *color;
                }
            }
            probabilities.first().map_or(parent1, |(color, _)| *color) // Fallback
        } else {
            warn!("Keine Vererbungsregel für Augenfarben {:?} / {:?} bei Spezies '{}' gefunden. Wähle zufällig.", parent1, parent2, species);
            if rng.gen::<bool>() {
                // <- gen()
                parent1
            } else {
                parent2
            }
        }
    }
}
