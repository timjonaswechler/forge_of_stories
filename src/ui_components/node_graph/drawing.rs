use bevy::log;
use bevy_egui::egui::{self, *};

use super::{
    context::NodesContext,
    interaction::ClickInteractionType,
    ui_link::LinkBezierData,
    ui_pin::PinShape,
    ui_style::{ColorStyle, StyleFlags},
};

pub fn draw_grid(context: &NodesContext, _canvas_size: egui::Vec2, ui: &mut egui::Ui) {
    /* Vollständige Implementierung wie oben */
    let painter = ui.painter();
    let style = &context.settings.style;
    let spacing = style.grid_spacing;
    let line_stroke = egui::Stroke::new(1.0, style.colors[ColorStyle::GridLine as usize]);
    let canvas_origin_screen = context.editor_space_to_screen_space(egui::Pos2::ZERO); // Wo der (0,0) Punkt des Grids auf dem Bildschirm liegt
    let visible_rect = ui.clip_rect(); // Sichtbarer Bereich der UI

    // Vertikale Linien
    let x_min_grid_for_screen =
        visible_rect.min.x - canvas_origin_screen.x - context.state.panning.x; // Korrektur: Verwende grid space für Berechnung
    let x_start_index = (x_min_grid_for_screen / spacing).floor() as i32;
    // Erste *sichtbare* vertikale Linie im Grid Space bestimmen
    let mut grid_x = x_start_index as f32 * spacing; // Grid Koordinate
    loop {
        let screen_x = context
            .grid_space_to_screen_space(egui::pos2(grid_x, 0.0))
            .x;
        if screen_x < visible_rect.min.x {
            grid_x += spacing;
            continue;
        } // Überspringe, wenn links ausserhalb
        if screen_x > visible_rect.max.x {
            break;
        } // Stoppe, wenn rechts ausserhalb

        painter.line_segment(
            [
                egui::pos2(screen_x, visible_rect.min.y),
                egui::pos2(screen_x, visible_rect.max.y),
            ],
            line_stroke,
        );
        grid_x += spacing;
    }

    // Horizontale Linien
    let y_min_grid_for_screen =
        visible_rect.min.y - canvas_origin_screen.y - context.state.panning.y; // Korrektur: Verwende grid space
    let y_start_index = (y_min_grid_for_screen / spacing).floor() as i32;
    // Erste *sichtbare* horizontale Linie im Grid Space bestimmen
    let mut grid_y = y_start_index as f32 * spacing; // Grid Koordinate
    loop {
        let screen_y = context
            .grid_space_to_screen_space(egui::pos2(0.0, grid_y))
            .y;
        if screen_y < visible_rect.min.y {
            grid_y += spacing;
            continue;
        } // Überspringe, wenn oben ausserhalb
        if screen_y > visible_rect.max.y {
            break;
        } // Stoppe, wenn unten ausserhalb

        painter.line_segment(
            [
                egui::pos2(visible_rect.min.x, screen_y),
                egui::pos2(visible_rect.max.x, screen_y),
            ],
            line_stroke,
        );
        grid_y += spacing;
    }
}

fn draw_link(context: &mut NodesContext, link_id: usize, ui: &mut egui::Ui) {
    let link_hovered = context.frame_state.hovered_link_idx == Some(link_id);
    let link_directly_selected = context.state.selected_link_indices.contains(&link_id);

    if let Some(link) = context.links.get(&link_id) {
        if let (Some(start_pin), Some(end_pin)) = (
            context.pins.get(&link.spec.start_pin_index),
            context.pins.get(&link.spec.end_pin_index),
        ) {
            // === NEUE PRÜFUNG: Ist einer der verbundenen Nodes ausgewählt? ===
            let start_node_id = start_pin.state.parent_node_idx;
            let end_node_id = end_pin.state.parent_node_idx;
            let node_is_selected = context.state.selected_node_indices.contains(&start_node_id)
                || context.state.selected_node_indices.contains(&end_node_id);
            // ================================================================

            let link_data = LinkBezierData::get_link_renderable(
                context.get_screen_space_pin_coordinates(start_pin),
                context.get_screen_space_pin_coordinates(end_pin),
                start_pin.spec.kind,
                context.settings.style.link_line_segments_per_length,
            );

            // === ANGEPASSTE FARBWAHL ===
            let color = if node_is_selected || link_directly_selected {
                // Link hervorheben, wenn Node oder Link selbst ausgewählt ist
                link.state.style.selected
            } else if link_hovered {
                link.state.style.hovered
            } else {
                link.state.style.base
            };
            // ===========================

            // --- Dicke optional anpassen ---
            // Man könnte auch die Dicke ändern, wenn ein Node selektiert ist
            let thickness = if node_is_selected || link_directly_selected {
                link.state.style.thickness * 1.2 // Mache den Link etwas dicker
            } else {
                link.state.style.thickness // Normale Dicke
            };
            let outline_thickness_factor = 1.2; // Faktor für die Outline-Breite
            let outline_color = Color32::GREEN;

            // --------------------------------

            // 1. Zeichne den breiteren Outline-Pfad zuerst
            ui.painter().add(link_data.draw(
                // Erzeuge den Stroke für die Outline
                egui::Stroke::new(thickness * outline_thickness_factor, outline_color),
            ));

            // 2. Zeichne den schmaleren, farbigen Pfad darüber
            ui.painter().add(link_data.draw(
                // Erzeuge den Stroke für den eigentlichen Link
                egui::Stroke::new(thickness, color),
            ));
        } else {
            // Link verweist auf ungültige Pins, sollte durch `retain` vorher entfernt worden sein
            bevy::log::warn!(
                "Versuche Link {} zu zeichnen, aber Pins nicht gefunden.",
                link_id
            );
        }
    } else {
        // Sollte nicht vorkommen, wenn link_id aus self.links kommt
    }
}

fn draw_node(context: &mut NodesContext, node_id: usize, ui: &mut egui::Ui) {
    if let Some(node) = context.nodes.get(&node_id) {
        // let node_hovered = context.frame_state.hovered_node_index == Some(node_id);
        let is_selected = context.state.selected_node_indices.contains(&node_id);

        // Farben auswählen
        let bg_col = if is_selected {
            node.state.color_style.background_selected
        } else {
            node.state.color_style.background
        };

        let title_col = if is_selected {
            node.state.color_style.titlebar_selected
        } else {
            node.state.color_style.titlebar
        };

        let outline_col = if is_selected {
            // SEHR HELLE Farbe für ausgewählten Node
            Color32::WHITE // Oder z.B. eine sehr helle Variante der Akzentfarbe
        } else if node.spec.active {
            // Falls wir `active` später nutzen
            context.settings.style.colors[ColorStyle::NodeOutlineActive as usize]
        } else {
            // Standard-Outline-Farbe (aus dem Blender Theme)
            context.settings.style.colors[ColorStyle::NodeOutline as usize]
        };

        let painter = ui.painter(); // Painter vom übergebenen UI
        let screen_rect = egui::Rect::from_min_max(
            context.grid_space_to_screen_space(node.state.rect.min),
            context.grid_space_to_screen_space(node.state.rect.max),
        );
        let screen_title_rect = node.state.get_node_title_rect_screen(context);
        let rounding = CornerRadius::from(node.state.layout_style.corner_rounding); // Korrigiert

        // Hintergrund
        if let Some(idx) = node.state.background_shape {
            painter.set(
                idx,
                egui::Shape::rect_filled(screen_rect, rounding.clone(), bg_col),
            ); // Clone rounding
        }
        // Titelbalken
        if node.state.title_bar_content_rect.height() > 0.0 {
            if let Some(idx) = node.state.titlebar_shape {
                let title_rect_shape = screen_rect.intersect(screen_title_rect);
                let title_shape = egui::Shape::Rect(egui::epaint::RectShape {
                    rect: title_rect_shape,
                    corner_radius: egui::epaint::CornerRadius {
                        nw: rounding.nw,
                        ne: rounding.ne,
                        sw: 0,
                        se: 0,
                    }, // Korrigiert
                    fill: title_col,
                    stroke: egui::Stroke::NONE,
                    stroke_kind: StrokeKind::Outside,
                    round_to_pixels: Some(false),
                    blur_width: 0.0,
                    brush: None,
                });
                painter.set(idx, title_shape);
            }
        }
        // Outline
        if (context.settings.style.flags & StyleFlags::NodeOutline as usize) != 0 {
            if let Some(idx) = node.state.outline_shape {
                painter.set(
                    idx,
                    egui::Shape::Rect(egui::epaint::RectShape {
                        rect: screen_rect.expand(node.state.layout_style.border_thickness / 2.0),
                        corner_radius: rounding.clone(), // Clone rounding
                        fill: Color32::TRANSPARENT,
                        stroke: egui::Stroke::new(
                            node.state.layout_style.border_thickness,
                            outline_col,
                        ),
                        stroke_kind: StrokeKind::Outside,
                        round_to_pixels: Some(false),
                        blur_width: 0.0,
                        brush: None,
                    }),
                );
            }
        }

        let pin_ids_to_draw = node.state.pin_indices.clone();
        // drop(node); // Release immutable borrow

        for pin_id in pin_ids_to_draw {
            draw_pin(context, pin_id, ui);
        }
    } else {
        log::warn!(
            "Versuche Node {} zu zeichnen, aber nicht gefunden.",
            node_id
        );
    }
}

fn draw_pin(context: &mut NodesContext, pin_idx: usize, ui: &mut egui::Ui) {
    let pin_hovered = context.frame_state.hovered_pin_index == Some(pin_idx);
    let pin_data: Option<(egui::Pos2, PinShape, Color32, egui::layers::ShapeIdx, usize)> = // Füge Pin Flags hinzu
        // Immutable borrow um Pin Daten zu lesen
        if let Some(pin) = context.pins.get(&pin_idx) {
             let screen_pos = context.get_screen_space_pin_coordinates(pin); // Position berechnen

             let draw_color = pin.state.color_style.background;
             let draw_shape = pin.state.color_style.shape;
             let shape_idx = pin.state.shape_gui.expect("Pin Shape Index nicht initialisiert"); // Sollte in add_pin gesetzt sein
            let flags = pin.spec.flags;

             Some((screen_pos, draw_shape, draw_color, shape_idx, flags))
         } else {
             None
        };

    // Wenn Pin Daten vorhanden sind -> Zeichnen und State aktualisieren (mutable borrow)
    if let Some((screen_pos, draw_shape, draw_color, shape_idx, flags)) = pin_data {
        // Zeichnen mit painter.set()
        context
            .settings
            .style
            .draw_pin_shape(screen_pos, draw_shape, draw_color, shape_idx, ui);

        // === Aktualisiere Pin State ===
        if let Some(pin_mut) = context.pins.get_mut(&pin_idx) {
            // Update der aktuellen Screen Position des Pins
            pin_mut.state.pos = screen_pos;

            // Wenn dieser Pin gerade gehovert wird, speichere seine Flags für Interaction Checks
            if pin_hovered {
                context.frame_state.hovered_pin_flags = flags;
            }
        }
    }
}

fn draw_temporary_elements(context: &NodesContext, ui: &mut egui::Ui) {
    // === Zeichnen für LinkCreation ===
    if context.state.click_interaction_type == ClickInteractionType::LinkCreation {
        // Hole Start-Pin (sollte existieren, da die Interaktion läuft)
        if let Some(start_pin_id) = context
            .state
            .click_interaction_state
            .link_creation
            .start_pin_idx
        {
            if let Some(start_pin) = context.pins.get(&start_pin_id) {
                let start_pos = context.get_screen_space_pin_coordinates(start_pin); // Berechne aktuelle Startposition

                // Bestimme Endposition: Entweder Mausposition oder Position eines gesnappten Pins
                let end_pos = context
                    .state
                    .click_interaction_state
                    .link_creation
                    .end_pin_index
                    .and_then(|id| context.pins.get(&id))
                    .map_or(context.state.interaction_state.mouse_pos, |p| {
                        context.get_screen_space_pin_coordinates(p)
                    }); // Berechne aktuelle Endposition

                // Erstelle die Bezier-Daten
                let link_data = LinkBezierData::get_link_renderable(
                    start_pos,
                    end_pos,
                    start_pin.spec.kind,
                    context.settings.style.link_line_segments_per_length,
                );

                // Zeichne die temporäre Linie
                let temp_link_color = if context
                    .state
                    .click_interaction_state
                    .link_creation
                    .end_pin_index
                    .is_some()
                {
                    // Farbe, wenn über gültigem Pin gesnapped
                    context.settings.style.colors[ColorStyle::LinkSelected as usize]
                } else {
                    // Standardfarbe während des Ziehens
                    context.settings.style.colors[ColorStyle::Link as usize]
                };

                ui.painter()
                    .add(link_data.draw((context.settings.style.link_thickness, temp_link_color)));
            }
        }
    }

    // === Zeichnen für BoxSelection ===
    if context.state.click_interaction_type == ClickInteractionType::BoxSelection {
        let selection_rect = context.state.click_interaction_state.box_selection;
        // Stelle sicher, dass das Rechteck normalisiert ist (min < max)
        let normalized_rect = egui::Rect::from_min_max(
            selection_rect.min.min(selection_rect.max),
            selection_rect.min.max(selection_rect.max),
        );

        // Zeichne gefülltes Rechteck
        ui.painter().rect_filled(
            normalized_rect,
            CornerRadius::ZERO,
            context.settings.style.colors[ColorStyle::BoxSelector as usize],
        );
        // Zeichne Umriss
        ui.painter().rect_stroke(
            normalized_rect,
            CornerRadius::ZERO,
            egui::Stroke::new(
                1.0,
                context.settings.style.colors[ColorStyle::BoxSelectorOutline as usize],
            ),
            StrokeKind::Inside,
        );
    }
}

pub fn final_draw(context: &mut NodesContext, ui_draw: &mut egui::Ui) {
    // === 1. Links zeichnen (unter den Nodes) ===
    let link_ids: Vec<usize> = context.links.keys().copied().collect(); // Beinhaltet jetzt nur noch XOR IDs

    let interaction_type = context.state.click_interaction_type;
    let modifying_id_opt = if interaction_type == ClickInteractionType::LinkCreation {
        context
            .state
            .click_interaction_state
            .link_creation
            .modifying_link_id
    } else {
        None
    };

    for link_id in link_ids {
        let is_being_modified = modifying_id_opt == Some(link_id);

        if is_being_modified {
            continue;
        }
        draw_link(context, link_id, ui_draw);
    }

    // === 2. Nodes zeichnen ...
    let node_order = context.state.node_depth_order.clone();
    for node_id in node_order.iter() {
        draw_node(context, *node_id, ui_draw);
    }

    // === 3. Temporäre Elemente zeichnen ...
    draw_temporary_elements(context, ui_draw);
}
