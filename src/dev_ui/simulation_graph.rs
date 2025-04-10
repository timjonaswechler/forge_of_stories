//src/dev_ui/simulation_graph.rs
use crate::genetics::components::SpeciesGenes;
use crate::ui_components::node_graph::{
    context::{GraphChange, NodesContext, PinType},
    resources::{DetailDisplayData, GraphUIData},
    ui_data::{generate_pin_id, LogicalPinInfo, PinDirection, VisLink, VisNode},
};

use bevy::prelude::*;
use bevy::utils::HashMap;
// Entferne: use bevy_egui::egui::{self, Color32}; // <- Diese Zeile entfernen oder anpassen
use bevy_egui::egui; // Nur egui behalten, wenn nur das gebraucht wird

#[derive(Component, Debug, Clone, Copy)] // Debug, Clone, Copy optional aber nützlich
pub struct FriendWith(pub Entity);

#[derive(Resource, Default, Debug, Clone)]
pub struct SelectedNodeDetails {
    pub selected_entity: Option<Entity>,
    // Optional: Hier könnten wir bereits aufbereitete Daten speichern,
    // z.B. Name, Komponentenwerte als Strings etc.
    // pub component_details: HashMap<String, String>,
}

pub fn provide_simulation_graph_data(
    node_entity_query: Query<(Entity, &SpeciesGenes)>,
    parent_query: Query<(Entity, &Parent)>,
    friend_query: Query<(Entity, &FriendWith)>,
    mut graph_data: ResMut<GraphUIData>,
) {
    graph_data.nodes.clear();
    graph_data.links.clear();

    let family_color = Color::srgb(1.0, 0.4, 0.0);
    let friendship_color = Color::srgb(0.8, 1.0, 1.0);

    let mut current_y = 50.0;
    const Y_SPACING: f32 = 250.0;
    const X_POS: f32 = 100.0;

    let get_header_color_for_species = |species_list: &[String]| -> Color {
        match species_list.first().map(|s| s.as_str()) {
            Some("Mensch") => Color::srgb(1.0, 1.0, 0.0),
            Some("Elf") => Color::srgb(0.565, 0.933, 0.565),
            Some("Ork") => Color::srgb(0.647, 0.165, 0.165),
            _ => Color::srgb(0.5, 0.5, 0.5),
        }
    };

    let mut temp_nodes: Vec<VisNode> = Vec::new();
    let mut entity_to_node_id_map: HashMap<Entity, usize> = HashMap::new();

    // --- VORLAGEN für Logische Pins (könnten auch Konstanten sein) ---
    //     Diese werden jetzt *innerhalb* der Schleife referenziert/geklont.
    let family_out_pin_template = LogicalPinInfo {
        identifier: "family_out".to_string(),
        display_name: "Parent".to_string(),
        relation_type: "Family".to_string(),
        direction: PinDirection::Output,
    };
    let family_in_pin_template = LogicalPinInfo {
        identifier: "family_in".to_string(),
        display_name: "Child".to_string(),
        relation_type: "Family".to_string(),
        direction: PinDirection::Input,
    };
    let friend_bi_pin_template = LogicalPinInfo {
        identifier: "friend_bi".to_string(),
        display_name: "Friend".to_string(),
        relation_type: "Friendship".to_string(),
        direction: PinDirection::InOut,
    };
    // --- ENDE VORLAGEN ---

    // --- Schritt 1: VisNodes erstellen (mit dynamischer Pin-Zuweisung) ---
    for (entity, species) in node_entity_query.iter() {
        let node_id = entity.index() as usize;
        entity_to_node_id_map.insert(entity, node_id);
        let header_color = get_header_color_for_species(&species.species);

        // --- Erstelle die Pin-Liste für DIESEN Node ---
        let mut current_logical_pins: Vec<LogicalPinInfo> = Vec::new();

        // --- FÜGE PINS BASIEREND AUF REGELN HINZU (aktuell: immer alle) ---
        // Regel 1: Alle können potentiell Eltern sein
        current_logical_pins.push(family_out_pin_template.clone());

        // Regel 2: Alle können potentiell Kinder haben
        current_logical_pins.push(family_in_pin_template.clone());

        // Regel 3: Alle können potentiell Freunde haben/werden
        current_logical_pins.push(friend_bi_pin_template.clone());

        // *** HIER wäre der Ort für bedingte Logik: ***
        // if has_component_xyz(entity, &some_query) {
        //     current_logical_pins.push(some_other_pin_template.clone());
        // }
        // *********************************************

        // --- Erstelle den VisNode mit der generierten Pin-Liste ---
        let node = VisNode {
            id: node_id,
            entity: Some(entity),
            name: species.species.join(", "),
            position: Vec2::new(X_POS, current_y),
            color: header_color,
            logical_pins: current_logical_pins, // <-- DYNAMISCH ZUGWIESEN
        };
        temp_nodes.push(node);
        current_y += Y_SPACING;
    } // --- Ende der Node-Erstellungsschleife ---

    // --- Schritt 2: VisLinks für Parent/Child-Beziehungen erstellen ---
    // (Dieser Block bleibt unverändert, er nutzt die Pin-Identifier)
    bevy::log::debug!("--- Running Parent Query ---");
    for (child_entity, parent_component) in parent_query.iter() {
        let parent_entity = parent_component.get();
        bevy::log::debug!(
            "Parent Query Found: Child={:?}, Parent={:?}",
            child_entity,
            parent_entity
        );
        if let (Some(&parent_node_id), Some(&child_node_id)) = (
            entity_to_node_id_map.get(&parent_entity),
            entity_to_node_id_map.get(&child_entity),
        ) {
            // Pin IDs werden weiterhin über die konstanten Identifier gefunden
            let start_pin_id = generate_pin_id(parent_node_id, "family_out");
            let end_pin_id = generate_pin_id(child_node_id, "family_in");
            let link_id = start_pin_id ^ end_pin_id;

            graph_data.links.push(VisLink {
                id: link_id,
                start_pin_id,
                end_pin_id,
                color: family_color,
            });
            bevy::log::debug!(" -> Creating VisLink for Parent relation");
        } else {
            bevy::log::debug!(" -> Skipping VisLink (Parent or Child not in node map)");
        }
    }
    bevy::log::debug!("--- Finished Parent Query ---");

    // --- Schritt 3: VisLinks für Freundschafts-Beziehungen erstellen ---
    // (Dieser Block bleibt auch unverändert)
    for (entity1, friend_component) in friend_query.iter() {
        let entity2 = friend_component.0; // Die Entity, mit der entity1 befreundet ist

        // Verhindere doppelte Links (verarbeite nur eine Richtung der Beziehung)
        // Wir vergleichen die Indizes, um sicherzustellen, dass wir jedes Paar nur einmal hinzufügen.
        if entity1.index() < entity2.index() {
            // <--- Prüfpunkt 1: Duplikat-Vermeidung
            // Prüfe, ob *beide* befreundeten Entities als Nodes im Graphen vorhanden sind
            if let (Some(&node1_id), Some(&node2_id)) = (
                // <--- Prüfpunkt 2: Existenz beider Nodes
                entity_to_node_id_map.get(&entity1),
                entity_to_node_id_map.get(&entity2),
            ) {
                // Generiere die spezifischen Pin-IDs für die Freundschaft
                // <--- Prüfpunkt 3: Konsistente Pin-Identifier
                let pin1_id = generate_pin_id(node1_id, "friend_bi");
                let pin2_id = generate_pin_id(node2_id, "friend_bi");

                // Eindeutige Link-ID generieren
                let link_id = pin1_id ^ pin2_id; // <--- Prüfpunkt 4: Link-ID Generierung

                // Erstelle den VisLink
                graph_data.links.push(VisLink {
                    id: link_id,
                    start_pin_id: pin1_id,   // <--- Zuweisung Pin 1
                    end_pin_id: pin2_id,     // <--- Zuweisung Pin 2
                    color: friendship_color, // NEU: Verwende die definierte Freundschaftsfarbe
                });
                bevy::log::trace!(
                    "Added Friendship VisLink between {:?} and {:?}",
                    entity1,
                    entity2
                );
            } else {
                // Einer oder beide Freunde sind nicht im Graphen dargestellt
                bevy::log::trace!(
                    "Skipping Friendship VisLink: {:?} or {:?} not in node map.",
                    entity1,
                    entity2
                );
            }
        }
    } // Ende der neuen Friend-Link Schleife

    // --- Schritt 4: Finale Zuweisung der Nodes (bleibt unverändert) ---
    graph_data.nodes = temp_nodes;
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

            GraphChange::LinkRemoved {
                start_pin_id,
                end_pin_id,
            } => {
                bevy::log::info!(
                    "System received LinkRemoved: StartPin={}, EndPin={}", // Angepasst
                    start_pin_id,
                    end_pin_id
                );

                // --- Implementierung für LinkRemoved ---

                // 1. Finde Nodes und Entities für die (ehemals) verbundenen Pins
                // WICHTIG: Die Pins existieren möglicherweise nicht mehr im context, wenn der Node
                // gleichzeitig gelöscht wurde. Wir versuchen trotzdem, die Nodes/Entities zu finden,
                // falls nur der Link gelöscht wurde.
                // Wir brauchen auch die PinSpecs, um den relation_type zu kennen.
                // Wenn die Pins nicht mehr da sind, können wir nichts tun.

                if let (Some(start_pin), Some(end_pin)) =
                    (context.get_pin(start_pin_id), context.get_pin(end_pin_id))
                {
                    let start_node_id = start_pin.state.parent_node_idx;
                    let end_node_id = end_pin.state.parent_node_idx;

                    let maybe_entities =
                        find_entities_for_nodes(&graph_data, start_node_id, end_node_id);

                    if let Some((start_entity, end_entity)) = maybe_entities {
                        bevy::log::info!(
                            "Found Entities for LinkRemoved: Start(Output)={:?}, End(Input)={:?}",
                            start_entity,
                            end_entity
                        );

                        // 2. Bestimme den Typ der zu entfernenden Beziehung anhand von relation_type
                        match start_pin.spec.relation_type.as_str() {
                            "Family" => {
                                // --- Logik zum Entfernen der Familienbeziehung ---
                                // Annahme: `start_entity` war Parent, `end_entity` war Child (da start_pin Output war).
                                // Überprüfe sicherheitshalber, ob `end_entity` tatsächlich `start_entity` als Parent *hatte*.
                                if let Ok(parent_component) = parent_query.get(end_entity) {
                                    if parent_component.get() == start_entity {
                                        commands.entity(end_entity).remove_parent();
                                        bevy::log::info!(
                                             "COMMAND (LinkRemoved): Removed Parent({:?}) from child {:?}",
                                             start_entity, end_entity
                                         );
                                    } else {
                                        // Sollte nicht passieren, wenn der Graph konsistent ist
                                        bevy::log::warn!("LinkRemoved(Family): Child {:?} had a different parent {:?} than expected ({:?}). Doing nothing.", end_entity, parent_component.get(), start_entity);
                                    }
                                } else {
                                    bevy::log::warn!("LinkRemoved(Family): Child {:?} had no Parent component to remove.", end_entity);
                                }
                            } // Ende "Family"

                            "Friendship" => {
                                // --- Logik zum Entfernen der Freundschaft ---
                                bevy::log::debug!(
                                    "Attempting to remove Friendship between {:?} and {:?}",
                                    start_entity,
                                    end_entity
                                );
                                // Entferne die Komponenten von *beiden* Entities
                                commands.entity(start_entity).remove::<FriendWith>(); // Annahme: FriendWith(end_entity) war hier
                                commands.entity(end_entity).remove::<FriendWith>(); // Annahme: FriendWith(start_entity) war hier
                                bevy::log::info!(
                                     "COMMAND (LinkRemoved): Removed Friendship components between {:?} and {:?}",
                                     start_entity, end_entity
                                 );
                            } // Ende "Friendship"

                            // Füge hier weitere `match`-Arme für andere relation_types hinzu
                            _ => {
                                bevy::log::warn!(
                                    "Unhandled relation type '{}' encountered in handle_graph_changes_system for LinkRemoved.",
                                    start_pin.spec.relation_type
                                );
                            } // Ende _ (Default)
                        } // Ende match relation_type
                    } else {
                        bevy::log::warn!("LinkRemoved: Could not find entities for node IDs {} and {}. Maybe nodes were removed simultaneously?", start_node_id, end_node_id);
                    }
                } else {
                    // Wichtigster Fall: Einer oder beide Pins/Nodes wurden gleichzeitig gelöscht.
                    // In diesem Fall müssen wir nichts extra für den Link tun, das Löschen des Nodes (oder der Beziehung durch Parent-Update) kümmert sich darum.
                    bevy::log::debug!("LinkRemoved: Could not find one or both pins (IDs {} and {}). Assuming relation was removed implicitly by node removal or other means.", start_pin_id, end_pin_id);
                }
            } // Ende LinkRemoved

            // Ersetze den gesamten Arm für LinkModified
            GraphChange::LinkModified {
                new_start_pin_id,
                new_end_pin_id,
                old_start_pin_id, // Jetzt verfügbar
                old_end_pin_id,   // Jetzt verfügbar
            } => {
                bevy::log::info!(
                    "System received LinkModified: OldPins={}->{}, NewPins={}->{}",
                    old_start_pin_id,
                    old_end_pin_id,
                    new_start_pin_id,
                    new_end_pin_id
                );

                // === Schritt A: ALTE Beziehung entfernen (falls möglich) ===

                // Versuche, die Entities der ALTEN Beziehung zu finden
                if let Some((old_source_node, old_target_node)) =
                    find_nodes_for_pins(&context, old_start_pin_id, old_end_pin_id)
                {
                    if let Some((old_source_entity, old_target_entity)) =
                        find_entities_for_nodes(&graph_data, old_source_node, old_target_node)
                    {
                        // Hole den Spec des ALTEN Start-Pins, um den Typ zu bestimmen
                        if let Some(old_start_pin_spec) = context.get_pin(old_start_pin_id) {
                            match old_start_pin_spec.spec.relation_type.as_str() {
                                "Family" => {
                                    // Alte Beziehung war Family: old_source war Parent, old_target war Child
                                    if let Ok(parent_comp) = parent_query.get(old_target_entity) {
                                        if parent_comp.get() == old_source_entity {
                                            commands.entity(old_target_entity).remove_parent();
                                            bevy::log::info!("COMMAND (LinkModified-Cleanup): Removed OLD Parent({:?}) from child {:?}", old_source_entity, old_target_entity);
                                        } else {
                                            bevy::log::warn!("LinkModified-Cleanup(Family): Old child {:?} had different parent {:?} than expected {:?}.", old_target_entity, parent_comp.get(), old_source_entity);
                                        }
                                    } else {
                                        bevy::log::warn!("LinkModified-Cleanup(Family): Old child {:?} had no parent to remove.", old_target_entity);
                                    }
                                }
                                "Friendship" => {
                                    bevy::log::debug!("LinkModified-Cleanup(Friendship): Removing OLD Friendship between {:?} and {:?}", old_source_entity, old_target_entity);
                                    commands.entity(old_source_entity).remove::<FriendWith>();
                                    commands.entity(old_target_entity).remove::<FriendWith>();

                                    bevy::log::info!("COMMAND (LinkModified-Cleanup): Removed OLD Friendship components between {:?} and {:?}", old_source_entity, old_target_entity);
                                }
                                _ => {
                                    bevy::log::warn!("LinkModified-Cleanup: Unhandled relation type '{}' for old link.", old_start_pin_spec.spec.relation_type);
                                }
                            }
                        } else {
                            bevy::log::warn!(
                                "LinkModified-Cleanup: Could not get spec for old start pin {}.",
                                old_start_pin_id
                            );
                        }
                    } else {
                        bevy::log::warn!("LinkModified-Cleanup: Could not find entities for old nodes {} and {}.", old_source_node, old_target_node);
                    }
                } else if old_start_pin_id != usize::MAX {
                    // Nur warnen, wenn keine Platzhalter-IDs
                    bevy::log::warn!("LinkModified-Cleanup: Could not find nodes for old pins {} and {}. Old relation might persist.", old_start_pin_id, old_end_pin_id);
                }

                // === Schritt B: NEUE Beziehung hinzufügen (nur wenn alte entfernt wurde ODER keine alte bekannt war) ===
                // Diese Bedingung ist optional, je nachdem, ob das Erstellen der neuen fehlschlagen soll,
                // wenn das Aufräumen scheitert. Aktuell: Füge die neue immer hinzu.
                // if old_relation_removed || old_start_pin_id == usize::MAX { // Wenn alte weg oder unbekannt
                // Logik von oben, leicht angepasst:
                let maybe_nodes = find_nodes_for_pins(&context, new_start_pin_id, new_end_pin_id);
                if let Some((new_source_node_id, new_target_node_id)) = maybe_nodes {
                    let maybe_entities = find_entities_for_nodes(
                        &graph_data,
                        new_source_node_id,
                        new_target_node_id,
                    );
                    if let Some((new_source_entity, new_target_entity)) = maybe_entities {
                        bevy::log::info!("LinkModified: Applying NEW connection: Source(Output)={:?}, Target(Input)={:?}", new_source_entity, new_target_entity);
                        let start_pin = context
                            .get_pin(new_start_pin_id)
                            .expect("New start pin should exist for LinkModified");
                        let _end_pin = context // Ist _end_pin, da momentan ungenutzt
                            .get_pin(new_end_pin_id)
                            .expect("New end pin should exist for LinkModified");

                        // PinType des *ursprünglich* geklickten Start-Pins der *neuen* Verbindung.
                        let new_start_pin_kind = start_pin.spec.kind;

                        // 4. Entscheide basierend auf relation_type der *neuen* Verbindung
                        match start_pin.spec.relation_type.as_str() {
                            "Family" => {
                                // --- Logik für Änderung einer Familienbeziehung ---
                                let (actual_parent_entity, actual_child_entity) =
                                    if new_start_pin_kind == PinType::Output {
                                        (new_source_entity, new_target_entity)
                                    } else {
                                        (new_target_entity, new_source_entity)
                                    };

                                bevy::log::debug!(
                        "Family link modified: Attempting New Parent={:?}, New Child={:?}",
                        actual_parent_entity,
                        actual_child_entity
                    );

                                // Validierung gegen Selbst-Parenting
                                if actual_parent_entity == actual_child_entity {
                                    bevy::log::warn!("LinkModified(Family): Attempted to parent entity {:?} to itself. Action skipped.", actual_child_entity);
                                    // Hier kein continue mehr, damit wir ggf. Logs danach sehen
                                } else {
                                    // Prüfung auf existierenden Parent. set_parent entfernt diesen automatisch.
                                    if let Ok(existing_parent) =
                                        parent_query.get(actual_child_entity)
                                    {
                                        if existing_parent.get() != actual_parent_entity {
                                            bevy::log::warn!("LinkModified(Family): Child entity {:?} already had parent {:?}. Overwriting with new parent {:?}.", actual_child_entity, existing_parent.get(), actual_parent_entity);
                                        } else {
                                            bevy::log::debug!("LinkModified(Family): Child entity {:?} already had parent {:?}, likely reconnecting same link.", actual_child_entity, existing_parent.get());
                                        }
                                    }

                                    // Führe den Bevy Command aus
                                    commands
                                        .entity(actual_child_entity)
                                        .set_parent(actual_parent_entity);
                                    bevy::log::info!(
                                        "COMMAND (LinkModified): Set Parent({:?}) for child {:?}",
                                        actual_parent_entity,
                                        actual_child_entity
                                    );
                                } // Ende else (nicht self-parenting)
                            } // Ende "Family"

                            "Friendship" => {
                                // --- Logik für Änderung einer Freundschaft ---
                                // TODO: Die alte Freundschaft sollte hier entfernt werden. Benötigt komplexere Event-Daten (alte Entity).
                                bevy::log::warn!("Handling LinkModified for Friendship: Cannot determine and remove the OLD friendship based only on new pin IDs. Adding the new friendship only. Old 'FriendWith' components might remain.");

                                bevy::log::debug!(
                                    "Friendship link modified: Creating new connection between {:?} and {:?}",
                                    new_source_entity,
                                    new_target_entity
                                );

                                // Füge die neue Freundschaft hinzu.
                                add_friendship(&mut commands, new_source_entity, new_target_entity);
                            } // Ende "Friendship"

                            _ => {
                                bevy::log::warn!(
                        "Unhandled relation type '{}' encountered in handle_graph_changes_system for LinkModified.",
                        start_pin.spec.relation_type
                    );
                            } // Ende _ (Default)
                        } // Ende match relation_type
                    } else {
                        // Fehlerfall: Entities für Nodes nicht gefunden
                        bevy::log::error!(
                 "LinkModified: Failed to find entities! Source Node ID {}, Target Node ID {}. Graph data might be stale or entities despawned.",
                 new_source_node_id, new_target_node_id
             );
                        // Hier KEIN continue
                    }
                } else {
                    // Fehlerfall: Nodes für Pins nicht gefunden
                    bevy::log::error!(
                        "LinkModified: Failed to find nodes! Start Pin ID {}, End Pin ID {}",
                        new_start_pin_id,
                        new_end_pin_id
                    );
                    // Hier KEIN continue
                }
            } // Ende LinkModified

            GraphChange::NodeMoved(_node_id, _new_pos) => {
                // Normalerweise nicht simulationsrelevant, aber nützlich für Speichern/Laden von Layouts
                // bevy::log::trace!("System empfing NodeMoved: Node {} nach {:?}", node_id, new_pos);
                // TODO: Layout speichern, falls gewünscht
            }
            GraphChange::NodeRemoved(node_id) => {
                bevy::log::info!("System received NodeRemoved: Node UI ID={}", node_id);

                // --- Implementierung für NodeRemoved ---

                // 1. Finde die Entity, die zu dieser UI-Node-ID gehört.
                // Wir durchsuchen die GraphUIData, die den Zustand *vor* dem Löschen im Context widerspiegelt.
                let maybe_entity = graph_data
                    .nodes
                    .iter()
                    .find(|n| n.id == node_id)
                    .and_then(|n| n.entity);

                if let Some(entity_to_despawn) = maybe_entity {
                    bevy::log::info!(
                        "Found Entity for NodeRemoved: {:?}. Despawning...",
                        entity_to_despawn
                    );

                    // 2. Despawne die Entity rekursiv.
                    //    `despawn_recursive` entfernt auch alle Kinder dieser Entity.
                    //    Wenn die Beziehungen (Parent/Child, FriendWith) korrekt implementiert sind,
                    //    sollten Komponenten wie FriendWith auf nicht mehr existierende Entities zeigen,
                    //    was (hoffentlich) kein Problem darstellt, aber evtl. später aufgeräumt werden muss.
                    //    Bevy entfernt auch automatisch die Parent-Beziehung zu Kindern, wenn der Parent despawned wird.
                    commands.entity(entity_to_despawn).despawn_recursive();

                    // Log, dass der Befehl abgesetzt wurde
                    bevy::log::info!(
                        "COMMAND (NodeRemoved): Despawn Recursive for entity {:?}",
                        entity_to_despawn
                    );
                } else {
                    // Dies könnte passieren, wenn die GraphUIData aus irgendeinem Grund nicht synchron ist
                    // oder der Node bereits auf andere Weise entfernt wurde.
                    bevy::log::warn!(
                        "NodeRemoved: Could not find entity matching UI Node ID {}. Unable to despawn.",
                        node_id
                    );
                }
            }
            // GraphChange::LinkCreated(start_pin_id, end_pin_id) -> Wird nicht mehr verwendet, stattdessen NewLinkRequested
            _ => {} // Andere Changes ignorieren (falls es noch welche gibt)
        }
    }

    // WICHTIG: Der Context sammelt Änderungen intern. Wir müssen sie *nicht* manuell löschen.
    // context.frame_state.graph_changes.clear(); // NICHT hier machen!
}

pub fn update_selected_node_details(
    nodes_context: Res<NodesContext>,
    mut graph_data: ResMut<GraphUIData>,
    query_name: Query<&Name>,
    query_genes: Query<&SpeciesGenes>,
    query_parent: Query<&Parent>,
    query_friend: Query<&FriendWith>,
) {
    let selected_nodes = nodes_context.get_selected_nodes();

    if selected_nodes.len() == 1 {
        let selected_node_id = selected_nodes[0];
        let maybe_entity = graph_data
            .nodes
            .iter()
            .find(|n| n.id == selected_node_id)
            .and_then(|n| n.entity);

        if let Some(entity) = maybe_entity {
            // Prüfen, ob sich Auswahl geändert hat oder Details noch nicht gesetzt sind
            let should_update = nodes_context.is_node_just_selected()
                || graph_data.selected_node_details_display.is_none()
                || graph_data
                    .selected_node_details_display
                    .as_ref()
                    .map_or(false, |d| d.title != format!("Entity: {:?}", entity)); // Grober Check

            if should_update {
                // Bereite die Daten für die Anzeige vor
                let mut display_data = DetailDisplayData {
                    title: format!("Entity: {:?}", entity),
                    properties: Vec::new(),
                };

                // Komponenten abfragen und zur Liste hinzufügen
                if let Ok(name) = query_name.get(entity) {
                    display_data
                        .properties
                        .push(("Name".to_string(), name.to_string()));
                }
                if let Ok(genes) = query_genes.get(entity) {
                    display_data
                        .properties
                        .push(("Genes".to_string(), format!("{:?}", genes.species)));
                }
                if let Ok(parent) = query_parent.get(entity) {
                    display_data
                        .properties
                        .push(("Parent".to_string(), format!("{:?}", parent.get())));
                }
                if let Ok(friend) = query_friend.get(entity) {
                    display_data
                        .properties
                        .push(("Friend".to_string(), format!("{:?}", friend.0)));
                }
                // ... Weitere Komponenten ...

                graph_data.selected_node_details_display = Some(display_data);
                bevy::log::debug!("Updated detail display data for {:?}", entity);
            }
        } else {
            // Entity nicht gefunden (oder nicht im Graphen) -> Details löschen
            if graph_data.selected_node_details_display.is_some() {
                graph_data.selected_node_details_display = None;
                bevy::log::debug!(
                    "Cleared detail display data: Entity for selected node not found."
                );
            }
        }
    } else {
        // Keine oder mehrere Nodes ausgewählt -> Details löschen
        if graph_data.selected_node_details_display.is_some() {
            graph_data.selected_node_details_display = None;
            bevy::log::debug!("Cleared detail display data: Selection count != 1.");
        }
    }
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
