use bevy::log;
use bevy_egui::egui::{self, *};

use super::{
    coords::{
        editor_space_to_screen_space, get_screen_space_pin_coordinates, grid_space_to_screen_space,
    },
    interaction::ClickInteractionType,
    settings::NodesSettings,
    state::GraphUiStateManager,
    storage::GraphStorage,
    ui_link::LinkBezierData,
    ui_pin::PinShape,
    ui_style::{ColorStyle,StyleFlags}   
};

pub fn draw_grid(
    ui_state: &mut GraphUiStateManager,
    settings: &NodesSettings,
    _canvas_size: egui::Vec2,
    ui: &mut egui::Ui,
) {
    /* Vollständige Implementierung wie oben */
    let painter = ui.painter();
    let style = &settings.style;
    let spacing = style.grid_spacing;
    let line_stroke = egui::Stroke::new(1.0, style.colors[ColorStyle::GridLine as usize]);
    let canvas_origin_screen = editor_space_to_screen_space(ui_state, egui::Pos2::ZERO); // Wo der (0,0) Punkt des Grids auf dem Bildschirm liegt
    let visible_rect = ui.clip_rect(); // Sichtbarer Bereich der UI

    // Vertikale Linien
    let x_min_grid_for_screen =
        visible_rect.min.x - canvas_origin_screen.x - ui_state.persistent.panning.x; // Korrektur: Verwende grid space für Berechnung
    let x_start_index = (x_min_grid_for_screen / spacing).floor() as i32;
    // Erste *sichtbare* vertikale Linie im Grid Space bestimmen
    let mut grid_x = x_start_index as f32 * spacing; // Grid Koordinate
    loop {
        let screen_x = grid_space_to_screen_space(ui_state, egui::pos2(grid_x, 0.0)).x;
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
        visible_rect.min.y - canvas_origin_screen.y - ui_state.persistent.panning.y; // Korrektur: Verwende grid space
    let y_start_index = (y_min_grid_for_screen / spacing).floor() as i32;
    // Erste *sichtbare* horizontale Linie im Grid Space bestimmen
    let mut grid_y = y_start_index as f32 * spacing; // Grid Koordinate
    loop {
        let screen_y = grid_space_to_screen_space(ui_state, egui::pos2(0.0, grid_y)).y;
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

fn draw_link(
    ui_state: &mut GraphUiStateManager,
    storage: &GraphStorage,
    settings: &NodesSettings,
    link_id: usize,
    ui: &mut egui::Ui,
) {
    let link_hovered = ui_state.frame.hovered_link_idx == Some(link_id);
    let link_directly_selected = ui_state.persistent.selected_link_indices.contains(&link_id);

    if let Some(link) = storage.links.get(&link_id) {
        if let (Some(start_pin), Some(end_pin)) = (
            storage.pins.get(&link.spec.start_pin_index),
            storage.pins.get(&link.spec.end_pin_index),
        ) {
            // === NEUE PRÜFUNG: Ist einer der verbundenen Nodes ausgewählt? ===
            let start_node_id = start_pin.state.parent_node_idx;
            let end_node_id = end_pin.state.parent_node_idx;
            let node_is_selected = ui_state
                .persistent
                .selected_node_indices
                .contains(&start_node_id)
                || ui_state
                    .persistent
                    .selected_node_indices
                    .contains(&end_node_id);
            // ================================================================

            let link_data = LinkBezierData::get_link_renderable(
                grid_space_to_screen_space(ui_state, start_pin.state.pos),
                grid_space_to_screen_space(ui_state, end_pin.state.pos),
                start_pin.spec.kind,
                settings.style.link_line_segments_per_length,
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

fn draw_node(
    ui_state: &mut GraphUiStateManager,
    storage: &mut GraphStorage,
    settings: &NodesSettings,
    node_id: usize,
    ui: &mut egui::Ui,
) {
    if let Some(node) = storage.nodes.get(&node_id) {
        // let node_hovered = context.ui_state.frame.hovered_node_index == Some(node_id);
        let is_selected = ui_state.persistent.selected_node_indices.contains(&node_id);

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
            settings.style.colors[ColorStyle::NodeOutlineActive as usize]
        } else {
            // Standard-Outline-Farbe (aus dem Blender Theme)
            settings.style.colors[ColorStyle::NodeOutline as usize]
        };

        let painter = ui.painter(); // Painter vom übergebenen UI
        let screen_rect = egui::Rect::from_min_max(
            grid_space_to_screen_space(&ui_state, node.state.rect.min),
            grid_space_to_screen_space(&ui_state, node.state.rect.max),
        );
        let title_rect_grid = node.state.get_node_title_rect(); // Nutzt Methode von NodeState
        let screen_title_rect = egui::Rect::from_min_max(
            grid_space_to_screen_space(&ui_state, title_rect_grid.min),
            grid_space_to_screen_space(&ui_state, title_rect_grid.max),
        );
        let rounding = CornerRadius::from(node.state.layout_style.corner_rounding); // Korrigiert

        // Hintergrund

        painter.rect_filled(screen_rect, rounding.clone(), bg_col);
        if node.state.title_bar_content_rect.height() > 0.0 {
            let title_rect_shape = screen_rect.intersect(screen_title_rect); // screen_title_rect von oben verwenden
            let title_shape = egui::Shape::Rect(egui::epaint::RectShape {
                rect: title_rect_shape,
                corner_radius: egui::epaint::CornerRadius { nw: rounding.nw, ne: rounding.ne, sw: 0, se: 0 },
                fill: title_col,
                stroke: egui::Stroke::NONE,
                stroke_kind: StrokeKind::Inside,
                round_to_pixels: Some(false),
                blur_width: 0.0,
                brush: None,
           });
           painter.add(title_shape);
        }
        // Outline
        if (settings.style.flags & StyleFlags::NodeOutline as usize) != 0 { 
            painter.add(egui::Shape::Rect(egui::epaint::RectShape {
                rect: screen_rect.expand(node.state.layout_style.border_thickness / 2.0),
                corner_radius: rounding.clone(), 
                fill: Color32::TRANSPARENT,
                stroke: egui::Stroke::new(node.state.layout_style.border_thickness, outline_col),
                stroke_kind: StrokeKind::Outside,
                round_to_pixels: Some(false),
                blur_width: 0.0,
                brush: None,
            }));
        }

        let pin_ids_to_draw = node.state.pin_indices.clone();
        // drop(node); // Release immutable borrow

        for pin_id in pin_ids_to_draw {
            draw_pin(ui_state, storage, settings, pin_id, ui);
        }
    } else {
        log::warn!(
            "Versuche Node {} zu zeichnen, aber nicht gefunden.",
            node_id
        );
    }
}

fn draw_pin(
    ui_state: &mut GraphUiStateManager,
    storage: &mut GraphStorage,
    settings: &NodesSettings,
    pin_idx: usize,
    ui: &mut egui::Ui,
) {
    let pin_hovered = ui_state.frame.hovered_pin_index == Some(pin_idx);
    let pin_data: Option<(egui::Pos2, PinShape, Color32,  usize)> = // Füge Pin Flags hinzu
        // Immutable borrow um Pin Daten zu lesen
        if let Some(pin) = storage.pins.get(&pin_idx) {
             let screen_pos = get_screen_space_pin_coordinates(ui_state, storage, settings, pin); // Position berechnen

             let draw_color = pin.state.color_style.background;
             let draw_shape = pin.state.color_style.shape;
             
            let flags = pin.spec.flags;

             Some((screen_pos, draw_shape, draw_color,  flags))
         } else {
             None
        };

    // Wenn Pin Daten vorhanden sind -> Zeichnen und State aktualisieren (mutable borrow)
    if let Some((screen_pos, draw_shape, draw_color,  flags)) = pin_data {
        // Zeichnen mit painter.set()

        settings
            .style
            .draw_pin_shape(screen_pos, draw_shape, draw_color,  ui);

        // === Aktualisiere Pin State ===
        if let Some(pin_mut) = storage.pins.get_mut(&pin_idx) {
            // Update der aktuellen Screen Position des Pins
            pin_mut.state.pos = screen_pos;

            // Wenn dieser Pin gerade gehovert wird, speichere seine Flags für Interaction Checks
            if pin_hovered {
                ui_state.frame.hovered_pin_flags = flags; // context durch ui_state ersetzt
            }
        }
    }
}

fn draw_temporary_elements(
    ui_state: &mut GraphUiStateManager,
    storage: &GraphStorage,
    settings: &NodesSettings,
    ui: &mut egui::Ui,
) {
    // === Zeichnen für LinkCreation ===
    if ui_state.persistent.click_interaction_type == ClickInteractionType::LinkCreation {
        // Hole Start-Pin (sollte existieren, da die Interaktion läuft)
        if let Some(start_pin_id) = ui_state
            .persistent
            .click_interaction_state
            .link_creation
            .start_pin_idx
        {
            if let Some(start_pin) = storage.pins.get(&start_pin_id) {
                let start_pos =
                    get_screen_space_pin_coordinates(ui_state, storage, settings, start_pin); // Berechne aktuelle Startposition

                // Bestimme Endposition: Entweder Mausposition oder Position eines gesnappten Pins
                let end_pos = ui_state
                    .persistent
                    .click_interaction_state
                    .link_creation
                    .end_pin_index
                    .and_then(|id| storage.pins.get(&id))
                    .map_or(ui_state.persistent.interaction_state.mouse_pos, |p| {
                        get_screen_space_pin_coordinates(ui_state, storage, settings, p)
                    }); // Berechne aktuelle Endposition

                // Erstelle die Bezier-Daten
                let link_data = LinkBezierData::get_link_renderable(
                    start_pos,
                    end_pos,
                    start_pin.spec.kind,
                    settings.style.link_line_segments_per_length,
                );

                // Zeichne die temporäre Linie
                let temp_link_color = if ui_state
                    .persistent
                    .click_interaction_state
                    .link_creation
                    .end_pin_index
                    .is_some()
                {
                    // Farbe, wenn über gültigem Pin gesnapped
                    settings.style.colors[ColorStyle::LinkSelected as usize]
                } else {
                    // Standardfarbe während des Ziehens
                    settings.style.colors[ColorStyle::Link as usize]
                };

                ui.painter()
                    .add(link_data.draw((settings.style.link_thickness, temp_link_color)));
            }
        }
    }

    // === Zeichnen für BoxSelection ===
    if ui_state.persistent.click_interaction_type == ClickInteractionType::BoxSelection {
        let selection_rect = ui_state.persistent.click_interaction_state.box_selection;
        // Stelle sicher, dass das Rechteck normalisiert ist (min < max)
        let normalized_rect = egui::Rect::from_min_max(
            selection_rect.min.min(selection_rect.max),
            selection_rect.min.max(selection_rect.max),
        );

        // Zeichne gefülltes Rechteck
        ui.painter().rect_filled(
            normalized_rect,
            CornerRadius::ZERO,
            settings.style.colors[ColorStyle::BoxSelector as usize],
        );
        // Zeichne Umriss
        ui.painter().rect_stroke(
            normalized_rect,
            CornerRadius::ZERO,
            egui::Stroke::new(
                1.0,
                settings.style.colors[ColorStyle::BoxSelectorOutline as usize],
            ),
            StrokeKind::Inside,
        );
    }
}

pub fn final_draw(
    ui_state: &mut GraphUiStateManager,
    storage: &mut GraphStorage,
    settings: &NodesSettings,
    ui_draw: &mut egui::Ui,
) {
    // === 1. Links zeichnen (unter den Nodes) ===
    let link_ids: Vec<usize> = storage.links.keys().copied().collect(); // Beinhaltet jetzt nur noch XOR IDs

    let interaction_type = ui_state.persistent.click_interaction_type;
    let modifying_id_opt = if interaction_type == ClickInteractionType::LinkCreation {
        ui_state
            .persistent
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
        draw_link(ui_state, storage, settings, link_id, ui_draw);
    }

    // === 2. Nodes zeichnen ...
    let node_order = ui_state.persistent.node_depth_order.clone();
    for node_id in node_order.iter() {
        draw_node(ui_state, storage, settings, *node_id, ui_draw);
    }

    // === 3. Temporäre Elemente zeichnen ...
    draw_temporary_elements(ui_state, storage, settings, ui_draw);
}

// // Gibt das Rect des Titelbalken-Bereichs (inkl. Padding) im Grid-Space zurück
// pub fn get_node_title_rect_grid(&self) -> egui::Rect {
//     let title_height_with_padding =
//         self.title_bar_content_rect.height() + self.layout_style.padding.y * 2.0; // Höhe des Inhalts + oberes/unteres Padding
//     egui::Rect::from_min_size(
//         self.rect.min, // Beginnt am Ursprung des Node-Rects
//         egui::vec2(self.rect.width(), title_height_with_padding.max(0.0)), // Nimmt die volle Breite, aber begrenzte Höhe
//     )
// }

// // Konvertiert das Grid-Space Titel-Rect in Screen-Space
// pub fn get_node_title_rect_screen(&self, context: &NodesContext) -> egui::Rect {
//     let grid_rect = self.get_node_title_rect_grid();
//     egui::Rect::from_min_max(
//         // MinMax verwenden, um sicherzugehen
//         context.grid_space_to_screen_space(grid_rect.min),
//         context.grid_space_to_screen_space(grid_rect.max),
//     )
// }
