// src/ui_components/node_graph/context.rs
use bevy::color::Srgba; // Direkter Import für Srgba Konvertierung
use bevy::math::Vec2 as BevyVec2; // Für Umwandlung
use bevy::prelude::Resource; // Bevy Typen
use bevy_egui::egui::{self, Color32, CornerRadius}; // Egui Typen, inkl. CornerRadius
use bevy_egui::egui::{Frame, Layout, Rect, Stroke};
use derivative::Derivative;
use std::collections::HashMap;

pub use super::{
    drawing::*,
    hover::resolve_hover_state,
    identity::generate_pin_id,
    interaction::*,
    ui_data::*,
    ui_link::*,
    ui_node::NodeArgs,
    ui_node::*,
    ui_pin::*,
    ui_pin::{AttributeFlags, PinShape, PinStyleArgs, PinType},
    ui_style::{ColorStyle, Style, StyleFlags},
};

pub type LinkValidationCallback = dyn Fn(&PinSpec, &PinSpec, &NodesContext) -> bool;

#[derive(Debug, Clone)]
pub enum GraphChange {
    LinkCreated(usize, usize), // Start Pin ID, End Pin ID
    LinkRemoved {
        start_pin_id: usize,
        end_pin_id: usize,
    },
    LinkModified {
        // NEU: Nur die neuen Pin IDs reichen
        new_start_pin_id: usize, // Neuer Output Pin
        new_end_pin_id: usize,   // Neuer Input Pin
        // TODO Evtl. sinnvoll: Die *alten* Pin-IDs mitschicken, um Aufräumen (Friendship) zu ermöglichen?
        old_start_pin_id: usize,
        old_end_pin_id: usize,
    },
    // === NEU: Event für Link Erstellung durch User-Aktion ===
    NewLinkRequested(usize, usize), // Start Pin ID, End Pin ID
    NodeMoved(usize, BevyVec2),     // Node ID, Grid Space Position
    NodeRemoved(usize),             // Node ID
}

#[derive(Derivative)]
#[derivative(Debug, Default)]
pub(crate) struct PersistentState {
    pub(crate) interaction_state: InteractionState,
    pub(crate) selected_node_indices: Vec<usize>,
    pub(crate) selected_link_indices: Vec<usize>,
    pub(crate) node_depth_order: Vec<usize>,
    pub(crate) panning: egui::Vec2,
    #[derivative(Default(value = "ClickInteractionType::None"))]
    pub(crate) click_interaction_type: ClickInteractionType,
    pub(crate) click_interaction_state: ClickInteractionState,
}

#[derive(Debug, Default)]
pub struct NodesSettings {
    pub io: IO,
    pub style: Style,
}

#[derive(Derivative)]
#[derivative(Debug, Default)] // Korrekte Syntax
pub struct FrameState {
    #[derivative(Default(value = "[[0.0; 2].into(); 2].into()"))]
    pub(crate) canvas_rect_screen_space: egui::Rect,
    pub(crate) node_indices_overlapping_with_mouse: Vec<usize>,
    pub(crate) occluded_pin_indices: Vec<usize>,
    pub(crate) hovered_node_index: Option<usize>,
    pub(crate) interactive_node_index: Option<usize>,
    pub(crate) hovered_link_idx: Option<usize>,
    pub(crate) hovered_pin_index: Option<usize>,
    pub(crate) hovered_pin_flags: usize,
    pub(crate) deleted_link_idx: Option<usize>,
    pub(crate) snap_link_idx: Option<usize>,
    pub(crate) element_state_change: ElementStateChange,
    pub(crate) active_pin: Option<usize>,
    pub(crate) graph_changes: Vec<GraphChange>, // Enthält jetzt auch NewLinkRequested
    pub(crate) pins_tmp: HashMap<usize, Pin>,
    pub(crate) nodes_tmp: HashMap<usize, Node>,
    pub(crate) just_selected_node: bool,
}

impl FrameState {
    pub fn reset(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        self.canvas_rect_screen_space = rect;
        self.node_indices_overlapping_with_mouse.clear();
        self.occluded_pin_indices.clear();
        self.hovered_node_index = None;
        self.interactive_node_index = None;
        self.hovered_link_idx = None;
        self.hovered_pin_index = None;
        self.hovered_pin_flags = AttributeFlags::None as usize;
        self.deleted_link_idx = None;
        self.snap_link_idx = None;
        self.element_state_change.reset();
        self.active_pin = None;
        self.graph_changes.clear();
        self.just_selected_node = false;
        self.pins_tmp.clear();
        self.nodes_tmp.clear();
    }
    pub fn canvas_origin_screen_space(&self) -> egui::Vec2 {
        self.canvas_rect_screen_space.min.to_vec2()
    }
}

#[derive(Resource, Derivative)]
#[derivative(Debug, Default)]
pub struct NodesContext {
    pub(crate) state: PersistentState,
    pub(crate) frame_state: FrameState,
    pub(crate) settings: NodesSettings,
    pub(crate) nodes: HashMap<usize, Node>,
    pub(crate) pins: HashMap<usize, Pin>,
    pub(crate) links: HashMap<usize, Link>,
}

impl NodesContext {
    pub fn show(
        &mut self,
        nodes_data: impl IntoIterator<Item = VisNode>,
        links_data: impl IntoIterator<Item = VisLink>,
        ui: &mut egui::Ui,
        link_validator: &LinkValidationCallback,
    ) -> egui::Response {
        self.frame_state.reset(ui);

        // 2. Convert Input Data
        let mut node_specs = HashMap::new();
        let mut link_specs = Vec::new();
        let mut current_pins_for_frame: HashMap<usize, PinSpec> = HashMap::new();

        // === NEU: Farbmapping für Relationstypen ===
        // Dieses Mapping könnte auch aus Style geladen werden, aber für's Erste hier.
        let get_color_for_relation = |relation_type: &str| -> Color32 {
            match relation_type {
                "Family" => Color32::ORANGE,
                "Friendship" => Color32::GREEN,
                _ => Color32::GRAY, // Default/Fallback
            }
        };
        // =========================================

        // === MODIFIED: Verarbeitet jetzt `logical_pins` ===
        for vis_node in nodes_data {
            let node_id = vis_node.id;
            let mut pins_for_this_node: Vec<PinSpec> = Vec::new();

            for logical_pin in &vis_node.logical_pins {
                let pin_id = generate_pin_id(node_id, &logical_pin.identifier);
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

            // Erstelle den NodeSpec wie bisher
            let node_args = NodeArgs::default();

            // === KORREKTUR FÜR E0609: Verwende to_srgba() ===
            let node_color_srgba = vis_node.color.to_srgba(); // Konvertiere Bevy Color zu Srgba

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
                    args: node_args, // Standard NodeArgs
                    subtitle: format!("E:{:?}", vis_node.entity),
                    time: None,
                    duration: None,
                    active: false, // Wird später gesetzt
                },
            );
        }

        // Verarbeite eingehende Link-Daten
        for vis_link in links_data {
            // Überprüfe, ob Start- und End-Pin in diesem Frame (noch) existieren
            if current_pins_for_frame.contains_key(&vis_link.start_pin_id)
                && current_pins_for_frame.contains_key(&vis_link.end_pin_id)
            {
                let link_base_color = if let Some(start_pin_spec) =
                    current_pins_for_frame.get(&vis_link.start_pin_id)
                {
                    get_color_for_relation(&start_pin_spec.relation_type)
                } else {
                    Color32::DARK_GRAY // Fallback, sollte nicht passieren
                };
                let link_style = LinkStyleArgs {
                    // --- NEU: Farbe für Link setzen ---
                    base: Some(link_base_color),
                    // --- ALT: Farbe direkt aus VisLink ---
                    // base: Some(Color32::from_rgba_premultiplied(/*...*/)),
                    ..Default::default() // Hover/Selected etc. erstmal default
                };
                link_specs.push(LinkSpec {
                    id: vis_link.id,
                    start_pin_index: vis_link.start_pin_id,
                    end_pin_index: vis_link.end_pin_id,
                    style: link_style,
                });
            } else {
                // Logge Warnung oder handle den Fall, dass ein Link auf nicht mehr existente Pins verweist
                // Zum Beispiel: Diesen Link nicht erstellen oder markieren.
                // bevy::log::warn!("Link {} verweist auf ungültige Pins und wird übersprungen.", vis_link.id);
            }
        }

        // 3. Definiere den Canvas-Bereich und erstelle Interaktion dafür
        let canvas_rect = self.frame_state.canvas_rect_screen_space;
        let canvas_interact_response = ui.interact(
            canvas_rect,
            ui.id().with("NodeCanvasInteractor"),
            egui::Sense::click_and_drag(),
        );

        // === MODIFIED: `child_ui` entfernt, stattdessen Clipping und direktes Malen ===
        // 4. Zeichne Canvas Hintergrund & Grid direkt im `ui` aber mit Clipping
        let painter = ui.painter_at(canvas_rect); // Painter geclippt auf den Canvas-Rect
        painter.rect_filled(
            canvas_rect,
            CornerRadius::ZERO, // Verwende das umbenannte CornerRadius
            self.settings.style.colors[ColorStyle::GridBackground as usize],
        );
        if (self.settings.style.flags & StyleFlags::GridLines as usize) != 0 {
            // Zeichne das Grid direkt im `ui`, es wird durch `painter_at` oder globales ClipRect begrenzt.
            draw_grid(self, canvas_rect.size(), ui);
        }

        // 5. Populate Internal State (Nodes/Pins/Links)
        {
            let mut node_ids_this_frame: Vec<usize> = node_specs.keys().copied().collect();
            self.state
                .node_depth_order
                .retain(|id| node_ids_this_frame.contains(id));
            node_ids_this_frame.retain(|id| !self.state.node_depth_order.contains(id));
            node_ids_this_frame.sort_unstable();
            self.state.node_depth_order.extend(node_ids_this_frame);

            // *** WICHTIG: Übergebe jetzt das geclippte `ui` an add_node/add_link ***
            // Sie fügen ihre Shapes zum Painter des *übergebenen* UI hinzu.
            self.nodes.clear();
            self.pins.clear(); // Leere auch Pins, da sie in add_node neu erzeugt/gefunden werden
                               // Collect all node specs we need to process first, to avoid borrowing conflicts
            let nodes_to_add: Vec<(usize, NodeSpec)> = self
                .state
                .node_depth_order
                .iter()
                .filter_map(|node_id| {
                    node_specs.get(node_id).map(|node_spec| {
                        let mut spec_clone = node_spec.clone();
                        spec_clone.active = self.state.selected_node_indices.contains(node_id);
                        (*node_id, spec_clone)
                    })
                })
                .collect();

            // Process all nodes after collecting them
            for (_node_id, spec_clone) in nodes_to_add {
                // `add_node` füllt `frame_state.nodes_tmp` und `frame_state.pins_tmp`
                self.add_node(spec_clone, ui);
            }
            // Kopiere temporäre Nodes/Pins nach Abschluss
            self.nodes = std::mem::take(&mut self.frame_state.nodes_tmp);
            self.pins = std::mem::take(&mut self.frame_state.pins_tmp);

            self.links.retain(|_link_id, link| {
                self.pins.contains_key(&link.spec.start_pin_index)
                    && self.pins.contains_key(&link.spec.end_pin_index)
            });
            for link_spec in link_specs.iter() {
                self.add_link(link_spec.clone(), ui); // Übergib Haupt-UI
            }
        }

        // 6. Interaction Processing
        let hover_pos = canvas_interact_response.hover_pos();
        ui.ctx().input(|io| {
            self.state.interaction_state = self.state.interaction_state.update(
                io,
                hover_pos,
                self.settings.io.emulate_three_button_mouse,
                self.settings.io.link_detatch_with_modifier_click,
                self.settings.io.alt_mouse_button,
            );
        });
        resolve_hover_state(self);
        process_clicks(self);
        click_interaction_update(self, ui, link_validator); // Wichtig: ui übergeben
        if self.state.interaction_state.delete_pressed {
            handle_delete(self);
        }

        // === MODIFIED: Zeichnen jetzt am Ende und im Haupt-UI ===
        // `final_draw` fügt Shapes zum Painter des übergebenen UI hinzu
        final_draw(self, ui); // Wichtig: ui übergeben
        self.collect_events();

        // 7. Draw Canvas Outline (Verwende den `painter`, der auf `canvas_rect` geclippt ist)
        let outline_stroke = Stroke::new(1.0, Color32::WHITE);
        painter.rect_stroke(
            canvas_rect,
            egui::epaint::CornerRadius::ZERO,
            outline_stroke,
            egui::StrokeKind::Outside,
        );
        canvas_interact_response
    }

    // --- Add Node ---
    #[allow(deprecated)] // Erlaube `allocate_ui_at_rect` vorerst
    fn add_node(&mut self, node_spec: NodeSpec, ui: &mut egui::Ui) {
        let node_id = node_spec.id;

        let mut node_args = node_spec.args.clone();

        let [r, g, b, a] = node_spec.color.to_array();
        let title_color_srgb = Srgba::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        );
        node_args.titlebar = Some(Color32::from_rgba_premultiplied(
            (title_color_srgb.red * 255.0).round() as u8,
            (title_color_srgb.green * 255.0).round() as u8,
            (title_color_srgb.blue * 255.0).round() as u8,
            (title_color_srgb.alpha * 255.0).round() as u8,
        ));
        // Man könnte auch Selected Titlebar anpassen, z.B. etwas heller/dunkler machen
        // node_args.titlebar_selected = Some(...)
        // =====================================================

        let mut node = Node {
            spec: node_spec.clone(),
            state: self
                .nodes
                .get(&node_id)
                .map_or_else(NodeState::default, |n| n.state.clone()),
        };

        let (color_style, layout_style) = self.settings.style.format_node(node_args); // Hier werden die Args verwendet
        node.state.color_style = color_style;
        node.state.layout_style = layout_style;

        // Shapes direkt im übergebenen UI-Painter registrieren
        node.state.background_shape = Some(ui.painter().add(egui::Shape::Noop));
        node.state.titlebar_shape = Some(ui.painter().add(egui::Shape::Noop));
        node.state.outline_shape = Some(ui.painter().add(egui::Shape::Noop));
        node.state.pin_indices.clear();

        // Verwende die spec.origin (Grid Space) um die Screen-Position zu finden
        let node_screen_pos = self.grid_space_to_screen_space(node.spec.origin);
        let initial_rect_guess = Rect::from_min_size(node_screen_pos, egui::vec2(150.0, 50.0)); // Größere initiale Schätzung?

        // Variable für title_response.rect deklarieren
        let mut title_rect_in_node_space = Rect::ZERO;

        // Alloziere UI an der berechneten Screen-Position
        let response = ui.allocate_ui_at_rect(initial_rect_guess, |ui| {
            // <-- allocate_ui_at_rect beibehalten
            // Rahmen für Padding etc.
            #[allow(deprecated)] // Erlaube Frame::none
            Frame::none() // <-- Frame::none beibehalten
                .inner_margin(node.state.layout_style.padding)
                .show(ui, |ui| {
                    // Titel Sektion
                    let title_response = ui
                        .vertical(|ui| {
                            ui.strong(node.spec.name.clone());
                            if !node.spec.subtitle.is_empty() {
                                ui.label(node.spec.subtitle.clone());
                            }
                        })
                        .response;

                    // === KORREKTUR E0425 ===
                    // Speichere das Rect relativ zum UI-Start (innerhalb des Frames)
                    // title_rect_in_node_space wird dann außerhalb dieses Closures verwendet
                    title_rect_in_node_space = title_response.rect;

                    ui.separator();

                    // Pin Sektion
                    egui::Grid::new(format!("node_pins_{}", node.spec.id))
                        .num_columns(2)
                        .spacing(egui::vec2(8.0, 4.0))
                        .show(ui, |ui| {
                            let pins = node.spec.attributes.clone();
                            let (input_pins, output_pins): (Vec<_>, Vec<_>) =
                                pins.into_iter().partition(|p| p.kind == PinType::Input);

                            ui.vertical(|ui| {
                                // Linke Spalte (Input)
                                ui.set_min_width(50.0);
                                for pin_spec in input_pins {
                                    self.add_pin(pin_spec.clone(), &mut node, ui);
                                } // Clone Spec
                            });
                            ui.vertical(|ui| {
                                // Rechte Spalte (Output)
                                ui.set_min_width(50.0);
                                for pin_spec in output_pins {
                                    ui.with_layout(Layout::top_down(egui::Align::Max), |ui| {
                                        self.add_pin(pin_spec.clone(), &mut node, ui);
                                        // Clone Spec
                                    });
                                }
                            });
                            ui.end_row();
                        });
                }); // Ende Frame::show
                    // Gib das Gesamt-Rect der UI zurück (wird von allocate_ui_at_rect verwendet)
        });

        // Das `response` von `allocate_ui_at_rect` enthält das finale Rect des Node-Inhalts
        let final_node_rect_screen = response.response.rect;

        // Update Node-Zustand basierend auf finalem Screen Rect
        node.state.size = final_node_rect_screen.size();
        node.state.rect = Rect::from_min_size(node.spec.origin, node.state.size); // Update Grid Space Rect

        // Konvertiere das lokal gespeicherte title_rect in den Node-Koordinatenraum (relativ zum Node Grid Ursprung)
        // `final_node_rect_screen.min` ist die linke obere Ecke des Node-Inhalts auf dem Bildschirm
        // `title_rect_in_node_space.min` ist relativ zu `final_node_rect_screen.min`
        node.state.title_bar_content_rect =
            title_rect_in_node_space.translate(-final_node_rect_screen.min.to_vec2());

        if ui.rect_contains_pointer(final_node_rect_screen) {
            self.frame_state
                .node_indices_overlapping_with_mouse
                .push(node_id);
        }
        self.frame_state.nodes_tmp.insert(node_id, node);
    }

    // --- Add Pin ---
    fn add_pin(&mut self, pin_spec: PinSpec, node: &mut Node, ui: &mut egui::Ui) {
        let pin_id = pin_spec.id;
        let response = ui.label(pin_spec.name.clone());

        // Klonen um mutable borrow auf self.pins zu vermeiden, wenn wir `Pin` holen
        let pin_spec_clone = pin_spec;

        let mut pin = Pin {
            spec: pin_spec_clone, // Use the clone
            state: self
                .pins
                .get(&pin_id)
                .map_or_else(PinState::default, |p| p.state.clone()),
        };

        // === KORREKTUR unused variables ===
        // let _label_rect_relative_to_ui_cursor = response.rect; // Wird nicht direkt verwendet
        // let _parent_min = ui.min_rect().min; // Wird nicht direkt verwendet

        let label_screen_rect = response.rect;
        // Korrekte Berechnung des relativen Rects
        let _node_origin_screen = self.grid_space_to_screen_space(node.spec.origin); // Muss node state/spec verwenden
        let _node_layout_padding = node.state.layout_style.padding;

        // Titelhöhe berücksichtigen, falls Pins unter dem Titel beginnen
        // Diese Berechnung ist komplex, vereinfacht: Annahme, Pin-Label Rect ist bereits korrekt im Node-Layout platziert.
        let label_rect_relative_to_node_origin = egui::Rect::from_min_max(
            self.screen_space_to_grid_space(label_screen_rect.min) - node.spec.origin.to_vec2(),
            self.screen_space_to_grid_space(label_screen_rect.max) - node.spec.origin.to_vec2(),
        );

        pin.state.parent_node_idx = node.spec.id;
        pin.state.attribute_rect = label_rect_relative_to_node_origin;
        pin.state.color_style = self.settings.style.format_pin(pin.spec.style_args.clone());
        pin.state.shape_gui = Some(ui.painter().add(egui::Shape::Noop));

        if !node.state.pin_indices.contains(&pin_id) {
            node.state.pin_indices.push(pin_id);
        }

        let pin_interaction_rect_screen = self.get_pin_interaction_rect_screen(&pin);
        // _ui wird benötigt für rect_contains_pointer
        if ui.rect_contains_pointer(pin_interaction_rect_screen) {
            // Hover-Effekte für das *Shape*, nicht das Label, werden in resolve_hovered_pin behandelt
        }

        // Klick auf Label (eher selten für Link-Start)
        if ui
            .interact(
                response.rect,
                ui.id().with(pin.spec.id),
                egui::Sense::click(),
            )
            .clicked()
        {
            // log::trace!("Pin Label {:?} clicked", pin_id);
        }

        self.frame_state.pins_tmp.insert(pin_id, pin);
    }

    // Behält bestehende Links bei oder fügt neue hinzu
    fn add_link(&mut self, link_spec: LinkSpec, ui: &mut egui::Ui) {
        let link_id = link_spec.id;

        // Füge den Link hinzu oder aktualisiere seinen Spec
        let entry = self.links.entry(link_id).or_insert_with(|| Link {
            spec: link_spec.clone(),
            state: LinkState::default(), // Beginne mit Default State
        });

        // Aktualisiere den Spec, falls sich was geändert hat (Start/End Pin, ID bleibt)
        entry.spec = link_spec.clone(); // Clone LinkSpec um sicherzustellen, dass es aktuell ist

        // Aktualisiere immer den Style und den Shape Index
        entry.state.style = self.settings.style.format_link(link_spec.style.clone());
        entry.state.shape = Some(ui.painter().add(egui::Shape::Noop));
    }

    pub fn get_pin(&self, pin_id: usize) -> Option<&Pin> {
        self.pins.get(&pin_id)
    }
    // Helfer, um das Klick-/Hover-Rechteck für einen Pin zu bekommen
    fn get_pin_interaction_rect_screen(&self, pin: &Pin) -> egui::Rect {
        let radius = self.settings.style.pin_hover_radius; // Verwende Hover-Radius für Interaktion
                                                           // Position wird im `draw_pin` aktualisiert, hole die letzte bekannte Position oder berechne neu
        let pin_pos_screen = self.get_screen_space_pin_coordinates(pin);
        egui::Rect::from_center_size(pin_pos_screen, egui::vec2(radius * 2.0, radius * 2.0))
    }
    // Gibt die *aktuelle* Screen-Space-Position eines Pins zurück
    // Wichtig, da sich die Node-Position durch Dragging ändern kann
    pub fn get_screen_space_pin_coordinates(&self, pin: &Pin) -> egui::Pos2 {
        let Some(parent_node) = self.nodes.get(&pin.state.parent_node_idx) else {
            return pin.state.pos; // Fallback auf gespeicherte Position, falls Node weg ist
        };

        // Node Rect ist im Grid Space
        let node_rect_grid = parent_node.state.rect;

        // Pin Attribut Rect ist relativ zum Node Grid Origin
        let attr_rect_grid_relative = pin.state.attribute_rect;
        let attr_rect_grid_absolute =
            attr_rect_grid_relative.translate(node_rect_grid.min.to_vec2()); // Addiere Node-Ursprung

        // Wandle die Grid-Rects in Screen-Rects um
        let node_rect_screen = egui::Rect::from_min_max(
            self.grid_space_to_screen_space(node_rect_grid.min),
            self.grid_space_to_screen_space(node_rect_grid.max),
        );
        let attr_rect_screen = egui::Rect::from_min_max(
            self.grid_space_to_screen_space(attr_rect_grid_absolute.min),
            self.grid_space_to_screen_space(attr_rect_grid_absolute.max),
        );

        // Verwende die Style-Methode mit den aktuellen Screen-Rects
        self.settings.style.get_screen_space_pin_coordinates(
            &node_rect_screen,
            &attr_rect_screen, // Das ist das Label-Rect im Screen Space
            pin.spec.kind,
        )
    }
    // --- Drawing Methods ---

    // --- Coordinate Systems ---
    pub fn screen_space_to_grid_space(&self, v: egui::Pos2) -> egui::Pos2 {
        // Von Screen-Koordinaten (relativ zum Fenster/Canvas) zu Grid-Koordinaten (virtueller Raum mit Panning)
        v - self.frame_state.canvas_origin_screen_space() - self.state.panning
    }
    pub fn grid_space_to_screen_space(&self, v: egui::Pos2) -> egui::Pos2 {
        // Von Grid-Koordinaten (virtueller Raum mit Panning) zu Screen-Koordinaten (relativ zum Fenster/Canvas)
        v + self.state.panning + self.frame_state.canvas_origin_screen_space()
    }
    pub fn editor_space_to_screen_space(&self, v: egui::Pos2) -> egui::Pos2 {
        // Von Editor-Koordinaten (z.B. 0,0 ist oben links im Canvas, ignoriert Panning) zu Screen-Koordinaten
        v + self.frame_state.canvas_origin_screen_space()
    }

    // --- Resolves (Mit eingefügtem Code) ---

    // === MODIFIED: TODO 4 - Event Sammlung angepasst ===
    fn collect_events(&mut self) {

        // === KORREKTUR: LinkRemoved wird jetzt nur noch via handle_delete gesendet ===
        // Das `frame_state.deleted_link_idx` wurde früher für das temporäre Entfernen beim Detach verwendet,
        // was wir jetzt durch das Speichern von `modifying_link_id` und Überspringen im Draw ersetzen.
        /*
        if self.frame_state.deleted_link_idx.is_some() {
            // Dieses Flag wird nicht mehr gesetzt. LinkRemoved kommt aus handle_delete.
        }*/

        // Beispiel: Wenn man noch spezifische Events für "dropped" etc. bräuchte:
        /*
         if self.frame_state.element_state_change.link_dropped {
             // Eventuell ein GraphChange::LinkDropped(start_pin_id) senden?
        }
        */
    }

    // --- Getter Methods ---

    pub fn get_selected_nodes(&self) -> Vec<usize> {
        self.state.selected_node_indices.clone()
    }

    pub fn get_selected_links(&self) -> Vec<usize> {
        self.state.selected_link_indices.clone()
    }

    pub fn clear_node_selection(&mut self) {
        self.state.selected_node_indices.clear();
    }

    pub fn clear_link_selection(&mut self) {
        self.state.selected_link_indices.clear();
    }

    pub fn active_attribute(&self) -> Option<usize> {
        self.frame_state.active_pin
    }

    pub fn link_started(&self) -> Option<usize> {
        if self.frame_state.element_state_change.link_started {
            self.state
                .click_interaction_state
                .link_creation
                .start_pin_idx
        } else {
            None
        }
    }

    pub fn link_dropped(&self, include_detached: bool) -> Option<usize> {
        if self.frame_state.element_state_change.link_dropped {
            let creation_state = &self.state.click_interaction_state.link_creation;
            // Nur melden, wenn nicht durch Detach ausgelöst (oder wenn Detach explizit gewünscht ist)
            if include_detached || creation_state.link_creation_type != LinkCreationType::FromDetach
            {
                return creation_state.start_pin_idx;
            }
        }
        None
    }

    // === MODIFIED: Gibt die *erfolgreich verbundenen* Pins zurück, wenn `link_created` wahr ist ===
    // Gibt (Output Pin ID, Input Pin ID) zurück.
    pub fn link_created(&self) -> Option<(usize, usize)> {
        if self.frame_state.element_state_change.link_created {
            let creation_state = &self.state.click_interaction_state.link_creation;
            // Beide Pins müssen vorhanden sein
            if let (Some(start_pin_id), Some(end_pin_id)) =
                (creation_state.start_pin_idx, creation_state.end_pin_index)
            {
                // Bestimme Start-/End-Pin basierend auf PinType
                let start_pin_kind = self
                    .pins
                    .get(&start_pin_id)
                    .map_or(PinType::None, |p| p.spec.kind);
                let (output_pin_id, input_pin_id) = if start_pin_kind == PinType::Output {
                    (start_pin_id, end_pin_id)
                } else {
                    (end_pin_id, start_pin_id) // Umkehren falls von Input gestartet
                };
                return Some((output_pin_id, input_pin_id));
            }
        }
        None
    }

    // Gibt optional den Pin zurück, an den erfolgreich drangesnapped wurde (Ende der LinkCreation)
    pub fn link_snapped_to_pin(&self) -> Option<usize> {
        if self.frame_state.element_state_change.link_created {
            self.state
                .click_interaction_state
                .link_creation
                .end_pin_index
        } else {
            None
        }
    }

    pub fn link_destroyed(&self) -> Option<usize> {
        self.frame_state.deleted_link_idx
    }

    pub fn get_changes(&self) -> &Vec<GraphChange> {
        &self.frame_state.graph_changes
    }

    pub fn get_panning(&self) -> egui::Vec2 {
        self.state.panning
    }

    pub fn reset_panning(&mut self, panning: egui::Vec2) {
        self.state.panning = panning;
    }

    pub fn get_node_dimensions(&self, id: usize) -> Option<egui::Vec2> {
        self.nodes.get(&id).map(|n| n.state.rect.size())
    }

    pub fn is_node_just_selected(&self) -> bool {
        self.frame_state.just_selected_node
    }

    pub fn set_node_draggable(&mut self, node_id: usize, draggable: bool) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.state.draggable = draggable;
        }
    }
} // === Ende impl NodesContext ===

// === NodeState Helfer (mit Context für Konvertierung) ===
impl NodeState {
    // Gibt das Rect des Titelbalken-Bereichs (inkl. Padding) im Grid-Space zurück
    fn get_node_title_rect_grid(&self) -> egui::Rect {
        let title_height_with_padding =
            self.title_bar_content_rect.height() + self.layout_style.padding.y * 2.0; // Höhe des Inhalts + oberes/unteres Padding
        egui::Rect::from_min_size(
            self.rect.min, // Beginnt am Ursprung des Node-Rects
            egui::vec2(self.rect.width(), title_height_with_padding.max(0.0)), // Nimmt die volle Breite, aber begrenzte Höhe
        )
    }

    // Konvertiert das Grid-Space Titel-Rect in Screen-Space
    pub(crate) fn get_node_title_rect_screen(&self, context: &NodesContext) -> egui::Rect {
        let grid_rect = self.get_node_title_rect_grid();
        egui::Rect::from_min_max(
            // MinMax verwenden, um sicherzugehen
            context.grid_space_to_screen_space(grid_rect.min),
            context.grid_space_to_screen_space(grid_rect.max),
        )
    }
}
