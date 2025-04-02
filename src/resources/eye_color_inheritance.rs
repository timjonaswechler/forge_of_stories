// Neue Datei: src/resources/eye_color_inheritance.rs
use crate::components::visual_traits::EyeColor;
use bevy::ecs::system::Resource;
use rand::Rng;
use std::collections::HashMap;

// Struktur für die Vererbungsmatrix
#[derive(Resource)]
pub struct EyeColorInheritance {
    inheritance_matrix: HashMap<(EyeColor, EyeColor), Vec<(EyeColor, f32)>>,
}

impl EyeColorInheritance {
    pub fn new() -> Self {
        let mut inheritance_matrix: HashMap<(EyeColor, EyeColor), Vec<(EyeColor, f32)>> =
            HashMap::new();

        // Beide Eltern haben blaue Augen
        inheritance_matrix.insert(
            (EyeColor::Blue, EyeColor::Blue),
            vec![(EyeColor::Blue, 0.99), (EyeColor::Green, 0.01)],
        );

        // Beide Eltern haben braune Augen
        inheritance_matrix.insert(
            (EyeColor::Brown, EyeColor::Brown),
            vec![
                (EyeColor::Brown, 0.74),
                (EyeColor::Green, 0.19),
                (EyeColor::Blue, 0.07),
            ],
        );

        // Beide Eltern haben graue Augen
        inheritance_matrix.insert(
            (EyeColor::Gray, EyeColor::Gray),
            vec![
                (EyeColor::Gray, 0.66),
                (EyeColor::Blue, 0.17),
                (EyeColor::Green, 0.17),
            ],
        );

        // Beide Eltern haben grüne Augen
        inheritance_matrix.insert(
            (EyeColor::Green, EyeColor::Green),
            vec![(EyeColor::Blue, 0.75), (EyeColor::Green, 0.25)],
        );

        // Ein Elternteil hat blaue Augen, anderer braune
        inheritance_matrix.insert(
            (EyeColor::Blue, EyeColor::Brown),
            vec![(EyeColor::Blue, 0.5), (EyeColor::Brown, 0.5)],
        );

        // Ein Elternteil hat blaue Augen, anderer grüne
        inheritance_matrix.insert(
            (EyeColor::Blue, EyeColor::Green),
            vec![(EyeColor::Blue, 0.5), (EyeColor::Green, 0.5)],
        );

        // Ein Elternteil hat blaue Augen, anderer graue
        inheritance_matrix.insert(
            (EyeColor::Blue, EyeColor::Gray),
            vec![(EyeColor::Gray, 0.8), (EyeColor::Blue, 0.2)],
        );

        // Ein Elternteil hat grüne Augen, anderer graue
        inheritance_matrix.insert(
            (EyeColor::Green, EyeColor::Gray),
            vec![(EyeColor::Gray, 0.75), (EyeColor::Green, 0.25)],
        );

        // Ein Elternteil hat grüne Augen, anderer braune
        inheritance_matrix.insert(
            (EyeColor::Green, EyeColor::Brown),
            vec![
                (EyeColor::Brown, 0.5),
                (EyeColor::Green, 0.38),
                (EyeColor::Blue, 0.12),
            ],
        );

        // Ein Elternteil hat graue Augen, anderer braune
        inheritance_matrix.insert(
            (EyeColor::Gray, EyeColor::Brown),
            vec![(EyeColor::Gray, 0.35), (EyeColor::Brown, 0.65)],
        );

        // Symmetrie der Matrix sicherstellen (A,B) = (B,A)
        let keys: Vec<(EyeColor, EyeColor)> = inheritance_matrix.keys().cloned().collect();
        for key in keys {
            let value = inheritance_matrix[&key].clone();
            let reverse_key = (key.1, key.0);

            if !inheritance_matrix.contains_key(&reverse_key) {
                inheritance_matrix.insert(reverse_key, value);
            }
        }

        // Für fehlende Kombinationen mit seltenen Augenfarben (gelb, rot, schwarz, weiß)
        // können wir später Standardregeln hinzufügen

        Self { inheritance_matrix }
    }

    // Bestimmt die Augenfarbe eines Kindes basierend auf den Eltern
    pub fn inherit_eye_color(&self, parent1: EyeColor, parent2: EyeColor) -> EyeColor {
        let mut rng = rand::thread_rng();

        // Versuche, die Kombination in der Matrix zu finden
        if let Some(probabilities) = self.inheritance_matrix.get(&(parent1, parent2)) {
            // Zufallszahl zwischen 0 und 1
            let random_value = rng.gen::<f32>();
            let mut cumulative_prob = 0.0;

            // Durchlaufe die Wahrscheinlichkeiten und wähle entsprechend aus
            for (color, prob) in probabilities {
                cumulative_prob += prob;
                if random_value <= cumulative_prob {
                    return *color;
                }
            }

            // Fallback bei Rundungsfehlern
            return probabilities[0].0;
        } else if let Some(probabilities) = self.inheritance_matrix.get(&(parent2, parent1)) {
            // Versuche die umgekehrte Reihenfolge
            let random_value = rng.gen::<f32>();
            let mut cumulative_prob = 0.0;

            for (color, prob) in probabilities {
                cumulative_prob += prob;
                if random_value <= cumulative_prob {
                    return *color;
                }
            }

            return probabilities[0].0;
        }

        // Standardregel: Wenn keine spezifische Regel existiert,
        // mische zufällig zwischen den Elternfarben
        if rng.gen::<bool>() {
            parent1
        } else {
            parent2
        }
    }
}
