use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32, CornerRadius, Stroke},
    EguiContexts,
};
use egui_dock::Style;
use std::collections::HashMap;

use super::{
    context::LinkValidationCallback,
    drawing::*,
    hover::*,
    interaction::*,
    plugin::*,
    resources::GraphUIData,
    settings::NodesSettings,
    state::GraphUiStateManager,
    storage::GraphStorage,
    ui_data::*,
    ui_link::*,
    ui_node::{Node, NodeArgs, NodeSpec, NodeState},
    ui_pin::*,
    ui_style::*,
};

pub fn graph_ui_system(
    mut egui_contexts: EguiContexts,
    mut dock_state: ResMut<MyDockState>,
    graph_data: Res<GraphUIData>,
    mut ui_state: ResMut<GraphUiStateManager>,
    mut storage: ResMut<GraphStorage>,
    settings: Res<NodesSettings>,
) {
    let ctx = egui_contexts.ctx_mut();

    let egui_style = ctx.style().clone(); // Holen nach potenziellen UI-Änderungen
    let mut tab_viewer = MyTabViewer {
        graph_data: &graph_data,
    };
    let dock_style = Style::from_egui(&egui_style);

    // DockArea::new(&mut dock_state.0)
    //     .style(dock_style)
    //     .show(ctx, &mut tab_viewer);

    egui::Window::new("Node Graph Editor")
        .resizable(true)
        .collapsible(true)
        .title_bar(true)
        .show(ctx, |ui| {
            // 1. Reset State (Verwendet jetzt ui_state und das Window ui)
            ui_state.reset_frame_state(ui);

            // 2. Convert Input Data (Verwendet graph_data, settings)
            let mut node_specs = HashMap::new();
            let mut link_specs = Vec::new();
            let mut current_pins_for_frame: HashMap<usize, PinSpec> = HashMap::new();
            let get_color_for_relation = |relation_type: &str| -> Color32 {
                match relation_type {
                    "Family" => Color32::ORANGE,
                    "Friendship" => Color32::GREEN,
                    _ => Color32::GRAY,
                }
            };
            for vis_node in graph_data.nodes.iter() {
                let node_id = vis_node.id;
                let mut pins_for_this_node: Vec<PinSpec> = Vec::new();
                for logical_pin in &vis_node.logical_pins {
                    let pin_id = generate_pin_id(node_id, &logical_pin.identifier); // Nutzt freie Funktion
                    let pin_kind = match logical_pin.direction {
                        PinDirection::Input => PinType::Input,
                        PinDirection::Output | PinDirection::InOut => PinType::Output,
                    };
                    let pin_flags = if pin_kind == PinType::Output {
                        (AttributeFlags::EnableLinkCreationOnSnap as usize)
                            | (AttributeFlags::EnableLinkDetachWithDragClick as usize)
                    } else {
                        AttributeFlags::None as usize
                    };
                    let pin_base_color = get_color_for_relation(&logical_pin.relation_type);
                    let pin_spec = PinSpec {
                        id: pin_id,
                        kind: pin_kind,
                        name: logical_pin.display_name.clone(),
                        relation_type: logical_pin.relation_type.clone(),
                        flags: pin_flags,
                        style_args: PinStyleArgs {
                            background: Some(pin_base_color),
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    pins_for_this_node.push(pin_spec.clone());
                    current_pins_for_frame.insert(pin_id, pin_spec);
                }
                let node_args = NodeArgs::default();
                let node_color_srgba = vis_node.color.to_srgba();
                node_specs.insert(
                    node_id,
                    NodeSpec {
                        id: node_id,
                        name: vis_node.name.clone(),
                        origin: egui::pos2(vis_node.position.x, vis_node.position.y),
                        color: Color32::from_rgba_premultiplied(
                            (node_color_srgba.red * 255.0).round() as u8, // Verwende Felder von Srgba
                            (node_color_srgba.green * 255.0).round() as u8,
                            (node_color_srgba.blue * 255.0).round() as u8,
                            (node_color_srgba.alpha * 255.0).round() as u8, // Achte auf alpha!
                        ),
                        attributes: pins_for_this_node,
                        args: node_args,
                        subtitle: format!("E:{:?}", vis_node.entity),
                        time: None,
                        duration: None,
                        active: false,
                    },
                );
            }
            for vis_link in graph_data.links.iter() {
                if current_pins_for_frame.contains_key(&vis_link.start_pin_id)
                    && current_pins_for_frame.contains_key(&vis_link.end_pin_id)
                {
                    let link_base_color = if let Some(start_pin_spec) =
                        current_pins_for_frame.get(&vis_link.start_pin_id)
                    {
                        get_color_for_relation(&start_pin_spec.relation_type)
                    } else {
                        Color32::DARK_GRAY
                    };
                    let link_style = LinkStyleArgs {
                        base: Some(link_base_color),
                        ..Default::default()
                    };
                    link_specs.push(LinkSpec {
                        id: vis_link.id,
                        start_pin_index: vis_link.start_pin_id,
                        end_pin_index: vis_link.end_pin_id,
                        style: link_style,
                    });
                }
            }

            // 3. Definiere den Canvas-Bereich und Interaktion (Verwendet ui_state und das Window ui)
            let canvas_rect = ui_state.frame.canvas_rect_screen_space; // Aus ui_state holen
            let canvas_interact_response = ui.interact(
                canvas_rect,
                ui.id().with("NodeCanvasInteractor"),
                egui::Sense::click_and_drag(),
            );

            // 4. Zeichne Hintergrund & Grid (Verwendet settings, ui_state und freie Funktionen aus coords/drawing)
            let painter = ui.painter_at(canvas_rect); // Painter vom Window ui holen
            painter.rect_filled(
                canvas_rect,
                CornerRadius::ZERO,
                settings.style.colors[ColorStyle::GridBackground as usize],
            );
            if (settings.style.flags & StyleFlags::GridLines as usize) != 0 {
                // Koordinatenfunktionen müssen hier aufgerufen werden
                let canvas_origin = ui_state.canvas_origin_screen_space();
                let panning = ui_state.get_panning();
                // Passe draw_grid Signatur an oder erstelle Wrapper
                draw_grid(&mut *ui_state, &settings, canvas_rect.size(), ui); // Veraltet
                                                                              // ----> Aufruf von draw_grid HIER einfügen, mit den richtigen Parametern
            }

            // 5. Populate Internal State (Verwendet ui_state, storage, settings)
            // Die Logik von `add_node`, `add_pin`, `add_link` kommt hierher,
            // aber modifiziert `storage` und interagiert NICHT mit dem Painter.
            {
                let mut node_ids_this_frame: Vec<usize> = node_specs.keys().copied().collect();
                ui_state
                    .get_node_depth_order_mut()
                    .retain(|id| node_ids_this_frame.contains(id)); // ui_state Methode nutzen
                node_ids_this_frame.retain(|id| !ui_state.get_node_depth_order().contains(id));
                node_ids_this_frame.sort_unstable();
                ui_state
                    .get_node_depth_order_mut()
                    .extend(node_ids_this_frame);

                storage.clear(); // Storage leeren

                let nodes_to_add: Vec<(usize, NodeSpec)> = ui_state
                    .get_node_depth_order() // ui_state Methode nutzen
                    .iter()
                    .filter_map(|node_id| {
                        node_specs.get(node_id).map(|node_spec| {
                            let mut spec_clone = node_spec.clone();
                            spec_clone.active = ui_state.is_node_selected(*node_id); // ui_state Methode nutzen
                            (*node_id, spec_clone)
                        })
                    })
                    .collect();

                for (node_id, node_spec) in nodes_to_add {
                    // --- Logik aus add_node ---
                    let mut node_args = node_spec.args.clone();
                    // ... node_args.titlebar berechnen ...
                    let mut node = Node {
                        spec: node_spec,
                        state: NodeState::default(),
                    }; // Alten State ggf. von woanders holen? Vorerst default.
                    let (color_style, layout_style) = settings.style.format_node(node_args);
                    node.state.color_style = color_style;
                    node.state.layout_style = layout_style;
                    node.state.pin_indices.clear();
                    // --- Painter Interaktion WEGLASSEN ---
                    // node.state.background_shape = Some(ui.painter().add(egui::Shape::Noop));

                    // --- Node UI Layout (wird komplexer ohne allocate_ui_at_rect direkt hier) ---
                    // TODO: Dieser Teil muss überdacht werden. Wie bekommen wir die Node-Größe?
                    //       Möglicherweise müssen wir Nodes *zweimal* durchlaufen:
                    //       1. Layout-Pass (ohne Zeichnen) um Größen zu bestimmen (in ui_state.frame.nodes_tmp?)
                    //       2. Zeichen-Pass (final_draw)
                    // Vorerst: Simples Rect aus Position (aus node_spec.origin) und Default-Größe
                    let default_size = egui::vec2(150.0, 50.0); // Beispiel
                    node.state.rect = egui::Rect::from_min_size(node.spec.origin, default_size);
                    node.state.size = default_size;
                    // title_bar_content_rect muss auch anders berechnet werden (z.B. Annahme fester Höhe)
                    node.state.title_bar_content_rect = egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(default_size.x, 20.0),
                    );

                    // Pins für diesen Node erstellen und hinzufügen
                    let pins_for_this_node = node.spec.attributes.clone();
                    for pin_spec in pins_for_this_node {
                        node.state.pin_indices.push(pin_spec.id);
                        // --- Logik aus add_pin ---
                        let pin_id = pin_spec.id;
                        let mut pin = Pin {
                            spec: pin_spec.clone(),
                            state: PinState::default(),
                        };
                        // ... label_rect_relative_to_node_origin berechnen (wird auch komplexer) ...
                        // Vorerst Annahme:
                        let pin_y_offset = 30.0 + node.state.pin_indices.len() as f32 * 15.0; // Beispiel
                        let relative_rect = egui::Rect::from_min_max(
                            egui::pos2(10.0, pin_y_offset - 5.0),
                            egui::pos2(50.0, pin_y_offset + 5.0),
                        );
                        pin.state.attribute_rect = relative_rect;
                        pin.state.parent_node_idx = node_id;
                        pin.state.color_style =
                            settings.style.format_pin(pin_spec.style_args.clone());
                        // --- Painter Interaktion WEGLASSEN ---
                        // pin.state.shape_gui = Some(ui.painter().add(egui::Shape::Noop));
                        storage.add_pin(pin); // Pin zu Storage
                    }

                    storage.add_node(node); // Node zu Storage
                }

                // Links hinzufügen
                // Create a Vec of link_ids to remove
                let links_to_remove: Vec<_> = storage
                    .links
                    .iter()
                    .filter(|&(_, link)| {
                        !storage.contains_pin(link.spec.start_pin_index)
                            || !storage.contains_pin(link.spec.end_pin_index)
                    })
                    .map(|(link_id, _)| *link_id)
                    .collect();

                // Remove the links in a separate step
                for link_id in links_to_remove {
                    storage.links.remove(&link_id);
                }
                for link_spec in link_specs.iter().cloned() {
                    // Klonen, um Borrowing zu vermeiden
                    // --- Logik aus add_link ---
                    let link_id = link_spec.id;
                    let link_state = LinkState {
                        style: settings.style.format_link(link_spec.style.clone()),
                    };
                    // Füge Link zu Storage hinzu (oder aktualisiere)
                    let link_spec_for_update = link_spec.clone();
                    storage
                        .links
                        .entry(link_id)
                        .or_insert_with(|| Link {
                            spec: link_spec.clone(),
                            state: LinkState::default(),
                        })
                        .spec = link_spec_for_update; // Immer Spec aktualisieren
                    if let Some(l) = storage.links.get_mut(&link_id) {
                        l.state = link_state;
                    } // State setzen
                }
            }

            // 6. Interaction Processing (Verwendet ui_state, storage, settings und freie Funktionen)
            let link_validator_closure: Box<LinkValidationCallback> =
                Box::new(|start_pin_spec, end_pin_spec, storage_cb, settings_cb| {
                    // Implementierung des Callbacks
                    if start_pin_spec.relation_type != end_pin_spec.relation_type {
                        return false;
                    }
                    let valid_direction = match (start_pin_spec.kind, end_pin_spec.kind) {
                        (PinType::Output, PinType::Input) => true,
                        (PinType::Input, PinType::Output) => true,
                        (PinType::Output, PinType::Output) => {
                            start_pin_spec.relation_type == "Friendship"
                        }
                        _ => false,
                    };
                    valid_direction
                });

            let hover_pos = canvas_interact_response.hover_pos();
            // Input State direkt holen
            let input = ui.ctx().input(|i| i.clone());
            ui_state.update_interaction_state(
                &input,
                hover_pos,
                settings.io.emulate_three_button_mouse,
                settings.io.link_detatch_with_modifier_click,
                settings.io.alt_mouse_button,
            );
            // Passe Signaturen der Helfer an und rufe sie hier auf
            resolve_hover_state(&mut ui_state, &storage, &settings);
            process_clicks(&mut ui_state, &mut storage, &settings); // Braucht ggf. Coords
            click_interaction_update(
                &mut ui_state,
                &mut storage,
                &settings,
                &link_validator_closure,
            ); // Übergib ui & Closure
            if ui_state.get_interaction_state().delete_pressed {
                handle_delete(&mut ui_state, &mut storage);
            }

            // 7. Drawing (Verwendet ui_state, storage, settings, ui und freie Funktionen)
            final_draw(&mut ui_state, &mut *storage, &settings, ui);

            // 8. Canvas Outline (Verwendet painter vom Window ui)
            let outline_stroke = Stroke::new(1.0, Color32::WHITE);
            painter.rect_stroke(
                canvas_rect,
                egui::epaint::CornerRadius::ZERO,
                outline_stroke,
                egui::StrokeKind::Outside,
            );

            // WICHTIG: Response des Canvas zurückgeben, falls benötigt
            // canvas_interact_response

            // *** ENDE DES EINGEFÜGTEN CODES ***
        });
}
