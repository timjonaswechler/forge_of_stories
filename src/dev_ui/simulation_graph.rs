//src/dev_ui/simulation_graph.rs

use bevy::color::palettes::basic::*;
use bevy::prelude::*;

use crate::genetics::components::SpeciesGenes;

use crate::ui_components::node_graph::{
    context::{GraphChange, NodesContext, PinType},
    resources::GraphUIData,
    VisLink, VisNode,
};

// === Wichtig: Diese Konstanten müssen mit denen in context.rs übereinstimmen! ===
const PIN_ID_MULTIPLIER: usize = 10;
const INPUT_PIN_OFFSET: usize = 0;
const OUTPUT_PIN_OFFSET: usize = 1;

pub fn provide_simulation_graph_data(
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

// *** NEUES SYSTEM ***
/// Verarbeitet Änderungen, die im NodesContext während des UI-Updates gesammelt wurden.
/// Reagiert z.B. auf das Erstellen von Links durch den Benutzer.
pub fn handle_graph_changes_system(
    mut _commands: Commands, // Wird oft benötigt, um Komponenten zu ändern/hinzuzufügen
    context: ResMut<NodesContext>, // Zugriff auf den Context, um Änderungen zu lesen
    // Query, um auf die Entities zuzugreifen, die durch die Pins repräsentiert werden.
    // Wir brauchen die VisNode-Daten, um von Node ID -> Entity ID zu kommen.
    graph_data: Res<GraphUIData>,
    // Optional: Query auf relevante Komponenten, falls das neue System sie direkt ändern soll
    // mut entity_query: Query<&mut Transform>, // Beispiel
) {
    // Hole alle Änderungen aus dem Kontext für diesen Frame.
    // .get_changes() gibt eine Referenz zurück, wir klonen sie hier,
    // damit wir den Kontext nicht mehr leihen, während wir auf Entities zugreifen.
    // Alternativ könnte man den Loop anders strukturieren.
    let changes = context.get_changes().clone();

    for change in changes {
        match change {
            GraphChange::NewLinkRequested(start_pin_id, end_pin_id) => {
                bevy::log::info!(
                    "System empfing NewLinkRequested: StartPin={}, EndPin={}",
                    start_pin_id,
                    end_pin_id
                );

                // --- Hier kommt die Logik deiner Simulation hin ---
                // 1. Finde die Node-IDs, die zu diesen Pin-IDs gehören.
                //    (Annahme: Input-Pin gehört zum Ziel-Node, Output-Pin zum Quell-Node)
                let maybe_nodes = find_nodes_for_pins(&context, start_pin_id, end_pin_id);

                if let Some((source_node_id, target_node_id)) = maybe_nodes {
                    bevy::log::info!(
                        "Zugehörige Node IDs: Source={}, Target={}",
                        source_node_id,
                        target_node_id
                    );

                    // 2. Finde die Bevy Entity IDs für diese Node IDs aus GraphUIData.
                    let maybe_entities =
                        find_entities_for_nodes(&graph_data, source_node_id, target_node_id);

                    if let Some((source_entity, target_entity)) = maybe_entities {
                        bevy::log::info!(
                            "Zugehörige Entities: Source={:?}, Target={:?}",
                            source_entity,
                            target_entity
                        );

                        // 3. Führe die Simulationslogik aus:
                        //    - Füge eine Komponente hinzu (z.B. `commands.entity(target_entity).insert(Parent(source_entity));`)
                        //    - Sende ein spezifisches Event (z.B. `relationship_added_events.send(...)`)
                        //    - Ändere Ressourcen etc.
                        println!(
                            "TODO: Implementiere Logik für Link zwischen {:?} und {:?}",
                            source_entity, target_entity
                        );
                        // BEISPIEL: Eine Parent-Komponente hinzufügen (wenn du eine hast)
                        // if let Ok(mut target_commands) = commands.get_entity(target_entity) {
                        //     target_commands.insert(Parent(source_entity)); // Nur ein Beispiel!
                        //     println!("Debug: Parent-Beziehung hinzugefügt.");
                        // }
                    } else {
                        bevy::log::warn!(
                            "Konnte Entities für Nodes ({}, {}) nicht finden.",
                            source_node_id,
                            target_node_id
                        );
                    }
                } else {
                    bevy::log::warn!(
                        "Konnte Nodes für Pins ({}, {}) nicht finden.",
                        start_pin_id,
                        end_pin_id
                    );
                }
            }
            GraphChange::LinkRemoved(_link_id) => {
                bevy::log::info!("System empfing LinkRemoved: {}", _link_id);
                // TODO: Implementiere Logik, um die Beziehung in der Simulation zu entfernen
            }
            GraphChange::NodeMoved(_node_id, _new_pos) => {
                // Normalerweise nicht simulationsrelevant, aber nützlich für Speichern/Laden von Layouts
                // bevy::log::trace!("System empfing NodeMoved: Node {} nach {:?}", node_id, new_pos);
                // TODO: Layout speichern, falls gewünscht
            }
            GraphChange::NodeRemoved(_node_id) => {
                bevy::log::info!("System empfing NodeRemoved: {}", _node_id);
                // TODO: Eventuell muss die Entity hier despawned werden? Oder die Entität bleibt bestehen?
            }
            // GraphChange::LinkCreated(start_pin_id, end_pin_id) -> Wird nicht mehr verwendet, stattdessen NewLinkRequested
            _ => {} // Andere Changes ignorieren (falls es noch welche gibt)
        }
    }

    // WICHTIG: Der Context sammelt Änderungen intern. Wir müssen sie *nicht* manuell löschen.
    // context.frame_state.graph_changes.clear(); // NICHT hier machen!
}

// --- Hilfsfunktionen (könnten auch in context.rs oder einem Helfermodul sein) ---

/// Findet die Node-IDs, zu denen die Pins gehören.
fn find_nodes_for_pins(
    context: &NodesContext,
    pin1_id: usize,
    pin2_id: usize,
) -> Option<(usize, usize)> {
    // Hole die kompletten Pin-Daten über die öffentliche Methode oder direkten Zugriff (falls public)
    // Annahme: Du hast jetzt eine pub fn get_pin(&self, pin_id: usize) -> Option<&Pin> in context.rs erstellt
    let pin1 = context.get_pin(pin1_id)?; // gibt &Pin zurück
    let pin2 = context.get_pin(pin2_id)?; // gibt &Pin zurück

    // *** KORREKTUR: Greife auf das 'state'-Feld für die Node-ID zu ***
    let node1_id = pin1.state.parent_node_idx;
    let node2_id = pin2.state.parent_node_idx;

    // *** KORREKTUR: Greife auf das 'spec'-Feld und das Feld 'kind' (nicht 'pin_type') zu ***
    let pin1_kind = pin1.spec.kind;

    // Bestimme Quelle (Output) und Ziel (Input)
    if pin1_kind == PinType::Output {
        Some((node1_id, node2_id))
    } else {
        // Annahme: Der andere Pin muss dann der Output-Pin sein.
        // Zusätzliche Sicherheit: Überprüfen, ob pin2.spec.kind == PinType::Output?
        Some((node2_id, node1_id))
    }
}

// --- find_entities_for_nodes bleibt wie zuvor ---
fn find_entities_for_nodes(
    graph_data: &GraphUIData,
    node1_id: usize,
    node2_id: usize,
) -> Option<(Entity, Entity)> {
    let entity1 = graph_data.nodes.iter().find(|n| n.id == node1_id)?.entity?;
    let entity2 = graph_data.nodes.iter().find(|n| n.id == node2_id)?.entity?;
    Some((entity1, entity2))
}
