use bevy::log;
use bevy::math::Vec2;
use bevy_egui::egui;
use derivative::Derivative;

use super::{
    context::{GraphChange, LinkValidationCallback, NodesContext},
    ui_link::{Link, LinkBezierData, LinkSpec, LinkState},
    ui_pin::{AttributeFlags, PinType},
};

#[derive(Derivative)]
#[derivative(Debug, Default)]
pub(crate) struct InteractionState {
    pub(crate) mouse_pos: egui::Pos2,
    pub(crate) mouse_delta: egui::Vec2,
    pub(crate) left_mouse_clicked: bool,
    pub(crate) left_mouse_released: bool,
    pub(crate) alt_mouse_clicked: bool,
    pub(crate) left_mouse_dragging: bool,
    pub(crate) alt_mouse_dragging: bool,
    pub(crate) mouse_in_canvas: bool,
    pub(crate) link_detatch_with_modifier_click: bool,
    pub(crate) delete_pressed: bool,
}
impl InteractionState {
    pub fn update(
        &self,
        io: &egui::InputState,
        opt_hover_pos: Option<egui::Pos2>,
        emulate_three_button_mouse: Modifier,
        link_detatch_with_modifier_click: Modifier,
        alt_mouse_button: Option<egui::PointerButton>,
    ) -> Self {
        let mut new_state = Self::default();
        if let Some(mouse_pos) = opt_hover_pos {
            new_state.mouse_in_canvas = true;
            new_state.mouse_pos = mouse_pos;
        } else {
            new_state.mouse_in_canvas = false;
            new_state.mouse_pos = self.mouse_pos;
        }
        new_state.mouse_delta = new_state.mouse_pos - self.mouse_pos;
        let primary_down = io.pointer.primary_down();
        new_state.left_mouse_released =
            (self.left_mouse_clicked || self.left_mouse_dragging) && !primary_down;
        new_state.left_mouse_dragging =
            (self.left_mouse_clicked || self.left_mouse_dragging) && primary_down;
        new_state.left_mouse_clicked =
            primary_down && !new_state.left_mouse_dragging && !self.left_mouse_clicked;
        let alt_btn_down = alt_mouse_button.is_some_and(|btn| io.pointer.button_down(btn));
        let emulate_active = emulate_three_button_mouse.is_active(&io.modifiers) && primary_down;
        let alt_down = alt_btn_down || emulate_active;
        new_state.alt_mouse_dragging =
            (self.alt_mouse_clicked || self.alt_mouse_dragging) && alt_down;
        new_state.alt_mouse_clicked =
            alt_down && !new_state.alt_mouse_dragging && !self.alt_mouse_clicked;
        new_state.link_detatch_with_modifier_click =
            link_detatch_with_modifier_click.is_active(&io.modifiers);
        new_state.delete_pressed = io.key_pressed(egui::Key::Delete);
        new_state
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) enum ClickInteractionType {
    None,
    Node,
    Link,
    LinkCreation,
    Panning,
    BoxSelection,
}
impl Default for ClickInteractionType {
    fn default() -> Self {
        Self::None
    }
}
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub(crate) enum LinkCreationType {
    #[default]
    Standard,
    FromDetach,
} // Default zu Standard

#[derive(Derivative, Debug, Clone)] // Clone hinzugefügt
#[derivative(Default)]
pub(crate) struct ClickInteractionStateLinkCreation {
    pub(crate) start_pin_idx: Option<usize>, // Option, falls Detach fehlschlägt
    pub(crate) end_pin_index: Option<usize>,
    pub(crate) modifying_link_id: Option<usize>, // NEU: ID des Links, der modifiziert wird
    #[derivative(Default(value = "LinkCreationType::default()"))]
    pub(crate) link_creation_type: LinkCreationType,
}

#[derive(Derivative, Debug, Clone)] // Only Clone hinzugefügt
#[derivative(Default)]
pub(crate) struct ClickInteractionState {
    pub(crate) link_creation: ClickInteractionStateLinkCreation,
    #[derivative(Default(value = "egui::Rect::ZERO"))]
    pub(crate) box_selection: egui::Rect,
}

#[derive(Derivative, Debug, Default)]
pub struct IO {
    #[derivative(Default(value = "Modifier::None"))]
    pub emulate_three_button_mouse: Modifier,
    #[derivative(Default(value = "Modifier::None"))]
    pub link_detatch_with_modifier_click: Modifier,
    #[derivative(Default(value = "Some(egui::PointerButton::Middle)"))]
    pub alt_mouse_button: Option<egui::PointerButton>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Modifier {
    Alt,
    Ctrl,
    Shift,
    Command,
    #[default]
    None,
}
impl Modifier {
    fn is_active(&self, mods: &egui::Modifiers) -> bool {
        match self {
            Modifier::Alt => mods.alt,
            Modifier::Ctrl => mods.ctrl,
            Modifier::Shift => mods.shift,
            Modifier::Command => mods.command,
            Modifier::None => mods.is_none() && !mods.any(), // Strengere Prüfung auf *gar keine* Modifier
        }
    }
}

#[derive(Derivative, Default, Debug, Clone)] // Clone hinzugefügt
pub(crate) struct ElementStateChange {
    pub(crate) link_started: bool,
    pub(crate) link_dropped: bool,
    pub(crate) link_created: bool, // Erfolgreich an Pin gesnapped und Maus losgelassen
}
impl ElementStateChange {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

pub(crate) fn process_clicks(context: &mut NodesContext) {
    if !context.state.interaction_state.mouse_in_canvas {
        return;
    }

    // --- Linksklick ---
    if context.state.interaction_state.left_mouse_clicked {
        if let Some(pin_idx) = context.frame_state.hovered_pin_index {
            // Klick auf Pin -> Versuche, Link-Erstellung zu starten
            begin_link_creation(context, pin_idx);
        } else if let Some(node_idx) = context.frame_state.hovered_node_index {
            // Klick auf Node -> Starte Node-Selektion/-Drag
            // Verhindere Start, wenn gerade ein Pin im *selben* Node geklickt wurde (falls `active_pin` genutzt wird)
            // if self.frame_state.active_pin.map_or(true, |p_id| self.pins.get(&p_id).map_or(true, |p| p.state.parent_node_idx != node_idx)) {
            begin_node_selection(context, node_idx);
            // }
        } else if let Some(link_idx) = context.frame_state.hovered_link_idx {
            // Klick auf Link -> Starte Link-Interaktion (Select/Detach)
            begin_link_interaction(context, link_idx);
        } else {
            // Klick ins Leere -> Starte Box-Selektion
            begin_canvas_interaction(context, false);
        }
    }
    // --- Alternativklick (z.B. Mitte, Rechts) ---
    else if context.state.interaction_state.alt_mouse_clicked {
        // Klick ins Leere mit Alt -> Starte Panning
        if context.frame_state.hovered_node_index.is_none()
            && context.frame_state.hovered_pin_index.is_none()
            && context.frame_state.hovered_link_idx.is_none()
        {
            begin_canvas_interaction(context, true);
        }
        // Optional: Alt-Klick auf Node/Pin/Link für Kontextmenü etc.
    }
}

fn begin_link_interaction(context: &mut NodesContext, link_id: usize) {
    // Prüfen, ob ein Pin gehovert wird UND Detach-Flag gesetzt ist
    let pin_is_hovered_and_detachable = context
        .frame_state
        .hovered_pin_index
        .map(|pin_id| {
            // Prüft ob der Pin zum Link gehört und detachbar ist
            (context.frame_state.hovered_pin_flags
                & AttributeFlags::EnableLinkDetachWithDragClick as usize
                != 0)
                && context.links.get(&link_id).map_or(false, |l| {
                    l.spec.start_pin_index == pin_id || l.spec.end_pin_index == pin_id
                })
        })
        .unwrap_or(false);

    // Prüfen, ob Detach-Modifier aktiv ist
    let detach_with_modifier = context
        .state
        .interaction_state
        .link_detatch_with_modifier_click;

    // Fall 1: Detach durch Klick auf Pin mit Flag
    if pin_is_hovered_and_detachable {
        if let Some(pin_id) = context.frame_state.hovered_pin_index {
            begin_link_detach(context, link_id, pin_id);
            return;
        }
    }

    // Fall 2: Detach durch Klick + Modifier auf Link
    if detach_with_modifier && !pin_is_hovered_and_detachable {
        // Verhindert Detach, wenn schon durch Pin getriggert
        if let Some(link) = context.links.get(&link_id) {
            // Finde den näheren Pin zur Mausposition für den Detach-Start
            if let (Some(start_pin), Some(end_pin)) = (
                context.pins.get(&link.spec.start_pin_index),
                context.pins.get(&link.spec.end_pin_index),
            ) {
                let pos_start = context.get_screen_space_pin_coordinates(start_pin);
                let pos_end = context.get_screen_space_pin_coordinates(end_pin);
                let dist_start_sq =
                    pos_start.distance_sq(context.state.interaction_state.mouse_pos);
                let dist_end_sq = pos_end.distance_sq(context.state.interaction_state.mouse_pos);

                let closest_pin_idx = if dist_start_sq < dist_end_sq {
                    link.spec.start_pin_index
                } else {
                    link.spec.end_pin_index
                };
                begin_link_detach(context, link_id, closest_pin_idx);
                return;
            }
        }
    }

    // Fall 3: Standard Link Selection
    begin_link_selection(context, link_id);
}

fn begin_link_creation(context: &mut NodesContext, pin_id: usize) {
    // Nur starten, wenn noch keine Interaktion läuft und der Pin existiert
    if context.state.click_interaction_type == ClickInteractionType::None {
        if let Some(pin) = context.pins.get(&pin_id) {
            // Prüfen, ob Link-Erstellung von diesem Pin-Typ erlaubt ist (typischerweise Output)
            // Oder ob spezielle Flags gesetzt sind. Hier vereinfacht: Outputs können starten.
            if pin.spec.kind == PinType::Output
                || (pin.spec.flags & AttributeFlags::EnableLinkCreationOnSnap as usize != 0)
            {
                // Beispiel für Flag-Nutzung
                // Initialisiere den LinkCreation State korrekt
                context.state.click_interaction_state.link_creation =
                    ClickInteractionStateLinkCreation {
                        start_pin_idx: Some(pin_id),
                        end_pin_index: None,     // Kein End-Pin beim Start
                        modifying_link_id: None, // Es wird kein Link modifiziert, also None
                        link_creation_type: LinkCreationType::Standard,
                    };
                // Setze Interaction Type erst, wenn State korrekt ist
                context.state.click_interaction_type = ClickInteractionType::LinkCreation;
                context.frame_state.element_state_change.link_started = true;
            } else {
                bevy::log::trace!(
                    "Link creation not allowed from pin {:?} (Type: {:?})",
                    pin_id,
                    pin.spec.kind
                );
            }
        }
    }
}

fn begin_link_selection(context: &mut NodesContext, link_id: usize) {
    if context.state.click_interaction_type == ClickInteractionType::None
        || (context.state.click_interaction_type == ClickInteractionType::Link
            && !context.state.selected_link_indices.contains(&link_id))
    {
        context.state.click_interaction_type = ClickInteractionType::Link;
        context.state.selected_node_indices.clear(); // Nur Links selektieren
        context.state.selected_link_indices.clear();
        context.state.selected_link_indices.push(link_id);
    } else if context.state.click_interaction_type == ClickInteractionType::Link {
        // Optional: Bei Klick auf bereits selektierten Link -> Deselection?
        // context.state.selected_link_indices.clear();
        // context.state.click_interaction_type = ClickInteractionType::None;
    }
}

fn begin_node_selection(context: &mut NodesContext, node_id: usize) {
    if context.state.click_interaction_type != ClickInteractionType::None {
        return;
    } // Nur wenn keine andere Interaktion läuft

    context.state.click_interaction_type = ClickInteractionType::Node;

    // Check for multi-selection (Shift-Click, Ctrl-Click, etc.) - Nicht implementiert hier
    // Standardverhalten: Clear und Select
    if !context.state.selected_node_indices.contains(&node_id) {
        context.state.selected_node_indices.clear();
        context.state.selected_link_indices.clear(); // Auch Links deselektieren
        context.state.selected_node_indices.push(node_id);
        context.frame_state.just_selected_node = true; // Markiere für Detail View Update etc.
    } else {
        // Node war bereits selektiert. Optional: Bei erneutem Klick nicht wieder auswählen oder Modifikatoren prüfen.
        context.frame_state.just_selected_node = false; // Nicht *gerade eben* selektiert
    }

    // Node in der Tiefenordnung nach oben bringen
    if let Some(pos) = context
        .state
        .node_depth_order
        .iter()
        .position(|x| *x == node_id)
    {
        // Wenn gefunden, entfernen und ans Ende (oben) schieben
        let id_to_move = context.state.node_depth_order.remove(pos);
        context.state.node_depth_order.push(id_to_move);
    } else {
        // Sollte nicht passieren, wenn node_id aus einer gültigen Quelle stammt
        bevy::log::warn!(
            "Node {:?} nicht in Tiefenordnung gefunden beim Selektieren.",
            node_id
        );
        context.state.node_depth_order.push(node_id); // Vorsichtshalber hinzufügen
    }
}

// Startet Link-Erstellung vom *anderen* Pin, nachdem der Original-Link entfernt wurde
fn begin_link_detach(context: &mut NodesContext, link_id: usize, detach_pin_idx: usize) {
    if context.state.click_interaction_type == ClickInteractionType::None {
        if let Some(link_to_detach) = context.links.get(&link_id) {
            // Nur lesen

            let other_pin_id = if link_to_detach.spec.start_pin_index == detach_pin_idx {
                link_to_detach.spec.end_pin_index
            } else {
                link_to_detach.spec.start_pin_index
            };

            if context.pins.contains_key(&other_pin_id) {
                let modifying_id = Some(link_id); // Hole die ID vor dem Verschieben
                context.state.click_interaction_state.link_creation =
                    ClickInteractionStateLinkCreation {
                        start_pin_idx: Some(other_pin_id),
                        end_pin_index: None,
                        modifying_link_id: modifying_id, // Setze die ID
                        link_creation_type: LinkCreationType::FromDetach,
                    };
                context.state.click_interaction_type = ClickInteractionType::LinkCreation;
                context.frame_state.element_state_change.link_started = true;
            } else {
                log::warn!(
                    "Link detach: Anderer Pin ({:?}) nicht gefunden.",
                    other_pin_id
                );
                // Interaktion wird nicht gestartet, da der andere Pin fehlt
            }
        } else {
            log::warn!("Link detach: Link {:?} nicht gefunden.", link_id);
        }
    }
}

fn begin_canvas_interaction(context: &mut NodesContext, start_panning: bool) {
    if context.state.click_interaction_type == ClickInteractionType::None {
        if start_panning {
            context.state.click_interaction_type = ClickInteractionType::Panning;
        } else {
            // Starte Box Selection
            context.state.click_interaction_type = ClickInteractionType::BoxSelection;
            context.state.selected_node_indices.clear(); // Auswahl löschen beim Starten
            context.state.selected_link_indices.clear();
            context.state.click_interaction_state.box_selection = egui::Rect::from_min_max(
                context.state.interaction_state.mouse_pos, // Start an der Mausposition
                context.state.interaction_state.mouse_pos,
            );
        }
    }
}

pub(crate) fn translate_selected_nodes(context: &mut NodesContext) {
    // Nur ausführen, wenn gezogen wird UND eine Node-Interaktion läuft
    if context.state.interaction_state.left_mouse_dragging
        && context.state.click_interaction_type == ClickInteractionType::Node
    {
        // Delta in Screen Space
        let delta_screen = context.state.interaction_state.mouse_delta;
        // Delta in Grid Space (einfache Subtraktion der Canvas-Origin reicht nicht, Panning muss raus!)
        // Da Panning *während* des Zugs konstant bleibt, ist Screen Delta = Grid Delta
        let delta_grid = delta_screen;

        if delta_grid.length_sq() > 0.0 {
            // Nur wenn es eine Bewegung gab
            let mut changes = Vec::new(); // Sammle Änderungen für Events

            // Gehe alle selektierten Node IDs durch
            for node_id in &context.state.selected_node_indices {
                // Mutable Borrow auf den Node, wenn er existiert
                if let Some(node) = context.nodes.get_mut(node_id) {
                    // Prüfe, ob Node beweglich ist (optionales Flag)
                    if node.state.draggable {
                        // Update die Node *Spezifikation* (Ursprung im Grid Space)
                        node.spec.origin += delta_grid;
                        // Update den *Zustand* (Rect im Grid Space)
                        node.state.rect = node.state.rect.translate(delta_grid);

                        // Sammle die Änderung für das Event
                        changes.push((
                            *node_id,
                            Vec2::new(node.spec.origin.x, node.spec.origin.y), // Aktuelle Grid-Position
                        ));
                    }
                }
            }

            // Sende alle gesammelten Events
            for (id, pos) in changes {
                context
                    .frame_state
                    .graph_changes
                    .push(GraphChange::NodeMoved(id, pos));
            }
        }
    }
}

pub(crate) fn box_selector_update_selection(context: &mut NodesContext) {
    // Nicht mehr &self, braucht &mut self
    // Nur ausführen, wenn die Box-Selektion aktiv ist
    if context.state.click_interaction_type != ClickInteractionType::BoxSelection {
        return;
    }

    // Update die Endposition der Box zur aktuellen Mausposition
    context.state.click_interaction_state.box_selection.max =
        context.state.interaction_state.mouse_pos;
    let box_rect_screen = context.state.click_interaction_state.box_selection;

    // Normalisiere das Rechteck (min sollte immer links oben sein)
    let normalized_box_screen = egui::Rect::from_min_max(
        box_rect_screen.min.min(box_rect_screen.max),
        box_rect_screen.min.max(box_rect_screen.max),
    );

    // Wandle das Screen-Rect der Box in Grid-Space um für den Node-Vergleich
    let box_rect_grid = egui::Rect::from_min_max(
        context.screen_space_to_grid_space(normalized_box_screen.min),
        context.screen_space_to_grid_space(normalized_box_screen.max),
    );

    let previous_selected_nodes = context.state.selected_node_indices.clone(); // Merke vorherige Auswahl für Reihenfolge
    context.state.selected_node_indices.clear(); // Leere aktuelle Auswahl
    context.state.selected_link_indices.clear(); // Auch Links leeren

    // --- Node Selection ---
    for (id, node) in context.nodes.iter() {
        // Prüfe Überschneidung im Grid Space
        if box_rect_grid.intersects(node.state.rect) {
            context.state.selected_node_indices.push(*id);
        }
    }
    // Optional: Behalte ursprüngliche Selektionsreihenfolge bei, falls Nodes wieder selektiert werden
    context.state.selected_node_indices.sort_by_key(|id| {
        previous_selected_nodes
            .iter()
            .position(|&old_id| old_id == *id)
            .unwrap_or(usize::MAX)
    });

    // --- Link Selection ---
    for (id, link) in context.links.iter() {
        // Prüfe, ob *beide* Pins des Links existieren
        if let (Some(start_pin), Some(end_pin)) = (
            context.pins.get(&link.spec.start_pin_index),
            context.pins.get(&link.spec.end_pin_index),
        ) {
            // Prüfe, ob *beide* Nodes der Pins existieren
            if context.nodes.contains_key(&start_pin.state.parent_node_idx)
                && context.nodes.contains_key(&end_pin.state.parent_node_idx)
            {
                // Berechne Pin-Positionen im Screen Space
                let p1_screen = context.get_screen_space_pin_coordinates(start_pin);
                let p2_screen = context.get_screen_space_pin_coordinates(end_pin);

                // Prüfe Überschneidung des Links mit der *Screen*-Box
                if rectangle_overlaps_link(
                    context,
                    &normalized_box_screen,
                    &p1_screen,
                    &p2_screen,
                    start_pin.spec.kind,
                ) {
                    context.state.selected_link_indices.push(*id);
                }
            }
        }
    }
    // box_rect_screen // Wird nicht mehr zurückgegeben
}

fn rectangle_overlaps_link(
    context: &NodesContext,
    rect: &egui::Rect,   // Box im Screen Space
    start: &egui::Pos2,  // Start-Pin im Screen Space
    end: &egui::Pos2,    // End-Pin im Screen Space
    start_type: PinType, // Typ des Start-Pins für Bezier-Richtung
) -> bool {
    // Schnelle Prüfung: Bounding Box des Links gegen Bounding Box des Rects
    let mut link_bounding_box = egui::Rect::from_two_pos(*start, *end);
    link_bounding_box = link_bounding_box.union(*rect); // Vergrößere, um sicher zu sein

    if !rect.intersects(link_bounding_box.expand(5.0)) {
        // Kleiner Puffer
        return false;
    }

    // Genauere Prüfung (falls Bounding Boxen überlappen)
    if rect.contains(*start) || rect.contains(*end) {
        return true; // Einer der Endpunkte ist in der Box
    }

    // Berechne die Bezier-Kurve des Links
    let link_data = LinkBezierData::get_link_renderable(
        *start,
        *end,
        start_type,
        context.settings.style.link_line_segments_per_length,
    );

    // Verwende die Hilfsfunktion der Bezier-Daten zur Überlappungsprüfung
    link_data.rectangle_overlaps_bezier(rect)
}

pub(crate) fn click_interaction_update(
    context: &mut NodesContext,
    ui: &mut egui::Ui,
    link_validator: &LinkValidationCallback,
) {
    match context.state.click_interaction_type {
        ClickInteractionType::LinkCreation => {
            // Hole ID des ggf. modifizierten Links
            let modifying_link_id = context
                .state
                .click_interaction_state
                .link_creation
                .modifying_link_id;
            let start_pin_id = context
                .state
                .click_interaction_state
                .link_creation
                .start_pin_idx
                .unwrap_or(usize::MAX);
            // ... Prüfe ob start_pin_id gültig ...

            let mut snapped_pin_id: Option<usize> = None;
            if let Some(hovered_pin_id) = context.frame_state.hovered_pin_index {
                let duplicate_link_id = find_duplicate_link(context, start_pin_id, hovered_pin_id);

                // *** VALIDIERUNG über Callback ***
                // Rufe `should_link_snap_to_pin` auf, das *jetzt* den Validator verwendet
                if should_link_snap_to_pin(
                    context,
                    start_pin_id,
                    hovered_pin_id,
                    duplicate_link_id,
                    link_validator,
                ) {
                    snapped_pin_id = Some(hovered_pin_id);
                }
            }
            context
                .state
                .click_interaction_state
                .link_creation
                .end_pin_index = snapped_pin_id;

            if context.state.interaction_state.left_mouse_released {
                if let Some(end_pin_id) = snapped_pin_id {
                    // --- Erfolgreich an Pin gesnapped ---

                    // Bestimme korrekte finale Pin-Rollen (Output -> Input)
                    let (output_pin_id, input_pin_id) = {
                        // Hole die Pin-Typen der beteiligten Pins
                        let start_pin_kind = context
                            .pins
                            .get(&start_pin_id)
                            .map_or(PinType::None, |p| p.spec.kind);
                        // end_pin_id ist hier im Scope gültig!
                        let end_pin_kind = context
                            .pins
                            .get(&end_pin_id)
                            .map_or(PinType::None, |p| p.spec.kind);

                        // Gehe davon aus, dass der Validator sichergestellt hat, dass einer Input und einer Output ist.
                        // Falls der Start-Pin ein Output ist ODER der End-Pin ein Input ist, ist die Reihenfolge start -> end korrekt.
                        if start_pin_kind == PinType::Output || end_pin_kind == PinType::Input {
                            (start_pin_id, end_pin_id)
                        } else {
                            // Andernfalls war der Start-Pin der Input, also drehe die Reihenfolge.
                            (end_pin_id, start_pin_id)
                        }
                    };

                    // Prüfe, ob wir gerade einen bestehenden Link modifiziert haben
                    if let Some(original_link_id) = modifying_link_id {
                        // --- Fall 1: Link wurde modifiziert (Detach & Re-Connect) ---

                        // --- NEUE LOGIK: Holen der alten Pins ---
                        // Entferne temporär, um an die Spec zu kommen
                        if let Some(removed_link_data) = context.links.remove(&original_link_id) {
                            let old_start_pin = removed_link_data.spec.start_pin_index;
                            let old_end_pin = removed_link_data.spec.end_pin_index;

                            // Sende das Event mit ALLEN Pin-Infos
                            context
                                .frame_state
                                .graph_changes
                                .push(GraphChange::LinkModified {
                                    new_start_pin_id: output_pin_id,
                                    new_end_pin_id: input_pin_id,
                                    old_start_pin_id: old_start_pin, // Alte Pins hinzufügen
                                    old_end_pin_id: old_end_pin,
                                });
                            bevy::log::info!(
                                "GraphChange::LinkModified sent. Old Pins: {}->{}, New Pins: {} -> {}",
                                old_start_pin, old_end_pin, output_pin_id, input_pin_id
                            );

                            // Den Link brauchen wir nicht wieder einzufügen,
                            // da er im nächsten Frame vom Provider sowieso
                            // neu erstellt wird (oder auch nicht, wenn die Beziehung weg ist).
                            // Das `remove` hier genügt.
                        } else {
                            // Fallback, falls Link schon weg war
                            bevy::log::error!("LinkModification failed: Could not find link with old UI ID {} to remove and get old pins.", original_link_id);
                            // Sende Event nur mit neuen Pins (wie vorher)
                            context
                                .frame_state
                                .graph_changes
                                .push(GraphChange::LinkModified {
                                    new_start_pin_id: output_pin_id,
                                    new_end_pin_id: input_pin_id,
                                    old_start_pin_id: usize::MAX, // Platzhalter
                                    old_end_pin_id: usize::MAX,   // Platzhalter
                                });
                        }
                        // ---------------------------------------
                    } else {
                        // --- Fall 2: Brandneuer Link wurde erstellt ---
                        // BERECHNE ID mit XOR der Pin IDs
                        let new_link_id = output_pin_id ^ input_pin_id;

                        // Erstelle die Spezifikation für den neuen Link
                        let new_link_spec = LinkSpec {
                            id: new_link_id, // XOR ID verwenden
                            start_pin_index: output_pin_id,
                            end_pin_index: input_pin_id,
                            style: Default::default(),
                        };
                        // Erstelle den Zustand für den neuen Link
                        let new_link_state = LinkState {
                            style: context
                                .settings
                                .style
                                .format_link(new_link_spec.style.clone()),
                            shape: Some(ui.painter().add(egui::Shape::Noop)),
                        };
                        // Füge den neuen Link zum internen Zustand hinzu
                        if let Some(_existing_link) = context.links.insert(
                            new_link_id,
                            Link {
                                spec: new_link_spec,
                                state: new_link_state,
                            },
                        ) {
                            bevy::log::warn!("Overwrote existing link with same XOR ID {} during new link creation!", new_link_id);
                        }

                        // Sende das Erstellt-Event
                        context
                            .frame_state
                            .graph_changes
                            .push(GraphChange::NewLinkRequested(output_pin_id, input_pin_id));
                        bevy::log::info!(
                            "GraphChange::NewLinkRequested sent for pins: {} -> {}",
                            output_pin_id,
                            input_pin_id
                        );
                    } // Ende else (Neuer Link)
                    context.frame_state.element_state_change.link_created = true;
                } else {
                    // --- Ins Leere gedropped ---
                    // Prüfe, ob wir im Modify-Modus waren
                    let _was_modifying = modifying_link_id.is_some(); // Hole den Wert *bevor* er zurückgesetzt wird

                    context.frame_state.element_state_change.link_dropped = true;
                }

                // Interaktion beenden, egal ob erfolgreich oder nicht
                context.state.click_interaction_type = ClickInteractionType::None;
                // Wichtig: Auch den spezifischen Zustand der LinkCreation-Interaktion zurücksetzen!
                // Dies löscht modifying_link_id, start_pin_idx etc. für die nächste Interaktion.
                context.state.click_interaction_state.link_creation = Default::default();

            // Sollte None sein
            } else {
                // --- Ins Leere gedropped ---
                // Wenn ein Link modifiziert wurde (modifying_link_id war Some), passiert hier nichts dauerhaftes.
                // Der Link wurde nie aus self.links entfernt und wird im nächsten Frame einfach wieder normal gezeichnet.
                // Wenn ein neuer Link erstellt wurde (modifying_link_id war None), wird er ebenfalls nicht hinzugefügt.
                // Setze nur das Flag für die Außenwelt (falls benötigt)
                context.frame_state.element_state_change.link_dropped = true;
                // log::warn!("Link detach: Link {:?} nicht gefunden.", link_id); // <-- DIESE ZEILE LÖSCHEN
            }
        }

        ClickInteractionType::BoxSelection => {
            box_selector_update_selection(context); // Aktualisiert die Auswahl basierend auf der Box
            if context.state.interaction_state.left_mouse_released {
                // Box-Selektion beendet
                context.state.click_interaction_type = ClickInteractionType::None;
                // Bringt neu ausgewählte Nodes in der Tiefenordnung nach oben (optional)
                let s = context.state.selected_node_indices.clone();
                context.state.node_depth_order.retain(|id| !s.contains(id));
                context.state.node_depth_order.extend(s);
            }
        }
        ClickInteractionType::Node => {
            translate_selected_nodes(context); // Bewegt ausgewählte Nodes
            if context.state.interaction_state.left_mouse_released {
                // Node-Drag beendet
                context.state.click_interaction_type = ClickInteractionType::None;
            }
        }
        ClickInteractionType::Link => {
            // Keine Aktion während des Haltens/Ziehens eines Links (nur bei Release interessant)
            if context.state.interaction_state.left_mouse_released {
                // Link-Selektion "beendet" (keine spezifische Aktion hier nötig)
                context.state.click_interaction_type = ClickInteractionType::None;
            }
        }

        ClickInteractionType::Panning => {
            // Nur wenn die Alt-Taste gehalten und gezogen wird
            if context.state.interaction_state.alt_mouse_dragging {
                context.state.panning += context.state.interaction_state.mouse_delta;
            // Update Panning
            }
            // Panning beenden, wenn Alt-Taste losgelassen wird (oder nicht mehr gezogen)
            // Prüfung auf !dragging UND !clicked könnte nötig sein, je nach egui Event Timing
            else if !context.state.interaction_state.alt_mouse_dragging
                && !context.state.interaction_state.alt_mouse_clicked
            {
                context.state.click_interaction_type = ClickInteractionType::None;
            }
        }
        ClickInteractionType::None => {
            // Keine aktive Interaktion
        }
    }
}

fn find_duplicate_link(
    context: &NodesContext,
    start_pin_id: usize,
    end_pin_id: usize,
) -> Option<usize> {
    // Normalisiere die Pin-IDs für den Vergleich (Reihenfolge egal)
    let (p1, p2) = if start_pin_id < end_pin_id {
        (start_pin_id, end_pin_id)
    } else {
        (end_pin_id, start_pin_id)
    };

    // Suche nach einem existierenden Link mit denselben (normalisierten) Pin-IDs
    for (id, link) in context.links.iter() {
        let (l1, l2) = if link.spec.start_pin_index < link.spec.end_pin_index {
            (link.spec.start_pin_index, link.spec.end_pin_index)
        } else {
            (link.spec.end_pin_index, link.spec.start_pin_index)
        };

        if p1 == l1 && p2 == l2 {
            return Some(*id); // Duplikat gefunden, gib seine ID zurück
        }
    }

    None // Kein Duplikat gefunden
}

fn should_link_snap_to_pin(
    context: &NodesContext,
    start_pin_id: usize,
    hovered_pin_id: usize,
    duplicate_link: Option<usize>,
    link_validator: &LinkValidationCallback, // NEU
) -> bool {
    // 1. Allgemeine Prüfungen (Pins existieren, nicht derselbe Node, kein Duplikat)
    let Some(start_pin) = context.pins.get(&start_pin_id) else {
        return false;
    };
    let Some(end_pin) = context.pins.get(&hovered_pin_id) else {
        return false;
    };
    if start_pin.state.parent_node_idx == end_pin.state.parent_node_idx {
        return false;
    }
    if duplicate_link.is_some() {
        return false;
    }
    // PinType Gleichheit wird jetzt im Callback geprüft, da InOut<->InOut erlaubt sein soll

    // === 2. ANWENDUNGSSPEZIFISCHE VALIDIERUNG via Callback ===
    // Rufe die übergebene Funktion auf. Wir übergeben die PinSpecs und den Kontext.
    if !link_validator(&start_pin.spec, &end_pin.spec, context) {
        return false;
    }

    // Wenn alle Prüfungen bestanden
    true
}

pub(crate) fn handle_delete(context: &mut NodesContext) {
    let mut links_to_remove_ids: Vec<usize> =
        context.state.selected_link_indices.drain(..).collect();
    let nodes_to_remove_ids: Vec<usize> = context.state.selected_node_indices.drain(..).collect();
    let mut pins_to_remove_ids = Vec::new();

    // 1. Finde alle Pins der zu löschenden Nodes
    for node_id in &nodes_to_remove_ids {
        if let Some(node) = context.nodes.get(node_id) {
            pins_to_remove_ids.extend(node.state.pin_indices.iter().copied());
        }
    }

    // 2. Finde alle Links, die mit den zu löschenden Pins verbunden sind
    // Wir müssen die Links hier nicht vorab finden, da wir sie direkt beim Iterieren entfernen können.
    // Allerdings brauchen wir eine Kopie der Link-IDs, falls wir die von zu löschenden Nodes auch entfernen wollen.
    let mut implicitly_removed_links = Vec::new();
    if !nodes_to_remove_ids.is_empty() {
        // Nur suchen, wenn Nodes entfernt werden
        for (link_id, link) in context.links.iter() {
            if pins_to_remove_ids.contains(&link.spec.start_pin_index)
                || pins_to_remove_ids.contains(&link.spec.end_pin_index)
            {
                if !links_to_remove_ids.contains(link_id) {
                    // Verhindere Duplikate
                    implicitly_removed_links.push(*link_id);
                }
            }
        }
        links_to_remove_ids.extend(implicitly_removed_links);
        links_to_remove_ids.sort_unstable();
        links_to_remove_ids.dedup();
    }

    // 3. Entferne die Links (ausgewählte + implizite) aus dem internen State und sammle Events
    for link_id in &links_to_remove_ids {
        // WICHTIG: Erst den Link holen, DANN entfernen!
        if let Some(removed_link) = context.links.remove(link_id) {
            // Sende das NEUE Event mit den Pin IDs
            context
                .frame_state
                .graph_changes
                .push(GraphChange::LinkRemoved {
                    start_pin_id: removed_link.spec.start_pin_index, // Pin IDs aus dem entfernten Link holen
                    end_pin_id: removed_link.spec.end_pin_index,
                });
        } else {
            bevy::log::warn!(
                "Attempted to remove link UI ID {} but it was not found.",
                *link_id
            );
        }
    }

    // 4. Entferne die Nodes aus dem internen State und sammle Events
    for node_id in nodes_to_remove_ids {
        // Iteriere über die zuvor gesammelten IDs
        if context.nodes.remove(&node_id).is_some() {
            context
                .frame_state
                .graph_changes
                .push(GraphChange::NodeRemoved(node_id));
            // Entferne Node auch aus der Tiefenordnung
            context.state.node_depth_order.retain(|id| *id != node_id);

            // Entferne zugehörige Pins (sicherer, die IDs vorher zu sammeln)
            // `pins_to_remove_ids` enthält bereits die Pins aller gelöschten Nodes.
        }
    }

    // 5. Entferne die Pins der gelöschten Nodes aus dem internen State
    for pin_id in pins_to_remove_ids {
        // Verwende die gesammelten IDs
        context.pins.remove(&pin_id);
    }

    // 6. Selektionen leeren (passiert durch .drain() oben)
    context.state.selected_link_indices.clear();
    context.state.selected_node_indices.clear();
}
