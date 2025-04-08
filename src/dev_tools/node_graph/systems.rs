use bevy::color::palettes::basic::*;
use bevy::prelude::*; // Importiere Bevy Preludes // Importiere Bevy Color Preludes
                      // Importiere benötigte Komponenten aus deiner Simulation
                      // Passe diese Pfade entsprechend deiner Struktur an!
use crate::genetics::components::SpeciesGenes;
// use crate::genetics::components::Genotype; // Falls auch Genotyp gebraucht wird

// Importiere die UI-Datenstrukturen und die Ressource
use super::resources::GraphUIData;
use super::ui_data::{VisLink, VisNode};
// use super::ui_data::VisLink; // Für später

// === Wichtig: Diese Konstanten müssen mit denen in context.rs übereinstimmen! ===
const PIN_ID_MULTIPLIER: usize = 10;
const INPUT_PIN_OFFSET: usize = 0;
const OUTPUT_PIN_OFFSET: usize = 1;

pub fn graph_data_provider_system(
    entity_query: Query<(Entity, &SpeciesGenes)>,
    mut graph_data: ResMut<GraphUIData>,
) {
    graph_data.nodes.clear();
    graph_data.links.clear(); // Auch Links für jeden Frame neu berechnen

    let mut current_x = 50.0;
    const X_SPACING: f32 = 200.0;
    const Y_POS: f32 = 100.0;

    // Sammle die erstellten VisNodes temporär, um darauf zugreifen zu können
    let mut temp_nodes: Vec<VisNode> = Vec::new();

    for (entity, species) in entity_query.iter() {
        let node_id = entity.index() as usize;
        let node = VisNode {
            id: node_id,
            entity: Some(entity),
            name: species.species.join(", "),
            position: Vec2::new(current_x, Y_POS),
            color: Color::from(GRAY), // Standard Bevy Color verwenden
        };
        temp_nodes.push(node); // Zum temporären Vektor hinzufügen
        current_x += X_SPACING;
    }

    // *** NEUER TEIL: Link hinzufügen (TESTWEISE) ***
    // Erstelle einen Link vom Output des ersten Nodes zum Input des zweiten Nodes
    if temp_nodes.len() >= 2 {
        let node0 = &temp_nodes[0];
        let node1 = &temp_nodes[1];

        // IDs basierend auf der Logik in context.rs generieren
        let start_pin_id = node0.id.wrapping_mul(PIN_ID_MULTIPLIER) + OUTPUT_PIN_OFFSET;
        let end_pin_id = node1.id.wrapping_mul(PIN_ID_MULTIPLIER) + INPUT_PIN_OFFSET;

        // Eindeutige ID für den Link (sehr einfacher Ansatz)
        let link_id = node0.id.wrapping_mul(1000) + node1.id; // Basis-ID

        let link = VisLink {
            id: link_id,
            start_pin_id: start_pin_id,
            end_pin_id: end_pin_id,
            color: Color::WHITE, // Standardfarbe für den Link
        };
        graph_data.links.push(link); // Füge den Link zur Ressource hinzu
    }
    // *** ENDE NEUER TEIL ***

    // Füge die temporär gesammelten Nodes zur finalen Ressource hinzu
    graph_data.nodes = temp_nodes;

    // Optionales Logging
    // bevy::log::info!(
    //     "Updated GraphUIData: {} nodes, {} links",
    //     graph_data.nodes.len(),
    //     graph_data.links.len()
    // );
}
