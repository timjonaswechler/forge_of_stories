use crate::genetics::{ChromosomeType, GeneType, Phenotype, VisualGene};
use crate::visuals::components::{EyeColor, VisualTraits};
use bevy::prelude::*;

pub fn apply_visual_traits_system(
    // ÄNDERUNG: Query verwendet jetzt VisualTraits direkt
    mut query: Query<(&Phenotype, &mut VisualTraits), Changed<Phenotype>>,
) {
    // Generiere die String Keys einmal außerhalb der Loop (unverändert)
    let key_skin_r = GeneType::Visual(VisualGene::SkinColorR).to_string();
    let key_skin_g = GeneType::Visual(VisualGene::SkinColorG).to_string();
    let key_skin_b = GeneType::Visual(VisualGene::SkinColorB).to_string();
    let key_hair_r = GeneType::Visual(VisualGene::HairColorR).to_string();
    let key_hair_g = GeneType::Visual(VisualGene::HairColorG).to_string();
    let key_hair_b = GeneType::Visual(VisualGene::HairColorB).to_string();
    let key_eye_color = GeneType::Visual(VisualGene::EyeColor).to_string();

    for (phenotype, mut visual_traits) in query.iter_mut() {
        if let Some(visual_values) = phenotype
            .attribute_groups
            .get(&ChromosomeType::VisualTraits)
        {
            // Hautfarbe: Lese Gen-Werte (sRGB 0-1) und erstelle Color::srgb
            let skin_r = visual_values.get(&key_skin_r).map_or(0.5, |g| g.value());
            let skin_g = visual_values.get(&key_skin_g).map_or(0.5, |g| g.value());
            let skin_b = visual_values.get(&key_skin_b).map_or(0.5, |g| g.value());
            visual_traits.skin_color = Color::srgb(
                // <- Änderung: Verwende srgb
                skin_r.clamp(0.0, 1.0),
                skin_g.clamp(0.0, 1.0),
                skin_b.clamp(0.0, 1.0),
            );

            // Haarfarbe: Lese Gen-Werte (sRGB 0-1) und erstelle Color::srgb
            let hair_r = visual_values.get(&key_hair_r).map_or(0.5, |g| g.value());
            let hair_g = visual_values.get(&key_hair_g).map_or(0.5, |g| g.value());
            let hair_b = visual_values.get(&key_hair_b).map_or(0.5, |g| g.value());
            visual_traits.hair_color = Color::srgb(
                // <- Änderung: Verwende srgb
                hair_r.clamp(0.0, 1.0),
                hair_g.clamp(0.0, 1.0),
                hair_b.clamp(0.0, 1.0),
            );

            // Augenfarbe: Lese Gen-Wert (f32), konvertiere zu EyeColor Enum,
            // bestimme RGB (sRGB 0-1) und erstelle Color::srgb
            if let Some(eye_color_gene) = visual_values.get(&key_eye_color) {
                let eye_color_val = eye_color_gene.value();
                if eye_color_val >= 0.0 {
                    // Einfache Prüfung, ob Wert plausibel ist
                    let eye_color_enum = EyeColor::from_f32(eye_color_val);
                    // Konvertiere das Enum zum sRGB-Tupel (0.0-1.0)
                    let srgb = match eye_color_enum {
                        // Verwende hier die gleichen sRGB-Werte wie vorher (angenommen sie waren schon sRGB-korrekt)
                        EyeColor::Brown => (0.55, 0.27, 0.07),
                        EyeColor::Green => (0.21, 0.47, 0.21),
                        EyeColor::Blue => (0.21, 0.35, 0.80), // Vorsicht: Blau ist oft nicht linear
                        EyeColor::Gray => (0.50, 0.50, 0.50),
                        EyeColor::Yellow => (0.80, 0.80, 0.20),
                        EyeColor::Red => (0.80, 0.20, 0.20),
                        EyeColor::Black => (0.10, 0.10, 0.10),
                        EyeColor::White => (0.90, 0.90, 0.90),
                    };
                    // Erstelle die Color mit Color::srgb und weise sie zu
                    visual_traits.eye_color = Color::srgb(srgb.0, srgb.1, srgb.2);
                // <- Änderung: Verwende srgb
                } else {
                    warn!(
                        "Ungültiger Genwert für Augenfarbe gefunden: {}. Verwende Fallback.",
                        eye_color_val
                    );
                    visual_traits.eye_color = Color::srgb(0.5, 0.5, 0.5); // <- Fallback Color::srgb
                }
            } else {
                warn!("Kein Augenfarben-Gen im Phänotyp gefunden. Verwende Fallback.");
                visual_traits.eye_color = Color::srgb(0.5, 0.5, 0.5); // <- Fallback Color::srgb
            }
        } else {
            warn!("Keine visuellen Gene im Phänotyp gefunden. Überspringe VisualTraits-Anwendung.");
        }
    }
}
