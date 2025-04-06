// Neue Datei: src/plugins/debug_plugin.rs
use bevy::prelude::*;
use std::str::FromStr;

use super::setup_plugin::AppState; // Importiere AppState für run_if
use crate::components::{
    attributes::{Attribute, MentalAttributes, PhysicalAttributes, SocialAttributes},
    gene_types::GeneType,
    genetics::{Genotype, Phenotype, SpeciesGenes},
    visual_traits::VisualTraits,
};
use crate::plugins::genetics_plugin::GeneticsSystemSet; // Für .after()

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // Füge Debug-Systeme nur hinzu, wenn im Debug-Build (optional aber empfohlen)
        #[cfg(debug_assertions)]
        {
            app.add_systems(
                Update,
                debug_entities
                    .after(GeneticsSystemSet::PhysicalTraits) // Läuft nach den Genetik-Systemen
                    .run_if(in_state(AppState::Running)),
            );
        }
    }
}

// --- Debug-Systeme, die vorher in main.rs waren ---

fn debug_entities(
    query: Query<(
        Entity,
        &Genotype,
        &Phenotype,
        &PhysicalAttributes,
        &MentalAttributes,
        &SocialAttributes,
        &VisualTraits,
        &SpeciesGenes,
    )>,
    mut ran_once: Local<bool>,
    app_state: Res<State<AppState>>,
) {
    // Die Prüfung auf AppState::Running ist hier redundant wegen .run_if(),
    // aber schadet auch nicht.
    if *app_state != AppState::Running {
        return;
    }
    if !*ran_once {
        info!(
            "=== DETAILLIERTE ENTITY-INFORMATIONEN (State: {:?}) ===",
            app_state.get()
        );
        for (entity, genotype, phenotype, physical, mental, social, visual, species) in query.iter()
        {
            info!("Entity: {:?}", entity);
            info!("----------------------------------------");
            info!("GENOTYP: {} Gene", genotype.gene_pairs.len());
            info!("PHÄNOTYP:");
            for (chrom_type, attributes) in &phenotype.attribute_groups {
                info!("  Chromosomentyp: {:?}", chrom_type);
                for (attr_id_str, gene_value) in attributes {
                    let gene_type_str = GeneType::from_str(attr_id_str).map_or_else(
                        |_| format!("Unbekannt: '{}'", attr_id_str),
                        |gt| format!("{:?}", gt),
                    );
                    info!(
                        "    {:<30}: Pheno-Wert: {:<6.3} (Expression: {:?})",
                        gene_type_str,
                        gene_value.value(),
                        gene_value.expression()
                    );
                }
            }
            info!("PHYSISCHE ATTRIBUTE:");
            debug_attribute(&physical.strength);
            debug_attribute(&physical.agility);
            debug_attribute(&physical.toughness);
            debug_attribute(&physical.endurance);
            debug_attribute(&physical.recuperation);
            debug_attribute(&physical.disease_resistance);

            info!("MENTALE ATTRIBUTE:");
            debug_attribute(&mental.analytical_ability);
            debug_attribute(&mental.focus);
            debug_attribute(&mental.willpower);
            debug_attribute(&mental.creativity);
            debug_attribute(&mental.intuition);
            debug_attribute(&mental.patience);
            debug_attribute(&mental.memory);
            debug_attribute(&mental.spatial_sense);

            info!("SOZIALE ATTRIBUTE:");
            debug_attribute(&social.empathy);
            debug_attribute(&social.social_awareness);
            debug_attribute(&social.linguistic_ability);
            debug_attribute(&social.musicality);
            debug_attribute(&social.leadership);
            debug_attribute(&social.negotiation);

            info!("VISUELLE MERKMALE:");
            let skin_srgba = visual.skin_color.to_srgba();
            info!(
                "  Hautfarbe (sRGB): R={:.3} G={:.3} B={:.3} (A={:.3})",
                skin_srgba.red, skin_srgba.green, skin_srgba.blue, skin_srgba.alpha
            );

            let hair_srgba = visual.hair_color.to_srgba();
            info!(
                "  Haarfarbe (sRGB): R={:.3} G={:.3} B={:.3} (A={:.3})",
                hair_srgba.red, hair_srgba.green, hair_srgba.blue, hair_srgba.alpha
            );

            let eye_srgba = visual.eye_color.to_srgba();
            info!(
                "  Augenfarbe (sRGB): R={:.3} G={:.3} B={:.3} (A={:.3})",
                eye_srgba.red, eye_srgba.green, eye_srgba.blue, eye_srgba.alpha
            );
            info!("SPEZIES: {:?}", species.species);
            info!("========================================\n");
        }
        *ran_once = true;
    }
}

fn debug_attribute(attribute: &Attribute) {
    info!(
        "  {:<20} ({:<20}): Base: {:<7.1}, Current: {:<7.1}, Effective: {:<7.1} (Max: {:.0}, Rust: {:?})",
        attribute.name,
        format!("{:?}", attribute.id),
        attribute.base_value,
        attribute.current_value,
        attribute.effective_value,
        attribute.max_value,
        attribute.rust_level
    );
}
