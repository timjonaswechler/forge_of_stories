use bevy_egui::egui;

use super::ui_link::LinkBezierData;
use super::NodesContext;

pub fn resolve_hover_state(context: &mut NodesContext) {
    if !context.state.interaction_state.mouse_in_canvas {
        context.frame_state.hovered_pin_index = None;
        context.frame_state.hovered_node_index = None;
        context.frame_state.hovered_link_idx = None;
        return;
    }

    // === Wiederherstellen: Pin Hover wieder aktivieren! ===
    resolve_occluded_pins(context);
    resolve_hovered_pin(context); // hovered_pin_index wird hier wieder korrekt gesetzt!

    // Node Hover deaktivieren wir aber weiterhin
    if context.frame_state.hovered_pin_index.is_none() {
        resolve_hovered_node(context); // Diese Zeile ändern
                                       // Node Hover State wird hier potentiell gesetzt
    } else {
        context.frame_state.hovered_node_index = None; // Oder hier auf None gesetzt, wenn Pin hovered
    }
    // *** ÄNDERUNG: Setze Node Hover IMMER auf None, NACHDEM er für die Pin-Prüfung benutzt wurde ***
    context.frame_state.hovered_node_index = None; // Node Hover hier deaktivieren

    // Link Hover (bleibt wie es war)
    if context.frame_state.hovered_pin_index.is_none()
        && context.frame_state.hovered_node_index.is_none()
    {
        // Node Hover ist jetzt immer None, Pin Hover aber potentiell Some!
        // Korrektur: Link nur hovern, wenn auch Pin nicht gehovert wird.
        resolve_hovered_link(context);
    } else {
        // Wenn Pin ODER (theoretisch) Node hovered -> kein Link Hover
        context.frame_state.hovered_link_idx = None;
    }
}

fn resolve_occluded_pins(context: &mut NodesContext) {
    context.frame_state.occluded_pin_indices.clear();
    let depth_stack = &context.state.node_depth_order;
    if depth_stack.len() < 2 {
        return;
    } // Nur nötig, wenn mind. 2 Nodes

    // Iteriere von unten nach oben (ausser dem obersten Node)
    for i in 0..(depth_stack.len() - 1) {
        if let Some(node_below) = context.nodes.get(&depth_stack[i]) {
            // Gehe alle Nodes *über* dem aktuellen Node durch
            for j in (i + 1)..depth_stack.len() {
                if let Some(node_above) = context.nodes.get(&depth_stack[j]) {
                    // Node Rect ist Grid Space -> Konvertieren zu Screen Space für den Test
                    let screen_rect_above = egui::Rect::from_min_max(
                        context.grid_space_to_screen_space(node_above.state.rect.min),
                        context.grid_space_to_screen_space(node_above.state.rect.max),
                    );

                    // Prüfe jeden Pin des unteren Nodes
                    for pin_id in &node_below.state.pin_indices {
                        if let Some(pin) = context.pins.get(pin_id) {
                            // Pin Position (Screen Space) wird im draw_pin gesetzt/aktualisiert
                            let pin_pos_screen = context.get_screen_space_pin_coordinates(pin);
                            if screen_rect_above.contains(pin_pos_screen) {
                                // Wenn der Pin vom oberen Node verdeckt wird, markieren
                                context.frame_state.occluded_pin_indices.push(*pin_id);
                            }
                        }
                    }
                }
            }
        }
    }
    // Dedupliziere die Liste (optional, falls ein Pin von mehreren Nodes verdeckt werden könnte)
    context.frame_state.occluded_pin_indices.sort_unstable();
    context.frame_state.occluded_pin_indices.dedup();
}

fn resolve_hovered_pin(context: &mut NodesContext) {
    context.frame_state.hovered_pin_index = None; // Start with no hovered pin
    let mut smallest_dist_sq = context.settings.style.pin_hover_radius.powi(2);
    let mouse_pos = context.state.interaction_state.mouse_pos;

    // Gehe alle aktuell im Frame existierenden Pins durch
    for (id, pin) in context.pins.iter() {
        // Überspringe, wenn der Pin von einem anderen Node verdeckt ist
        if context.frame_state.occluded_pin_indices.contains(id) {
            continue;
        }

        // pin.state.pos sollte die aktuelle Screen Position sein (wird in draw_pin gesetzt)
        let pin_pos_screen = context.get_screen_space_pin_coordinates(pin);
        let dist_sq = pin_pos_screen.distance_sq(mouse_pos);

        // Prüfe, ob dieser Pin näher ist als der bisher nächste gefundene
        if dist_sq < smallest_dist_sq {
            smallest_dist_sq = dist_sq;
            context.frame_state.hovered_pin_index = Some(*id); // Merke diesen Pin als gehovered
        }
    }
}

fn resolve_hovered_node(context: &mut NodesContext) {
    context.frame_state.hovered_node_index = None;
    // === KORREKTUR unused_mut / unused variable ===
    // let mut _max_depth = -1_isize; // Nicht mehr benötigt mit der neuen Logik

    for node_id in context.state.node_depth_order.iter().rev() {
        if let Some(node) = context.nodes.get(node_id) {
            let screen_rect = egui::Rect::from_min_max(
                context.grid_space_to_screen_space(node.state.rect.min),
                context.grid_space_to_screen_space(node.state.rect.max),
            );
            if screen_rect.contains(context.state.interaction_state.mouse_pos) {
                context.frame_state.hovered_node_index = Some(*node_id);
                return;
            }
        }
    }
}

fn resolve_hovered_link(context: &mut NodesContext) {
    context.frame_state.hovered_link_idx = None; // Reset
    let mut smallest_dist_sq = context.settings.style.link_hover_distance.powi(2);
    let mouse_pos = context.state.interaction_state.mouse_pos;

    for (id, link) in context.links.iter() {
        // Hole Start- und End-Pins (existieren sicher, da vorher geprüft)
        if let (Some(start_pin), Some(end_pin)) = (
            context.pins.get(&link.spec.start_pin_index),
            context.pins.get(&link.spec.end_pin_index),
        ) {
            // Ignoriere Link-Hover, wenn Maus über einem der beteiligten Pins ist
            // Dies wird bereits durch die Hover-Priorisierung (Pins > Links) abgedeckt.

            // Berechne Bezier basierend auf aktuellen Pin-Positionen
            let start_pos_screen = context.get_screen_space_pin_coordinates(start_pin);
            let end_pos_screen = context.get_screen_space_pin_coordinates(end_pin);
            let link_data = LinkBezierData::get_link_renderable(
                start_pos_screen,
                end_pos_screen,
                start_pin.spec.kind,
                context.settings.style.link_line_segments_per_length,
            );

            // Grobe Prüfung: Bounding Box
            let containing_rect = link_data.bezier.get_containing_rect_for_bezier_curve(
                context.settings.style.link_hover_distance, // Abstand zum Expandieren der Box
            );
            if containing_rect.contains(mouse_pos) {
                // Feine Prüfung: Abstand zur Kurve
                let dist_sq = link_data.get_distance_to_cubic_bezier_sq(&mouse_pos); // Nutze quadratischen Abstand
                if dist_sq < smallest_dist_sq {
                    smallest_dist_sq = dist_sq;
                    context.frame_state.hovered_link_idx = Some(*id);
                }
            }
        }
    }
}
