//src/dev_ui/simulation_graph.rs

use crate::genetics::components::SpeciesGenes;
use crate::ui_components::node_graph::{
    context::{GraphChange, NodesContext, PinType}, // PinType importieren
    resources::GraphUIData,
    ui_data::{generate_pin_id, LogicalPinInfo, PinDirection, VisLink, VisNode}, // NEUE Imports
};

use bevy::color::palettes::css::*;
use bevy::prelude::*;
use bevy::utils::HashMap;

// === Wichtig: Diese Konstanten müssen mit denen in context.rs übereinstimmen! ===
const PIN_ID_MULTIPLIER: usize = 10;
const INPUT_PIN_OFFSET: usize = 0;
const OUTPUT_PIN_OFFSET: usize = 1;

#[derive(Component, Debug, Clone, Copy)] // Debug, Clone, Copy optional aber nützlich
pub struct FriendWith(pub Entity);

pub fn provide_simulation_graph_data(
    // Query für alle Entities, die wir als Nodes darstellen wollen
    node_entity_query: Query<(Entity, &SpeciesGenes)>,
    // Query, um Parent-Beziehungen zu finden
    parent_query: Query<(Entity, &Parent)>, // Query für Entities, die einen Parent *haben*
    mut graph_data: ResMut<GraphUIData>,
) {
    graph_data.nodes.clear();
    graph_data.links.clear();

    let mut current_x = 50.0;
    const X_SPACING: f32 = 250.0;
    const Y_POS: f32 = 100.0;

    // Temporäre Speicherung der Nodes und Mapping für schnellen Zugriff
    let mut temp_nodes: Vec<VisNode> = Vec::new();
    let mut entity_to_node_id_map: HashMap<Entity, usize> = HashMap::new();

    // --- Schritt 1: VisNodes erstellen und Mapping aufbauen ---
    for (entity, species) in node_entity_query.iter() {
        let node_id = entity.index() as usize; // Temporäre ID
        entity_to_node_id_map.insert(entity, node_id); // Füge zur Map hinzu

        // Erstelle die logischen Pins für diesen Node-Typ (Testweise Hardcoded)
        let mut logical_pins = Vec::new();
        logical_pins.push(LogicalPinInfo {
            identifier: "parent_out".to_string(),
            display_name: "Parent".to_string(),
            relation_type: "Family".to_string(),
            direction: PinDirection::Output,
        });
        logical_pins.push(LogicalPinInfo {
            identifier: "child_in".to_string(),
            display_name: "Child".to_string(),
            relation_type: "Family".to_string(),
            direction: PinDirection::Input,
        });
        logical_pins.push(LogicalPinInfo {
            identifier: "friend_bi".to_string(),
            display_name: "Friend".to_string(),
            relation_type: "Friendship".to_string(),
            direction: PinDirection::InOut,
        });

        let node = VisNode {
            id: node_id,
            entity: Some(entity),
            name: species.species.join(", "),
            position: Vec2::new(current_x, Y_POS),
            color: Color::from(GRAY),
            logical_pins,
        };
        temp_nodes.push(node);
        current_x += X_SPACING;
    }

    // --- Schritt 2: VisLinks für Parent/Child-Beziehungen erstellen ---
    // Iteriere über alle Entities, die eine Parent-Komponente haben
    for (child_entity, parent_component) in parent_query.iter() {
        let parent_entity = parent_component.get(); // Die Entity des Parents holen

        // Prüfe, ob *beide* (Parent und Child) in unserer Node-Map sind
        if let (Some(&parent_node_id), Some(&child_node_id)) = (
            entity_to_node_id_map.get(&parent_entity),
            entity_to_node_id_map.get(&child_entity), // Child Entity muss auch in der Map sein
        ) {
            // Generiere die spezifischen Pin-IDs für die Familienbeziehung
            let start_pin_id = generate_pin_id(parent_node_id, "parent_out");
            let end_pin_id = generate_pin_id(child_node_id, "child_in");

            // Eindeutige Link-ID generieren (z.B. XOR der Pin-IDs)
            let link_id = start_pin_id ^ end_pin_id;

            graph_data.links.push(VisLink {
                id: link_id,
                start_pin_id,
                end_pin_id,
                color: Color::from(ORANGE_RED),
            });
        }
    }

    // --- Schritt 3: Optionalen Test-Link für Freundschaft hinzufügen ---
    // Greift jetzt korrekt auf temp_nodes zu, *bevor* es verschoben wird.
    if temp_nodes.len() >= 2 {
        let node0_id = temp_nodes[0].id;
        let node1_id = temp_nodes[1].id;
        // Freundschafts-Pins verbinden
        let pin0_id = generate_pin_id(node0_id, "friend_bi");
        let pin1_id = generate_pin_id(node1_id, "friend_bi");
        let link_id = pin0_id ^ pin1_id; // Andere Methode für Link-ID?

        graph_data.links.push(VisLink {
            id: link_id,
            start_pin_id: pin0_id,
            end_pin_id: pin1_id,
            color: Color::from(LIGHT_CYAN),
        });
    }

    // --- Schritt 4: Finale Zuweisung der Nodes (nach allen Lesezugriffen) ---
    graph_data.nodes = temp_nodes; // Verschiebe die Nodes in die Ressource
}

/// Verarbeitet Änderungen, die im NodesContext während des UI-Updates gesammelt wurden.
/// Reagiert z.B. auf das Erstellen von Links durch den Benutzer.
pub fn handle_graph_changes_system(
    mut commands: Commands,
    context: Res<NodesContext>,
    graph_data: Res<GraphUIData>,
    parent_query: Query<&Parent>,
) {
    // Hole alle Änderungen aus dem Kontext für diesen Frame.
    let changes = context.get_changes().clone();

    for change in changes {
        match change {
            GraphChange::NewLinkRequested(start_pin_id, end_pin_id) => {
                bevy::log::info!(
                    "System received NewLinkRequested: StartPin={}, EndPin={}",
                    start_pin_id,
                    end_pin_id
                );

                // --- Implementierung für NewLinkRequested ---

                // 1. Finde Nodes und Entities für die beteiligten Pins
                // Annahme: Validator hat sichergestellt, dass Pins zu unterschiedl. Nodes gehören.
                let maybe_nodes = find_nodes_for_pins(&context, start_pin_id, end_pin_id);
                if let Some((source_node_id, target_node_id)) = maybe_nodes {
                    // 'source' ist der Node mit dem Output-Pin, 'target' der mit dem Input-Pin

                    let maybe_entities =
                        find_entities_for_nodes(&graph_data, source_node_id, target_node_id);

                    if let Some((source_entity, target_entity)) = maybe_entities {
                        bevy::log::info!(
                            "Found Entities for NewLink: Source(Output)={:?}, Target(Input)={:?}",
                            source_entity,
                            target_entity
                        );

                        // 2. Hole Pin-Spezifikationen, um relation_type und kind zu prüfen
                        // (context wird nur lesend benötigt, daher kein Borrowing-Problem hier)
                        let start_pin = context
                            .get_pin(start_pin_id)
                            .expect("Start Pin should exist for NewLinkRequested");
                        // PinType des Start-Pins bestimmt die Interpretation von source/target
                        let start_pin_kind = start_pin.spec.kind;

                        // 3. Entscheide basierend auf relation_type, was zu tun ist
                        // (Wir können relation_type vom start_pin nehmen, Validator stellte Match sicher)
                        match start_pin.spec.relation_type.as_str() {
                            "Family" => {
                                // --- Logik für Familienbeziehung ---

                                // Bestimme die tatsächlichen Parent/Child Entities basierend auf dem PinType
                                // des *ursprünglich geklickten* Start-Pins der UI-Interaktion.
                                let (actual_parent_entity, actual_child_entity) =
                                    if start_pin_kind == PinType::Output {
                                        // Start war der Parent-Pin (Output) -> source_entity ist Parent
                                        (source_entity, target_entity)
                                    } else {
                                        // Start war der Child-Pin (Input) -> target_entity ist Parent
                                        (target_entity, source_entity)
                                    };

                                bevy::log::debug!(
                                    "Family link requested: Parent={:?}, Child={:?}",
                                    actual_parent_entity,
                                    actual_child_entity
                                );

                                // Validierung gegen Selbst-Parenting
                                if actual_parent_entity == actual_child_entity {
                                    bevy::log::warn!(
                                        "Attempted to parent entity {:?} to itself. Skipping.",
                                        actual_child_entity
                                    );
                                    continue; // Nächste Änderung bearbeiten
                                }

                                // Prüfung auf existierenden Parent (optional, je nach Design)
                                if let Ok(existing_parent) = parent_query.get(actual_child_entity) {
                                    // Standardverhalten von Bevy: `set_parent` entfernt alte Beziehung automatisch.
                                    // Logge nur, dass überschrieben wird.
                                    bevy::log::warn!("Child entity {:?} already has parent {:?}. Overwriting with new parent {:?}.", actual_child_entity, existing_parent.get(), actual_parent_entity);
                                }

                                // Führe den Bevy Command aus
                                commands
                                    .entity(actual_child_entity)
                                    .set_parent(actual_parent_entity);
                                bevy::log::info!(
                                    "COMMAND (NewLink): Set Parent({:?}) for child {:?}",
                                    actual_parent_entity,
                                    actual_child_entity
                                );
                            } // Ende "Family"

                            "Friendship" => {
                                // --- Logik für Freundschaft ---
                                bevy::log::debug!(
                                    "Friendship link requested between {:?} and {:?}",
                                    source_entity,
                                    target_entity
                                );
                                // Verwende die Hilfsfunktion (Namen egal, da bidirektional)
                                add_friendship(&mut commands, source_entity, target_entity);
                            } // Ende "Friendship"

                            // Füge hier weitere `match`-Arme für andere relation_types hinzu
                            // z.B. "Rivalry", "Employer", etc.
                            _ => {
                                // Unbekannter/Nicht behandelter Beziehungstyp
                                bevy::log::warn!(
                                    "Unhandled relation type '{}' encountered in handle_graph_changes_system for NewLinkRequested.",
                                    start_pin.spec.relation_type
                                );
                            } // Ende _ (Default)
                        } // Ende match relation_type
                    } else {
                        bevy::log::warn!(
                            "NewLinkRequested: Could not find entities for node IDs {} and {}.",
                            source_node_id,
                            target_node_id
                        );
                        continue; // Nächste Änderung bearbeiten
                    }
                } else {
                    bevy::log::warn!(
                        "NewLinkRequested: Could not find nodes for pin IDs {} and {}.",
                        start_pin_id,
                        end_pin_id
                    );
                    continue; // Nächste Änderung bearbeiten
                }
            }

            GraphChange::LinkRemoved(_link_id) => {
                bevy::log::info!("System empfing LinkRemoved: {}", _link_id);
                // TODO: Implementiere Logik, um die Beziehung in der Simulation zu entfernen
            }
            GraphChange::LinkModified {
                link_id,
                new_start_pin_id,
                new_end_pin_id,
            } => {
                bevy::log::info!(
                    "System empfing LinkModified: Link ID={}, New Start Pin={}, New End Pin={}",
                    link_id,
                    new_start_pin_id,
                    new_end_pin_id
                );

                // --- Hier kommt die komplexere Logik für das Ändern einer Beziehung ---
                // 1. Finde die Entities für die NEUEN Pins (new_start_pin_id, new_end_pin_id).
                // Nutze find_nodes_for_pins und find_entities_for_nodes wie bei NewLinkRequested.
                // Annahme: new_start ist Output, new_end ist Input.
                let maybe_nodes = find_nodes_for_pins(&context, new_start_pin_id, new_end_pin_id);
                if let Some((new_source_node_id, new_target_node_id)) = maybe_nodes {
                    let maybe_entities = find_entities_for_nodes(
                        &graph_data,
                        new_source_node_id,
                        new_target_node_id,
                    );
                    if let Some((new_source_entity, new_target_entity)) = maybe_entities {
                        bevy::log::info!(
                            "LinkModified: Found new entities: Source={:?}, Target={:?}",
                            new_source_entity,
                            new_target_entity
                        );

                        // 2. Finde heraus, welche Beziehung der alten `link_id` entsprach UND
                        //    welche Entities *ursprünglich* verbunden waren.
                        // SCHWIERIG: Die UI `link_id` ist temporär und hat keine direkte Entsprechung in Bevy.
                        // MÖGLICHE ANSÄTZE:
                        //    a) GraphUIData könnte ein Mapping `ui_link_id -> (start_entity, end_entity, relation_type)` enthalten. (Aufwändig zu pflegen)
                        //    b) Wir könnten versuchen, aus den neuen Entities/Pins rückzuschließen, welche Beziehung geändert wurde.
                        //       Z.B. wenn der neue Link ein "Parent"-"Child"-Link ist, muss das `new_target_entity` vorher
                        //       entweder keinen Parent gehabt haben oder einen anderen.
                        //    c) Im Event `LinkModified` die alten Pin-IDs/Entity-IDs mitschicken? (Änderung in context.rs nötig)

                        // TODO: Implementiere das Finden der alten Beziehung und der alten Entities.

                        // 3. Entferne die ALTE Beziehung in der Simulation.
                        // Z.B. wenn es ein Parent-Link war:
                        // `commands.entity(alte_child_entity).remove_parent();`
                        // TODO: Implementiere das Entfernen der alten Beziehung.

                        // 4. Füge die NEUE Beziehung hinzu, basierend auf den NEUEN Pins.
                        // Nutze die Logik von `NewLinkRequested`: Prüfe die neuen Pin-Namen/Typen.
                        let start_pin = context
                            .get_pin(new_start_pin_id)
                            .expect("New start pin should exist");
                        let end_pin = context
                            .get_pin(new_end_pin_id)
                            .expect("New end pin should exist");

                        if start_pin.spec.name == "Parent" && end_pin.spec.name == "Child" {
                            // Setze NEUEN Parent für new_target_entity
                            if new_source_entity == new_target_entity {
                                /* warn */
                                continue;
                            }
                            if let Ok(existing_parent) = parent_query.get(new_target_entity) { /* warn: überschreibt */
                            }
                            commands
                                .entity(new_target_entity)
                                .set_parent(new_source_entity);
                            bevy::log::info!(
                                "COMMAND (LinkModified): Set Parent({:?}) for child {:?}",
                                new_source_entity,
                                new_target_entity
                            );
                        } else if start_pin.spec.name == "Friend" && end_pin.spec.name == "Friend" {
                            add_friendship(&mut commands, new_source_entity, new_target_entity);
                        } else {
                            bevy::log::warn!(
                                "Unhandled pin combination for LinkModified: '{}' <-> '{}'",
                                start_pin.spec.name,
                                end_pin.spec.name
                            );
                        }
                        // TODO: Stelle sicher, dass die alte Beziehung korrekt entfernt wurde!
                    } else {
                        bevy::log::warn!("LinkModified: Could not find entities for new pins.");
                    }
                } else {
                    bevy::log::warn!("LinkModified: Could not find nodes for new pins.");
                }
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

fn add_friendship(commands: &mut Commands, entity1: Entity, entity2: Entity) {
    if entity1 == entity2 {
        bevy::log::warn!(
            "Attempted to create friendship between an entity and itself: {:?}",
            entity1
        );
        return;
    }
    // Füge die Komponenten hinzu
    commands.entity(entity1).insert(FriendWith(entity2));
    commands.entity(entity2).insert(FriendWith(entity1));
    // Log-Nachricht hier statt im Aufrufer, für zentrale Info
    bevy::log::info!(
        "Friendship Component added/updated between {:?} and {:?}",
        entity1,
        entity2
    );
}
