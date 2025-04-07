// src/dev_tools/node_graph/context.rs
use bevy::color::Srgba; // Direkter Import für Srgba Konvertierung
use bevy::log;
use bevy::math::Vec2 as BevyVec2; // Für Umwandlung
use bevy::prelude::Resource; // Bevy Typen
use bevy_egui::egui::{self, Color32, CornerRadius, StrokeKind}; // Egui Typen, inkl. CornerRadius
use bevy_egui::egui::{Frame, Layout, Pos2, Rect, Sense, Stroke, Vec2, WidgetText};
use derivative::Derivative;
use std::collections::HashMap;

// Use-Anweisungen für die UI-Module
use super::ui_data::*;
use super::ui_link::*;
use super::ui_node::*;
use super::ui_pin::*;
use super::ui_style::{ColorStyle, Style, StyleFlags};

pub use {
    super::ui_node::NodeArgs,
    super::ui_pin::{AttributeFlags, PinShape, PinStyleArgs, PinType},
};

// Konstante für die Pin ID Generierung (Beispiel)
const PIN_ID_MULTIPLIER: usize = 10; // Oder eine andere ausreichend große Zahl
const INPUT_PIN_OFFSET: usize = 0;
const OUTPUT_PIN_OFFSET: usize = 1;

#[derive(Debug, Clone)]
pub enum GraphChange {
    LinkCreated(usize, usize),  // Start Pin ID, End Pin ID
    LinkRemoved(usize),         // Link ID
    NodeMoved(usize, BevyVec2), // Node ID, Grid Space Position
    NodeRemoved(usize),         // Node ID
    // === NEU: Event für Link Erstellung durch User-Aktion ===
    NewLinkRequested(usize, usize), // Start Pin ID, End Pin ID
}

#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct InteractionState {
    mouse_pos: egui::Pos2,
    mouse_delta: egui::Vec2,
    left_mouse_clicked: bool,
    left_mouse_released: bool,
    alt_mouse_clicked: bool,
    left_mouse_dragging: bool,
    alt_mouse_dragging: bool,
    mouse_in_canvas: bool,
    link_detatch_with_modifier_click: bool,
    delete_pressed: bool,
}

#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct PersistentState {
    interaction_state: InteractionState,
    selected_node_indices: Vec<usize>,
    selected_link_indices: Vec<usize>,
    node_depth_order: Vec<usize>,
    panning: egui::Vec2,
    #[derivative(Default(value = "ClickInteractionType::None"))]
    click_interaction_type: ClickInteractionType,
    click_interaction_state: ClickInteractionState,
    // === NEU: Einfacher Zähler für interne Link IDs ===
    // HINWEIS: Nicht persistent über Neustarts! Nur für die Session.
    // Eine bessere Lösung bräuchte ein Asset oder eine dedizierte Ressource.
    next_link_id: usize,
}

#[derive(Derivative)]
#[derivative(Debug, Default)] // Korrekte Syntax
pub struct FrameState {
    #[derivative(Default(value = "[[0.0; 2].into(); 2].into()"))]
    canvas_rect_screen_space: egui::Rect,
    node_indices_overlapping_with_mouse: Vec<usize>,
    occluded_pin_indices: Vec<usize>,
    hovered_node_index: Option<usize>,
    interactive_node_index: Option<usize>,
    hovered_link_idx: Option<usize>,
    hovered_pin_index: Option<usize>,
    hovered_pin_flags: usize,
    deleted_link_idx: Option<usize>,
    snap_link_idx: Option<usize>,
    element_state_change: ElementStateChange,
    active_pin: Option<usize>,
    graph_changes: Vec<GraphChange>, // Enthält jetzt auch NewLinkRequested
    pins_tmp: HashMap<usize, Pin>,
    nodes_tmp: HashMap<usize, Node>,
    just_selected_node: bool,
}

#[derive(Debug, Default)]
pub struct NodesSettings {
    pub io: IO,
    pub style: Style,
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

#[derive(Resource, Derivative)]
#[derivative(Debug, Default)]
pub struct NodesContext {
    state: PersistentState,
    frame_state: FrameState,
    settings: NodesSettings,
    nodes: HashMap<usize, Node>, // Behält erzeugte Nodes (Persistent State)
    pins: HashMap<usize, Pin>,   // Behält erzeugte Pins (Persistent State)
    links: HashMap<usize, Link>, // Behält erzeugte Links (Persistent State)
}

impl NodesContext {
    // === BEGINN: `show`-Methode ===
    pub fn show(
        &mut self,
        nodes_data: impl IntoIterator<Item = VisNode>,
        links_data: impl IntoIterator<Item = VisLink>,
        ui: &mut egui::Ui,
    ) -> egui::Response {
        self.frame_state.reset(ui); // 1. Reset

        // === MODIFIED: TODO 1 - Pin Implementation Start ===
        // 2. Convert Input Data
        let mut node_specs = HashMap::new(); // Key: node ID (z.B. entity index)
        let mut link_specs = Vec::new(); // Wird von links_data gefüllt
        let mut current_pins_for_frame: HashMap<usize, PinSpec> = HashMap::new(); // Sammelt *alle* Pins dieses Frames

        for vis_node in nodes_data {
            // *** ID-Generierung für Pins ***
            // WICHTIG: Dies muss pro Frame *konsistent* sein und idealerweise über
            // Frames hinweg stabil bleiben, wenn sich die Node-Daten nicht ändern.
            // Entity-Index als Basis ist okay für die Demo, aber nicht für persistente Graphen.
            // Stattdessen könnte eine eindeutige Pin-Kennung aus der Simulation kommen
            // (z.B. wenn Pins Komponenten wären) oder durch Kombination von Node-ID
            // und einem Pin-Index/Namen generiert werden.

            let input_pin_id = vis_node.id.wrapping_mul(PIN_ID_MULTIPLIER) + INPUT_PIN_OFFSET;
            let output_pin_id = vis_node.id.wrapping_mul(PIN_ID_MULTIPLIER) + OUTPUT_PIN_OFFSET;

            // Erstelle die PinSpecs für diesen Node
            let mut pins_for_this_node: Vec<PinSpec> = Vec::new();

            // Beispiel: Ein Input-Pin
            let input_pin = PinSpec {
                id: input_pin_id,
                kind: PinType::Input,
                name: "In".to_string(),
                flags: AttributeFlags::None as usize, // Inputs erlauben normalerweise kein *Starten* von Links
                style_args: PinStyleArgs {
                    shape: Some(PinShape::CircleFilled),
                    ..Default::default()
                }, // Beispiel-Style
                ..Default::default()
            };
            pins_for_this_node.push(input_pin.clone());
            current_pins_for_frame.insert(input_pin_id, input_pin);

            // Beispiel: Ein Output-Pin
            let output_pin = PinSpec {
                id: output_pin_id,
                kind: PinType::Output,
                name: "Out".to_string(),
                // Flags, die Interaktion ermöglichen
                flags: (AttributeFlags::EnableLinkCreationOnSnap as usize)
                    | (AttributeFlags::EnableLinkDetachWithDragClick as usize),
                style_args: PinStyleArgs {
                    shape: Some(PinShape::TriangleFilled),
                    ..Default::default()
                }, // Beispiel-Style
                ..Default::default()
            };
            pins_for_this_node.push(output_pin.clone());
            current_pins_for_frame.insert(output_pin_id, output_pin);

            // Erstelle den NodeSpec wie bisher, füge die generierten PinSpecs hinzu
            let color_srgba: Srgba = vis_node.color.into();
            let node_args = NodeArgs {
                background: Some(Color32::from_rgba_premultiplied(
                    (color_srgba.red * 255.0).round() as u8,
                    (color_srgba.green * 255.0).round() as u8,
                    (color_srgba.blue * 255.0).round() as u8,
                    (color_srgba.alpha * 255.0).round() as u8,
                )),
                ..Default::default()
            };
            node_specs.insert(
                vis_node.id, // Key ist die Node-ID
                NodeSpec {
                    id: vis_node.id,
                    name: vis_node.name,
                    origin: egui::pos2(vis_node.position.x, vis_node.position.y),
                    attributes: pins_for_this_node, // *** Hier die PinSpecs übergeben ***
                    args: node_args,
                    subtitle: format!("E:{:?}", vis_node.entity),
                    time: None,
                    duration: None,
                    active: self.state.selected_node_indices.contains(&vis_node.id),
                },
            );
        }
        // === MODIFIED: TODO 1 - Pin Implementation End ===

        // Verarbeite eingehende Link-Daten
        for vis_link in links_data {
            // Überprüfe, ob Start- und End-Pin in diesem Frame (noch) existieren
            if current_pins_for_frame.contains_key(&vis_link.start_pin_id)
                && current_pins_for_frame.contains_key(&vis_link.end_pin_id)
            {
                let color_srgba: Srgba = vis_link.color.into();
                let link_style = LinkStyleArgs {
                    base: Some(Color32::from_rgba_premultiplied(
                        (color_srgba.red * 255.0).round() as u8,
                        (color_srgba.green * 255.0).round() as u8,
                        (color_srgba.blue * 255.0).round() as u8,
                        (color_srgba.alpha * 255.0).round() as u8,
                    )),
                    ..Default::default()
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
            self.draw_grid(canvas_rect.size(), ui);
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
            // Verwende das Haupt-UI ctx
            self.state.interaction_state = self.state.interaction_state.update(
                io,
                hover_pos,
                self.settings.io.emulate_three_button_mouse,
                self.settings.io.link_detatch_with_modifier_click,
                self.settings.io.alt_mouse_button,
            );
        });
        self.resolve_hover_state();
        self.process_clicks();
        // Übergebe das Haupt-UI, interne Operationen müssen relativ zu diesem sein
        self.click_interaction_update(ui);
        if self.state.interaction_state.delete_pressed {
            self.handle_delete();
        }

        // === MODIFIED: Zeichnen jetzt am Ende und im Haupt-UI ===
        // `final_draw` fügt Shapes zum Painter des übergebenen UI hinzu
        self.final_draw(ui); // Übergib Haupt-UI

        self.collect_events(); // Sammelt Änderungen

        // 7. Draw Canvas Outline (Verwende den `painter`, der auf `canvas_rect` geclippt ist)
        let outline_stroke = Stroke::new(1.0, Color32::WHITE);
        painter.rect_stroke(
            canvas_rect,
            CornerRadius::ZERO, // Verwende umbenanntes CornerRadius
            outline_stroke,
            StrokeKind::Outside, // StrokeKind hinzugefügt (Outside, Inside oder Centered)
        );

        // 8. Return Response (von der Canvas-Interaktion)
        canvas_interact_response // Gibt die Response des Haupt-Canvas zurück
    }

    // --- Add Node ---
    #[allow(deprecated)] // Erlaube `allocate_ui_at_rect` vorerst
    fn add_node(&mut self, node_spec: NodeSpec, ui: &mut egui::Ui) {
        let node_id = node_spec.id;

        let mut node = Node {
            spec: node_spec.clone(),
            state: self
                .nodes
                .get(&node_id)
                .map_or_else(NodeState::default, |n| n.state.clone()),
        };

        let (color_style, layout_style) = self.settings.style.format_node(node.spec.args.clone());
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
        let node_origin_screen = self.grid_space_to_screen_space(node.spec.origin); // Muss node state/spec verwenden
        let node_layout_padding = node.state.layout_style.padding;

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
    // Helfer, um das Klick-/Hover-Rechteck für einen Pin zu bekommen
    fn get_pin_interaction_rect_screen(&self, pin: &Pin) -> egui::Rect {
        let radius = self.settings.style.pin_hover_radius; // Verwende Hover-Radius für Interaktion
                                                           // Position wird im `draw_pin` aktualisiert, hole die letzte bekannte Position oder berechne neu
        let pin_pos_screen = self.get_screen_space_pin_coordinates(pin);
        egui::Rect::from_center_size(pin_pos_screen, egui::vec2(radius * 2.0, radius * 2.0))
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

    // --- Drawing Methods ---
    fn draw_grid(&self, _canvas_size: egui::Vec2, ui: &mut egui::Ui) {
        /* Vollständige Implementierung wie oben */
        let painter = ui.painter();
        let style = &self.settings.style;
        let spacing = style.grid_spacing;
        let line_stroke = egui::Stroke::new(1.0, style.colors[ColorStyle::GridLine as usize]);
        let canvas_origin_screen = self.editor_space_to_screen_space(egui::Pos2::ZERO); // Wo der (0,0) Punkt des Grids auf dem Bildschirm liegt
        let visible_rect = ui.clip_rect(); // Sichtbarer Bereich der UI

        // Vertikale Linien
        let x_min_grid_for_screen =
            visible_rect.min.x - canvas_origin_screen.x - self.state.panning.x; // Korrektur: Verwende grid space für Berechnung
        let x_start_index = (x_min_grid_for_screen / spacing).floor() as i32;
        // Erste *sichtbare* vertikale Linie im Grid Space bestimmen
        let mut grid_x = x_start_index as f32 * spacing; // Grid Koordinate
        loop {
            let screen_x = self.grid_space_to_screen_space(egui::pos2(grid_x, 0.0)).x;
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
            visible_rect.min.y - canvas_origin_screen.y - self.state.panning.y; // Korrektur: Verwende grid space
        let y_start_index = (y_min_grid_for_screen / spacing).floor() as i32;
        // Erste *sichtbare* horizontale Linie im Grid Space bestimmen
        let mut grid_y = y_start_index as f32 * spacing; // Grid Koordinate
        loop {
            let screen_y = self.grid_space_to_screen_space(egui::pos2(0.0, grid_y)).y;
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

    fn draw_link(&mut self, link_id: usize, ui: &mut egui::Ui) {
        let link_hovered = self.frame_state.hovered_link_idx == Some(link_id);
        let link_selected = self.state.selected_link_indices.contains(&link_id);

        // Lese den Link (immutable), um auf Pin-Daten zuzugreifen (immutable)
        if let Some(link) = self.links.get(&link_id) {
            if let (Some(start_pin), Some(end_pin)) = (
                self.pins.get(&link.spec.start_pin_index),
                self.pins.get(&link.spec.end_pin_index),
            ) {
                // Berechne Bezier-Kurve basierend auf den (aktuellen) Pin-Positionen (Screen Space)
                let link_data = LinkBezierData::get_link_renderable(
                    self.get_screen_space_pin_coordinates(start_pin), // Immer neu berechnen
                    self.get_screen_space_pin_coordinates(end_pin),   // Immer neu berechnen
                    start_pin.spec.kind,                              // Richtung wichtig für Bezier
                    self.settings.style.link_line_segments_per_length,
                );

                // Wähle Farbe basierend auf Zustand
                let color = if link_selected {
                    link.state.style.selected
                } else if link_hovered {
                    link.state.style.hovered
                } else {
                    link.state.style.base
                };

                // Zeichne die Linie mit painter.set() auf dem reservierten Shape Index
                if let Some(shape_idx) = link.state.shape {
                    // Muss Some sein, da in add_link gesetzt
                    ui.painter().set(
                        shape_idx,
                        link_data.draw((link.state.style.thickness, color)),
                    );
                }
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

    fn draw_node(&mut self, node_id: usize, ui: &mut egui::Ui) {
        if let Some(node) = self.nodes.get(&node_id) {
            let node_hovered = self.frame_state.hovered_node_index == Some(node_id);
            let is_selected = self.state.selected_node_indices.contains(&node_id);

            // Farben auswählen
            let bg_col = if is_selected {
                node.state.color_style.background_selected
            } else if node_hovered {
                node.state.color_style.background_hovered
            } else {
                node.state.color_style.background
            };
            let title_col = if is_selected {
                node.state.color_style.titlebar_selected
            } else if node_hovered {
                node.state.color_style.titlebar_hovered
            } else {
                node.state.color_style.titlebar
            };
            let outline_col = if node.spec.active {
                self.settings.style.colors[ColorStyle::NodeOutlineActive as usize]
            } else if is_selected {
                node.state.color_style.background_selected
            }
            // Standard-Outline für selektierte
            else {
                node.state.color_style.outline
            };

            let painter = ui.painter(); // Painter vom übergebenen UI
            let screen_rect = egui::Rect::from_min_max(
                self.grid_space_to_screen_space(node.state.rect.min),
                self.grid_space_to_screen_space(node.state.rect.max),
            );
            let screen_title_rect = node.state.get_node_title_rect_screen(self);
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
            if (self.settings.style.flags & StyleFlags::NodeOutline as usize) != 0 {
                if let Some(idx) = node.state.outline_shape {
                    painter.set(
                        idx,
                        egui::Shape::Rect(egui::epaint::RectShape {
                            rect: screen_rect
                                .expand(node.state.layout_style.border_thickness / 2.0),
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
            drop(node); // Release immutable borrow

            for pin_id in pin_ids_to_draw {
                self.draw_pin(pin_id, ui);
            } // Übergebe gleiches UI
        } else {
            log::warn!(
                "Versuche Node {} zu zeichnen, aber nicht gefunden.",
                node_id
            );
        }
    }
    fn draw_pin(&mut self, pin_idx: usize, ui: &mut egui::Ui) {
        let pin_hovered = self.frame_state.hovered_pin_index == Some(pin_idx);
        let pin_data: Option<(egui::Pos2, PinShape, Color32, egui::layers::ShapeIdx, usize)> = // Füge Pin Flags hinzu
            // Immutable borrow um Pin Daten zu lesen
            if let Some(pin) = self.pins.get(&pin_idx) {
                 let screen_pos = self.get_screen_space_pin_coordinates(pin); // Position berechnen

                let draw_color = if pin_hovered {
                    pin.state.color_style.hovered
                } else {
                    pin.state.color_style.background
                 };
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
            self.settings
                .style
                .draw_pin_shape(screen_pos, draw_shape, draw_color, shape_idx, ui);

            // === Aktualisiere Pin State ===
            if let Some(pin_mut) = self.pins.get_mut(&pin_idx) {
                // Update der aktuellen Screen Position des Pins
                pin_mut.state.pos = screen_pos;

                // Wenn dieser Pin gerade gehovert wird, speichere seine Flags für Interaction Checks
                if pin_hovered {
                    self.frame_state.hovered_pin_flags = flags;
                }
            }
        }
    }

    fn draw_temporary_elements(&self, ui: &mut egui::Ui) {
        // === Zeichnen für LinkCreation ===
        if self.state.click_interaction_type == ClickInteractionType::LinkCreation {
            // Hole Start-Pin (sollte existieren, da die Interaktion läuft)
            if let Some(start_pin_id) = self
                .state
                .click_interaction_state
                .link_creation
                .start_pin_idx
            {
                if let Some(start_pin) = self.pins.get(&start_pin_id) {
                    let start_pos = self.get_screen_space_pin_coordinates(start_pin); // Berechne aktuelle Startposition

                    // Bestimme Endposition: Entweder Mausposition oder Position eines gesnappten Pins
                    let end_pos = self
                        .state
                        .click_interaction_state
                        .link_creation
                        .end_pin_index
                        .and_then(|id| self.pins.get(&id))
                        .map_or(self.state.interaction_state.mouse_pos, |p| {
                            self.get_screen_space_pin_coordinates(p)
                        }); // Berechne aktuelle Endposition

                    // Erstelle die Bezier-Daten
                    let link_data = LinkBezierData::get_link_renderable(
                        start_pos,
                        end_pos,
                        start_pin.spec.kind,
                        self.settings.style.link_line_segments_per_length,
                    );

                    // Zeichne die temporäre Linie
                    let temp_link_color = if self
                        .state
                        .click_interaction_state
                        .link_creation
                        .end_pin_index
                        .is_some()
                    {
                        // Farbe, wenn über gültigem Pin gesnapped
                        self.settings.style.colors[ColorStyle::LinkSelected as usize]
                    } else {
                        // Standardfarbe während des Ziehens
                        self.settings.style.colors[ColorStyle::Link as usize]
                    };

                    ui.painter()
                        .add(link_data.draw((self.settings.style.link_thickness, temp_link_color)));
                }
            }
        }

        // === Zeichnen für BoxSelection ===
        if self.state.click_interaction_type == ClickInteractionType::BoxSelection {
            let selection_rect = self.state.click_interaction_state.box_selection;
            // Stelle sicher, dass das Rechteck normalisiert ist (min < max)
            let normalized_rect = egui::Rect::from_min_max(
                selection_rect.min.min(selection_rect.max),
                selection_rect.min.max(selection_rect.max),
            );

            // Zeichne gefülltes Rechteck
            ui.painter().rect_filled(
                normalized_rect,
                CornerRadius::ZERO,
                self.settings.style.colors[ColorStyle::BoxSelector as usize],
            );
            // Zeichne Umriss
            ui.painter().rect_stroke(
                normalized_rect,
                CornerRadius::ZERO,
                egui::Stroke::new(
                    1.0,
                    self.settings.style.colors[ColorStyle::BoxSelectorOutline as usize],
                ),
                StrokeKind::Inside,
            );
        }
    }

    // --- Coordinate Systems ---
    fn screen_space_to_grid_space(&self, v: egui::Pos2) -> egui::Pos2 {
        // Von Screen-Koordinaten (relativ zum Fenster/Canvas) zu Grid-Koordinaten (virtueller Raum mit Panning)
        v - self.frame_state.canvas_origin_screen_space() - self.state.panning
    }
    fn grid_space_to_screen_space(&self, v: egui::Pos2) -> egui::Pos2 {
        // Von Grid-Koordinaten (virtueller Raum mit Panning) zu Screen-Koordinaten (relativ zum Fenster/Canvas)
        v + self.state.panning + self.frame_state.canvas_origin_screen_space()
    }
    fn editor_space_to_screen_space(&self, v: egui::Pos2) -> egui::Pos2 {
        // Von Editor-Koordinaten (z.B. 0,0 ist oben links im Canvas, ignoriert Panning) zu Screen-Koordinaten
        v + self.frame_state.canvas_origin_screen_space()
    }

    // Gibt die *aktuelle* Screen-Space-Position eines Pins zurück
    // Wichtig, da sich die Node-Position durch Dragging ändern kann
    fn get_screen_space_pin_coordinates(&self, pin: &Pin) -> egui::Pos2 {
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

    // --- Resolves (Mit eingefügtem Code) ---
    fn resolve_hover_state(&mut self) {
        if !self.state.interaction_state.mouse_in_canvas {
            self.frame_state.hovered_pin_index = None;
            self.frame_state.hovered_node_index = None;
            self.frame_state.hovered_link_idx = None;
            return;
        }
        // Reihenfolge ist wichtig: Pins > Nodes > Links
        self.resolve_occluded_pins(); // Markiert Pins, die von Nodes verdeckt sind
        self.resolve_hovered_pin(); // Findet den obersten, nicht verdeckten Pin unter der Maus

        // Nur nach Node suchen, wenn *kein* Pin gehovert wird
        if self.frame_state.hovered_pin_index.is_none() {
            self.resolve_hovered_node(); // Findet den obersten Node unter der Maus
        } else {
            self.frame_state.hovered_node_index = None; // Wenn Pin hovered -> kein Node Hover
        }

        // Nur nach Link suchen, wenn *weder* Pin *noch* Node gehovert wird
        if self.frame_state.hovered_pin_index.is_none()
            && self.frame_state.hovered_node_index.is_none()
        {
            self.resolve_hovered_link(); // Findet den Link unter der Maus
        } else {
            self.frame_state.hovered_link_idx = None; // Wenn Pin/Node hovered -> kein Link Hover
        }
    }

    fn resolve_occluded_pins(&mut self) {
        self.frame_state.occluded_pin_indices.clear();
        let depth_stack = &self.state.node_depth_order;
        if depth_stack.len() < 2 {
            return;
        } // Nur nötig, wenn mind. 2 Nodes

        // Iteriere von unten nach oben (ausser dem obersten Node)
        for i in 0..(depth_stack.len() - 1) {
            if let Some(node_below) = self.nodes.get(&depth_stack[i]) {
                // Gehe alle Nodes *über* dem aktuellen Node durch
                for j in (i + 1)..depth_stack.len() {
                    if let Some(node_above) = self.nodes.get(&depth_stack[j]) {
                        // Node Rect ist Grid Space -> Konvertieren zu Screen Space für den Test
                        let screen_rect_above = egui::Rect::from_min_max(
                            self.grid_space_to_screen_space(node_above.state.rect.min),
                            self.grid_space_to_screen_space(node_above.state.rect.max),
                        );

                        // Prüfe jeden Pin des unteren Nodes
                        for pin_id in &node_below.state.pin_indices {
                            if let Some(pin) = self.pins.get(pin_id) {
                                // Pin Position (Screen Space) wird im draw_pin gesetzt/aktualisiert
                                let pin_pos_screen = self.get_screen_space_pin_coordinates(pin);
                                if screen_rect_above.contains(pin_pos_screen) {
                                    // Wenn der Pin vom oberen Node verdeckt wird, markieren
                                    self.frame_state.occluded_pin_indices.push(*pin_id);
                                }
                            }
                        }
                    }
                }
            }
        }
        // Dedupliziere die Liste (optional, falls ein Pin von mehreren Nodes verdeckt werden könnte)
        self.frame_state.occluded_pin_indices.sort_unstable();
        self.frame_state.occluded_pin_indices.dedup();
    }

    fn resolve_hovered_pin(&mut self) {
        self.frame_state.hovered_pin_index = None; // Start with no hovered pin
        let mut smallest_dist_sq = self.settings.style.pin_hover_radius.powi(2);
        let mouse_pos = self.state.interaction_state.mouse_pos;

        // Gehe alle aktuell im Frame existierenden Pins durch
        for (id, pin) in self.pins.iter() {
            // Überspringe, wenn der Pin von einem anderen Node verdeckt ist
            if self.frame_state.occluded_pin_indices.contains(id) {
                continue;
            }

            // pin.state.pos sollte die aktuelle Screen Position sein (wird in draw_pin gesetzt)
            let pin_pos_screen = self.get_screen_space_pin_coordinates(pin);
            let dist_sq = pin_pos_screen.distance_sq(mouse_pos);

            // Prüfe, ob dieser Pin näher ist als der bisher nächste gefundene
            if dist_sq < smallest_dist_sq {
                smallest_dist_sq = dist_sq;
                self.frame_state.hovered_pin_index = Some(*id); // Merke diesen Pin als gehovered
            }
        }
    }

    fn resolve_hovered_node(&mut self) {
        self.frame_state.hovered_node_index = None;
        // === KORREKTUR unused_mut / unused variable ===
        // let mut _max_depth = -1_isize; // Nicht mehr benötigt mit der neuen Logik

        for node_id in self.state.node_depth_order.iter().rev() {
            if let Some(node) = self.nodes.get(node_id) {
                let screen_rect = egui::Rect::from_min_max(
                    self.grid_space_to_screen_space(node.state.rect.min),
                    self.grid_space_to_screen_space(node.state.rect.max),
                );
                if screen_rect.contains(self.state.interaction_state.mouse_pos) {
                    self.frame_state.hovered_node_index = Some(*node_id);
                    return;
                }
            }
        }
    }

    fn resolve_hovered_link(&mut self) {
        self.frame_state.hovered_link_idx = None; // Reset
        let mut smallest_dist_sq = self.settings.style.link_hover_distance.powi(2);
        let mouse_pos = self.state.interaction_state.mouse_pos;

        for (id, link) in self.links.iter() {
            // Hole Start- und End-Pins (existieren sicher, da vorher geprüft)
            if let (Some(start_pin), Some(end_pin)) = (
                self.pins.get(&link.spec.start_pin_index),
                self.pins.get(&link.spec.end_pin_index),
            ) {
                // Ignoriere Link-Hover, wenn Maus über einem der beteiligten Pins ist
                // Dies wird bereits durch die Hover-Priorisierung (Pins > Links) abgedeckt.

                // Berechne Bezier basierend auf aktuellen Pin-Positionen
                let start_pos_screen = self.get_screen_space_pin_coordinates(start_pin);
                let end_pos_screen = self.get_screen_space_pin_coordinates(end_pin);
                let link_data = LinkBezierData::get_link_renderable(
                    start_pos_screen,
                    end_pos_screen,
                    start_pin.spec.kind,
                    self.settings.style.link_line_segments_per_length,
                );

                // Grobe Prüfung: Bounding Box
                let containing_rect = link_data.bezier.get_containing_rect_for_bezier_curve(
                    self.settings.style.link_hover_distance, // Abstand zum Expandieren der Box
                );
                if containing_rect.contains(mouse_pos) {
                    // Feine Prüfung: Abstand zur Kurve
                    let dist_sq = link_data.get_distance_to_cubic_bezier_sq(&mouse_pos); // Nutze quadratischen Abstand
                    if dist_sq < smallest_dist_sq {
                        smallest_dist_sq = dist_sq;
                        self.frame_state.hovered_link_idx = Some(*id);
                    }
                }
            }
        }
    }

    // --- Begin Interaction Methods ---
    fn begin_canvas_interaction(&mut self, start_panning: bool) {
        if self.state.click_interaction_type == ClickInteractionType::None {
            if start_panning {
                self.state.click_interaction_type = ClickInteractionType::Panning;
            } else {
                // Starte Box Selection
                self.state.click_interaction_type = ClickInteractionType::BoxSelection;
                self.state.selected_node_indices.clear(); // Auswahl löschen beim Starten
                self.state.selected_link_indices.clear();
                self.state.click_interaction_state.box_selection = egui::Rect::from_min_max(
                    self.state.interaction_state.mouse_pos, // Start an der Mausposition
                    self.state.interaction_state.mouse_pos,
                );
            }
        }
    }

    fn begin_link_interaction(&mut self, link_id: usize) {
        // Prüfen, ob ein Pin gehovert wird UND Detach-Flag gesetzt ist
        let pin_is_hovered_and_detachable = self
            .frame_state
            .hovered_pin_index
            .map(|pin_id| {
                // Prüft ob der Pin zum Link gehört und detachbar ist
                (self.frame_state.hovered_pin_flags
                    & AttributeFlags::EnableLinkDetachWithDragClick as usize
                    != 0)
                    && self.links.get(&link_id).map_or(false, |l| {
                        l.spec.start_pin_index == pin_id || l.spec.end_pin_index == pin_id
                    })
            })
            .unwrap_or(false);

        // Prüfen, ob Detach-Modifier aktiv ist
        let detach_with_modifier = self
            .state
            .interaction_state
            .link_detatch_with_modifier_click;

        // Fall 1: Detach durch Klick auf Pin mit Flag
        if pin_is_hovered_and_detachable {
            if let Some(pin_id) = self.frame_state.hovered_pin_index {
                self.begin_link_detach(link_id, pin_id);
                return;
            }
        }

        // Fall 2: Detach durch Klick + Modifier auf Link
        if detach_with_modifier && !pin_is_hovered_and_detachable {
            // Verhindert Detach, wenn schon durch Pin getriggert
            if let Some(link) = self.links.get(&link_id) {
                // Finde den näheren Pin zur Mausposition für den Detach-Start
                if let (Some(start_pin), Some(end_pin)) = (
                    self.pins.get(&link.spec.start_pin_index),
                    self.pins.get(&link.spec.end_pin_index),
                ) {
                    let pos_start = self.get_screen_space_pin_coordinates(start_pin);
                    let pos_end = self.get_screen_space_pin_coordinates(end_pin);
                    let dist_start_sq =
                        pos_start.distance_sq(self.state.interaction_state.mouse_pos);
                    let dist_end_sq = pos_end.distance_sq(self.state.interaction_state.mouse_pos);

                    let closest_pin_idx = if dist_start_sq < dist_end_sq {
                        link.spec.start_pin_index
                    } else {
                        link.spec.end_pin_index
                    };
                    self.begin_link_detach(link_id, closest_pin_idx);
                    return;
                }
            }
        }

        // Fall 3: Standard Link Selection
        self.begin_link_selection(link_id);
    }

    fn begin_link_creation(&mut self, pin_id: usize) {
        // Nur starten, wenn noch keine Interaktion läuft und der Pin existiert
        if self.state.click_interaction_type == ClickInteractionType::None {
            if let Some(pin) = self.pins.get(&pin_id) {
                // Prüfen, ob Link-Erstellung von diesem Pin-Typ erlaubt ist (typischerweise Output)
                // Oder ob spezielle Flags gesetzt sind. Hier vereinfacht: Outputs können starten.
                if pin.spec.kind == PinType::Output
                    || (pin.spec.flags & AttributeFlags::EnableLinkCreationOnSnap as usize != 0)
                {
                    // Beispiel für Flag-Nutzung
                    self.state.click_interaction_type = ClickInteractionType::LinkCreation;
                    self.state.click_interaction_state.link_creation =
                        ClickInteractionStateLinkCreation {
                            start_pin_idx: Some(pin_id),
                            end_pin_index: None, // Kein End-Pin beim Start
                            link_creation_type: LinkCreationType::Standard,
                        };
                    self.frame_state.element_state_change.link_started = true;
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

    fn begin_link_selection(&mut self, link_id: usize) {
        if self.state.click_interaction_type == ClickInteractionType::None
            || (self.state.click_interaction_type == ClickInteractionType::Link
                && !self.state.selected_link_indices.contains(&link_id))
        {
            self.state.click_interaction_type = ClickInteractionType::Link;
            self.state.selected_node_indices.clear(); // Nur Links selektieren
            self.state.selected_link_indices.clear();
            self.state.selected_link_indices.push(link_id);
        } else if self.state.click_interaction_type == ClickInteractionType::Link {
            // Optional: Bei Klick auf bereits selektierten Link -> Deselection?
            // self.state.selected_link_indices.clear();
            // self.state.click_interaction_type = ClickInteractionType::None;
        }
    }

    fn begin_node_selection(&mut self, node_id: usize) {
        if self.state.click_interaction_type != ClickInteractionType::None {
            return;
        } // Nur wenn keine andere Interaktion läuft

        self.state.click_interaction_type = ClickInteractionType::Node;

        // Check for multi-selection (Shift-Click, Ctrl-Click, etc.) - Nicht implementiert hier
        // Standardverhalten: Clear und Select
        if !self.state.selected_node_indices.contains(&node_id) {
            self.state.selected_node_indices.clear();
            self.state.selected_link_indices.clear(); // Auch Links deselektieren
            self.state.selected_node_indices.push(node_id);
            self.frame_state.just_selected_node = true; // Markiere für Detail View Update etc.
        } else {
            // Node war bereits selektiert. Optional: Bei erneutem Klick nicht wieder auswählen oder Modifikatoren prüfen.
            self.frame_state.just_selected_node = false; // Nicht *gerade eben* selektiert
        }

        // Node in der Tiefenordnung nach oben bringen
        if let Some(pos) = self
            .state
            .node_depth_order
            .iter()
            .position(|x| *x == node_id)
        {
            // Wenn gefunden, entfernen und ans Ende (oben) schieben
            let id_to_move = self.state.node_depth_order.remove(pos);
            self.state.node_depth_order.push(id_to_move);
        } else {
            // Sollte nicht passieren, wenn node_id aus einer gültigen Quelle stammt
            bevy::log::warn!(
                "Node {:?} nicht in Tiefenordnung gefunden beim Selektieren.",
                node_id
            );
            self.state.node_depth_order.push(node_id); // Vorsichtshalber hinzufügen
        }
    }

    // Startet Link-Erstellung vom *anderen* Pin, nachdem der Original-Link entfernt wurde
    fn begin_link_detach(&mut self, link_id: usize, detach_pin_idx: usize) {
        if self.state.click_interaction_type == ClickInteractionType::None {
            if let Some(removed_link) = self.links.remove(&link_id) {
                self.frame_state.deleted_link_idx = Some(link_id);
                self.frame_state
                    .graph_changes
                    .push(GraphChange::LinkRemoved(link_id));

                let other_pin_id = if removed_link.spec.start_pin_index == detach_pin_idx {
                    removed_link.spec.end_pin_index
                } else {
                    removed_link.spec.start_pin_index
                };

                // === KORREKTUR unused variable ===
                // Prüfen ob der andere Pin existiert, ohne ihn zu binden
                if self.pins.contains_key(&other_pin_id) {
                    self.state.click_interaction_type = ClickInteractionType::LinkCreation;
                    self.state.click_interaction_state.link_creation =
                        ClickInteractionStateLinkCreation {
                            start_pin_idx: Some(other_pin_id), // Store as Option
                            end_pin_index: None,
                            link_creation_type: LinkCreationType::FromDetach,
                        };
                    self.frame_state.element_state_change.link_started = true;
                } else {
                    log::warn!(
                        "Link detach: Anderer Pin ({:?}) nicht gefunden.",
                        other_pin_id
                    );
                }
            } else {
                log::warn!("Link detach: Link {:?} nicht gefunden.", link_id);
            }
        }
    }

    // --- Update/Logic Methods ---
    fn translate_selected_nodes(&mut self) {
        // Nur ausführen, wenn gezogen wird UND eine Node-Interaktion läuft
        if self.state.interaction_state.left_mouse_dragging
            && self.state.click_interaction_type == ClickInteractionType::Node
        {
            // Delta in Screen Space
            let delta_screen = self.state.interaction_state.mouse_delta;
            // Delta in Grid Space (einfache Subtraktion der Canvas-Origin reicht nicht, Panning muss raus!)
            // Da Panning *während* des Zugs konstant bleibt, ist Screen Delta = Grid Delta
            let delta_grid = delta_screen;

            if delta_grid.length_sq() > 0.0 {
                // Nur wenn es eine Bewegung gab
                let mut changes = Vec::new(); // Sammle Änderungen für Events

                // Gehe alle selektierten Node IDs durch
                for node_id in &self.state.selected_node_indices {
                    // Mutable Borrow auf den Node, wenn er existiert
                    if let Some(node) = self.nodes.get_mut(node_id) {
                        // Prüfe, ob Node beweglich ist (optionales Flag)
                        if node.state.draggable {
                            // Update die Node *Spezifikation* (Ursprung im Grid Space)
                            node.spec.origin += delta_grid;
                            // Update den *Zustand* (Rect im Grid Space)
                            node.state.rect = node.state.rect.translate(delta_grid);

                            // Sammle die Änderung für das Event
                            changes.push((
                                *node_id,
                                BevyVec2::new(node.spec.origin.x, node.spec.origin.y), // Aktuelle Grid-Position
                            ));
                        }
                    }
                }

                // Sende alle gesammelten Events
                for (id, pos) in changes {
                    self.frame_state
                        .graph_changes
                        .push(GraphChange::NodeMoved(id, pos));
                }
            }
        }
    }

    fn should_link_snap_to_pin(
        &self,
        start_pin_id: usize,   // Der Pin, von dem die Verbindung gestartet wurde
        hovered_pin_id: usize, // Der Pin, über dem die Maus gerade schwebt
        duplicate_link: Option<usize>, // ID eines existierenden Links zwischen diesen Pins (falls vorhanden)
    ) -> bool {
        // 1. Prüfe, ob beide Pins existieren
        let Some(start_pin) = self.pins.get(&start_pin_id) else {
            return false;
        };
        let Some(end_pin) = self.pins.get(&hovered_pin_id) else {
            return false;
        };

        // 2. Verbindung zum selben Node verhindern
        if start_pin.state.parent_node_idx == end_pin.state.parent_node_idx {
            return false;
        }

        // 3. Verbindung zwischen Pins des gleichen Typs verhindern (Input <-> Input nicht erlaubt)
        if start_pin.spec.kind == end_pin.spec.kind {
            return false;
        }

        // 4. Doppelte Links verhindern (wenn `duplicate_link` Some ist)
        if duplicate_link.is_some() {
            return false;
        }

        // 5. Prüfen, ob der Ziel-Pin überhaupt Verbindungen erlaubt
        //    (Beispiel: Ein Input-Pin könnte ein Flag haben `AcceptsLink`)
        if (end_pin.spec.flags & AttributeFlags::EnableLinkCreationOnSnap as usize) == 0
            && (start_pin.spec.flags & AttributeFlags::EnableLinkCreationOnSnap as usize) == 0
        //&& end_pin.spec.kind == PinType::Input // Oder nur für Inputs prüfen?
        {
            // Optional: Wenn keiner der Pins das Snapping explizit erlaubt (gemäß Flag)
            // return false;
            // Hängt von der gewünschten Logik ab. Oft reicht es, dass der *startende* (Output) Pin es erlaubt.
        }

        // Wenn alle Prüfungen bestanden, ist Snapping erlaubt
        true
    }

    fn box_selector_update_selection(&mut self) {
        // Nicht mehr &self, braucht &mut self
        // Nur ausführen, wenn die Box-Selektion aktiv ist
        if self.state.click_interaction_type != ClickInteractionType::BoxSelection {
            return;
        }

        // Update die Endposition der Box zur aktuellen Mausposition
        self.state.click_interaction_state.box_selection.max =
            self.state.interaction_state.mouse_pos;
        let box_rect_screen = self.state.click_interaction_state.box_selection;

        // Normalisiere das Rechteck (min sollte immer links oben sein)
        let normalized_box_screen = egui::Rect::from_min_max(
            box_rect_screen.min.min(box_rect_screen.max),
            box_rect_screen.min.max(box_rect_screen.max),
        );

        // Wandle das Screen-Rect der Box in Grid-Space um für den Node-Vergleich
        let box_rect_grid = egui::Rect::from_min_max(
            self.screen_space_to_grid_space(normalized_box_screen.min),
            self.screen_space_to_grid_space(normalized_box_screen.max),
        );

        let previous_selected_nodes = self.state.selected_node_indices.clone(); // Merke vorherige Auswahl für Reihenfolge
        self.state.selected_node_indices.clear(); // Leere aktuelle Auswahl
        self.state.selected_link_indices.clear(); // Auch Links leeren

        // --- Node Selection ---
        for (id, node) in self.nodes.iter() {
            // Prüfe Überschneidung im Grid Space
            if box_rect_grid.intersects(node.state.rect) {
                self.state.selected_node_indices.push(*id);
            }
        }
        // Optional: Behalte ursprüngliche Selektionsreihenfolge bei, falls Nodes wieder selektiert werden
        self.state.selected_node_indices.sort_by_key(|id| {
            previous_selected_nodes
                .iter()
                .position(|&old_id| old_id == *id)
                .unwrap_or(usize::MAX)
        });

        // --- Link Selection ---
        for (id, link) in self.links.iter() {
            // Prüfe, ob *beide* Pins des Links existieren
            if let (Some(start_pin), Some(end_pin)) = (
                self.pins.get(&link.spec.start_pin_index),
                self.pins.get(&link.spec.end_pin_index),
            ) {
                // Prüfe, ob *beide* Nodes der Pins existieren
                if self.nodes.contains_key(&start_pin.state.parent_node_idx)
                    && self.nodes.contains_key(&end_pin.state.parent_node_idx)
                {
                    // Berechne Pin-Positionen im Screen Space
                    let p1_screen = self.get_screen_space_pin_coordinates(start_pin);
                    let p2_screen = self.get_screen_space_pin_coordinates(end_pin);

                    // Prüfe Überschneidung des Links mit der *Screen*-Box
                    if self.rectangle_overlaps_link(
                        &normalized_box_screen,
                        &p1_screen,
                        &p2_screen,
                        start_pin.spec.kind,
                    ) {
                        self.state.selected_link_indices.push(*id);
                    }
                }
            }
        }
        // box_rect_screen // Wird nicht mehr zurückgegeben
    }

    fn rectangle_overlaps_link(
        &self,
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
            self.settings.style.link_line_segments_per_length,
        );

        // Verwende die Hilfsfunktion der Bezier-Daten zur Überlappungsprüfung
        link_data.rectangle_overlaps_bezier(rect)
    }

    fn find_duplicate_link(&self, start_pin_id: usize, end_pin_id: usize) -> Option<usize> {
        // Normalisiere die Pin-IDs für den Vergleich (Reihenfolge egal)
        let (p1, p2) = if start_pin_id < end_pin_id {
            (start_pin_id, end_pin_id)
        } else {
            (end_pin_id, start_pin_id)
        };

        // Suche nach einem existierenden Link mit denselben (normalisierten) Pin-IDs
        for (id, link) in self.links.iter() {
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

    // === MODIFIED: TODO 4 - Permanente Link-Erstellung integriert ===
    fn click_interaction_update(&mut self, _ui: &mut egui::Ui) {
        match self.state.click_interaction_type {
            ClickInteractionType::BoxSelection => {
                self.box_selector_update_selection(); // Aktualisiert die Auswahl basierend auf der Box
                if self.state.interaction_state.left_mouse_released {
                    // Box-Selektion beendet
                    self.state.click_interaction_type = ClickInteractionType::None;
                    // Bringt neu ausgewählte Nodes in der Tiefenordnung nach oben (optional)
                    let s = self.state.selected_node_indices.clone();
                    self.state.node_depth_order.retain(|id| !s.contains(id));
                    self.state.node_depth_order.extend(s);
                }
            }
            ClickInteractionType::Node => {
                self.translate_selected_nodes(); // Bewegt ausgewählte Nodes
                if self.state.interaction_state.left_mouse_released {
                    // Node-Drag beendet
                    self.state.click_interaction_type = ClickInteractionType::None;
                }
            }
            ClickInteractionType::Link => {
                // Keine Aktion während des Haltens/Ziehens eines Links (nur bei Release interessant)
                if self.state.interaction_state.left_mouse_released {
                    // Link-Selektion "beendet" (keine spezifische Aktion hier nötig)
                    self.state.click_interaction_type = ClickInteractionType::None;
                }
            }
            ClickInteractionType::LinkCreation => {
                // Aktuellen Start-Pin holen (sollte existieren)
                let start_pin_id = self
                    .state
                    .click_interaction_state
                    .link_creation
                    .start_pin_idx
                    .unwrap_or(usize::MAX); // Fehlerbehandlung?
                if self.pins.get(&start_pin_id).is_none() {
                    bevy::log::error!("Link Creation: Start pin {} missing!", start_pin_id);
                    self.state.click_interaction_type = ClickInteractionType::None;
                    return;
                };

                // Prüfen, ob über einem gültigen Pin gesnapped wird
                let mut snapped_pin_id: Option<usize> = None; // Reset
                if let Some(hovered_pin_id) = self.frame_state.hovered_pin_index {
                    // Prüfe, ob ein Link zwischen diesen Pins bereits existiert
                    let duplicate_link_id = self.find_duplicate_link(start_pin_id, hovered_pin_id);

                    // Prüfe, ob die Verbindung gültig ist (Pins existieren, nicht derselbe Node, Typen passen, kein Duplikat etc.)
                    if self.should_link_snap_to_pin(start_pin_id, hovered_pin_id, duplicate_link_id)
                    {
                        // Wenn gültig -> Markiere als gesnapped
                        snapped_pin_id = Some(hovered_pin_id);
                    }
                }

                // Update den Endpunkt für die temporäre Visualisierung
                self.state
                    .click_interaction_state
                    .link_creation
                    .end_pin_index = snapped_pin_id;

                // --- Logik beim Loslassen der Maustaste ---
                if self.state.interaction_state.left_mouse_released {
                    if let Some(end_pin_id) = snapped_pin_id {
                        // *** Erfolgreich verbunden! ***

                        // 1. Erzeuge Event/Änderung für die Außenwelt (System, das Links tatsächlich erstellt)
                        //   Das Event sollte die Pin-IDs enthalten.
                        self.frame_state
                            .graph_changes
                            .push(GraphChange::NewLinkRequested(start_pin_id, end_pin_id));
                        bevy::log::info!(
                            "GraphChange::NewLinkRequested sent for pins: {} -> {}",
                            start_pin_id,
                            end_pin_id
                        );

                        // 2. Optional: Füge den Link direkt zum *internen* Zustand hinzu, damit er sofort angezeigt wird.
                        //    Braucht eine neue interne ID. Diese ist NICHT die persistente ID!
                        let internal_new_link_id = self.state.next_link_id;
                        self.state.next_link_id += 1; // Inkrementiere Zähler

                        // Bestimme korrekte Start/End Pin basierend auf Typ
                        let (output_pin_id, input_pin_id) = {
                            let start_pin_kind = self
                                .pins
                                .get(&start_pin_id)
                                .map_or(PinType::None, |p| p.spec.kind);
                            if start_pin_kind == PinType::Output {
                                (start_pin_id, end_pin_id)
                            } else {
                                (end_pin_id, start_pin_id) // Drehe um, wenn von Input gestartet wurde (z.B. nach Detach)
                            }
                        };

                        let new_link_spec = LinkSpec {
                            id: internal_new_link_id,        // Temporäre interne ID
                            start_pin_index: output_pin_id,  // Immer Output Pin als Start speichern
                            end_pin_index: input_pin_id,     // Immer Input Pin als Ende speichern
                            style: LinkStyleArgs::default(), // Standard-Style
                        };

                        let new_link_state = LinkState {
                            style: self.settings.style.format_link(new_link_spec.style.clone()),
                            // Shape Index muss neu geholt werden, da add_link nicht direkt hier aufgerufen wird
                            // Shape wird dann in final_draw gezeichnet
                            shape: None, // Wird in final_draw oder beim nächsten show() gesetzt? -> Besser: direkt hier versuchen
                        };

                        // Füge zur internen Map hinzu
                        self.links.insert(
                            internal_new_link_id,
                            Link {
                                spec: new_link_spec,
                                state: new_link_state, // Style ist hier formatiert
                            },
                        );

                        // Markiere, dass ein Link *visuell* erstellt wurde (für interne Logik)
                        self.frame_state.element_state_change.link_created = true;
                    } else {
                        // Nicht über gültigem Pin losgelassen
                        self.frame_state.element_state_change.link_dropped = true;
                    }
                    // Link-Erstellung beenden (egal ob erfolgreich oder nicht)
                    self.state.click_interaction_type = ClickInteractionType::None;
                }
            }
            ClickInteractionType::Panning => {
                // Nur wenn die Alt-Taste gehalten und gezogen wird
                if self.state.interaction_state.alt_mouse_dragging {
                    self.state.panning += self.state.interaction_state.mouse_delta;
                // Update Panning
                }
                // Panning beenden, wenn Alt-Taste losgelassen wird (oder nicht mehr gezogen)
                // Prüfung auf !dragging UND !clicked könnte nötig sein, je nach egui Event Timing
                else if !self.state.interaction_state.alt_mouse_dragging
                    && !self.state.interaction_state.alt_mouse_clicked
                {
                    self.state.click_interaction_type = ClickInteractionType::None;
                }
            }
            ClickInteractionType::None => {
                // Keine aktive Interaktion
            }
        }
    }

    fn handle_delete(&mut self) {
        let mut links_to_remove: Vec<usize> = self.state.selected_link_indices.drain(..).collect();
        let nodes_to_remove: Vec<usize> = self.state.selected_node_indices.drain(..).collect();
        let mut pins_to_remove = Vec::new();

        // 1. Finde alle Pins der zu löschenden Nodes
        for node_id in &nodes_to_remove {
            if let Some(node) = self.nodes.get(node_id) {
                pins_to_remove.extend(node.state.pin_indices.iter().copied());
            }
        }

        // 2. Finde alle Links, die mit den zu löschenden Pins verbunden sind
        for (link_id, link) in self.links.iter() {
            if pins_to_remove.contains(&link.spec.start_pin_index)
                || pins_to_remove.contains(&link.spec.end_pin_index)
            {
                if !links_to_remove.contains(link_id) {
                    links_to_remove.push(*link_id); // Füge implizit zu löschende Links hinzu
                }
            }
        }
        links_to_remove.sort_unstable();
        links_to_remove.dedup(); // Sicherstellen, dass IDs unique sind

        // 3. Entferne die Links aus dem internen State und sammle Events
        for link_id in &links_to_remove {
            if self.links.remove(link_id).is_some() {
                self.frame_state
                    .graph_changes
                    .push(GraphChange::LinkRemoved(*link_id));
            }
        }

        // 4. Entferne die Nodes aus dem internen State und sammle Events
        for node_id in nodes_to_remove {
            if self.nodes.remove(&node_id).is_some() {
                self.frame_state
                    .graph_changes
                    .push(GraphChange::NodeRemoved(node_id));
                // Entferne Node auch aus der Tiefenordnung
                self.state.node_depth_order.retain(|id| *id != node_id);
            }
        }

        // 5. Entferne die Pins aus dem internen State
        for pin_id in pins_to_remove {
            self.pins.remove(&pin_id);
        }

        // 6. Selektionen leeren (sollte schon durch .drain() passiert sein, aber zur Sicherheit)
        self.state.selected_link_indices.clear();
        self.state.selected_node_indices.clear();
    }

    fn final_draw(&mut self, ui_draw: &mut egui::Ui) {
        // === 1. Links zeichnen (unter den Nodes) ===
        // Hole Link IDs bevor iteriert wird, da draw_link &mut self nimmt
        let link_ids: Vec<usize> = self.links.keys().copied().collect();
        for link_id in link_ids {
            self.draw_link(link_id, ui_draw);
        }

        // === 2. Nodes zeichnen (gemäß Tiefenordnung) ===
        // Hole Node IDs aus der Tiefenordnung, da draw_node &mut self nimmt
        let node_order = self.state.node_depth_order.clone();
        for node_id in node_order.iter() {
            // `draw_node` kümmert sich intern um das Zeichnen der Pins dieses Nodes
            self.draw_node(*node_id, ui_draw);
        }

        // === 3. Temporäre Elemente zeichnen (über allem anderen) ===
        self.draw_temporary_elements(ui_draw); // Nimmt &self
    }

    // === MODIFIED: TODO 4 - Event Sammlung angepasst ===
    fn collect_events(&mut self) {
        // `GraphChange` Events für NodeMoved, LinkRemoved, NodeRemoved
        // werden bereits in `translate_selected_nodes` und `handle_delete` hinzugefügt.

        // Hier geht es darum, interne `ElementStateChange`-Flags in `GraphChange`-Events
        // umzuwandeln, falls nötig.
        // `LinkCreated` wird jetzt in `click_interaction_update` direkt als `NewLinkRequested` gesendet.

        // Beispiel: Wenn man noch spezifische Events für "dropped" etc. bräuchte:
        /*
         if self.frame_state.element_state_change.link_dropped {
             // Eventuell ein GraphChange::LinkDropped(start_pin_id) senden?
        }
        if self.frame_state.deleted_link_idx.is_some() {
             // Wurde bereits in handle_delete/begin_link_detach als LinkRemoved hinzugefügt
        }
        */
        // Reset der Frame-State-Änderungsflags (am Ende von `reset`?)
        // self.frame_state.element_state_change.reset(); // Schon in reset()
    }

    // --- Getter Methods ---
    #[allow(dead_code)]
    pub fn node_hovered(&self) -> Option<usize> {
        self.frame_state.hovered_node_index
    }
    #[allow(dead_code)]
    pub fn link_hovered(&self) -> Option<usize> {
        self.frame_state.hovered_link_idx
    }
    #[allow(dead_code)]
    pub fn pin_hovered(&self) -> Option<usize> {
        self.frame_state.hovered_pin_index
    }
    #[allow(dead_code)]
    pub fn num_selected_nodes(&self) -> usize {
        self.state.selected_node_indices.len()
    }

    pub fn get_selected_nodes(&self) -> Vec<usize> {
        self.state.selected_node_indices.clone()
    }

    #[allow(dead_code)]
    pub fn get_selected_links(&self) -> Vec<usize> {
        self.state.selected_link_indices.clone()
    }
    #[allow(dead_code)]
    pub fn clear_node_selection(&mut self) {
        self.state.selected_node_indices.clear();
    }
    #[allow(dead_code)]
    pub fn clear_link_selection(&mut self) {
        self.state.selected_link_indices.clear();
    }
    #[allow(dead_code)]
    pub fn active_attribute(&self) -> Option<usize> {
        self.frame_state.active_pin
    } // Eher selten genutzt, da Hovered Pin wichtiger ist
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn link_destroyed(&self) -> Option<usize> {
        self.frame_state.deleted_link_idx
    }

    pub fn get_changes(&self) -> &Vec<GraphChange> {
        &self.frame_state.graph_changes
    }

    #[allow(dead_code)]
    pub fn get_panning(&self) -> egui::Vec2 {
        self.state.panning
    }
    #[allow(dead_code)]
    pub fn reset_panning(&mut self, panning: egui::Vec2) {
        self.state.panning = panning;
    }
    #[allow(dead_code)]
    pub fn get_node_dimensions(&self, id: usize) -> Option<egui::Vec2> {
        self.nodes.get(&id).map(|n| n.state.rect.size())
    }

    pub fn is_node_just_selected(&self) -> bool {
        self.frame_state.just_selected_node
    }

    #[allow(dead_code)]
    pub fn set_node_draggable(&mut self, node_id: usize, draggable: bool) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.state.draggable = draggable;
        }
    }

    fn process_clicks(&mut self) {
        if !self.state.interaction_state.mouse_in_canvas {
            return;
        }

        // --- Linksklick ---
        if self.state.interaction_state.left_mouse_clicked {
            if let Some(pin_idx) = self.frame_state.hovered_pin_index {
                // Klick auf Pin -> Versuche, Link-Erstellung zu starten
                self.begin_link_creation(pin_idx);
            } else if let Some(node_idx) = self.frame_state.hovered_node_index {
                // Klick auf Node -> Starte Node-Selektion/-Drag
                // Verhindere Start, wenn gerade ein Pin im *selben* Node geklickt wurde (falls `active_pin` genutzt wird)
                // if self.frame_state.active_pin.map_or(true, |p_id| self.pins.get(&p_id).map_or(true, |p| p.state.parent_node_idx != node_idx)) {
                self.begin_node_selection(node_idx);
                // }
            } else if let Some(link_idx) = self.frame_state.hovered_link_idx {
                // Klick auf Link -> Starte Link-Interaktion (Select/Detach)
                self.begin_link_interaction(link_idx);
            } else {
                // Klick ins Leere -> Starte Box-Selektion
                self.begin_canvas_interaction(false);
            }
        }
        // --- Alternativklick (z.B. Mitte, Rechts) ---
        else if self.state.interaction_state.alt_mouse_clicked {
            // Klick ins Leere mit Alt -> Starte Panning
            if self.frame_state.hovered_node_index.is_none()
                && self.frame_state.hovered_pin_index.is_none()
                && self.frame_state.hovered_link_idx.is_none()
            {
                self.begin_canvas_interaction(true);
            }
            // Optional: Alt-Klick auf Node/Pin/Link für Kontextmenü etc.
        }
    }
} // === Ende impl NodesContext ===

// === Hilfsstrukturen/Enums am Ende ===
#[derive(Derivative, Default, Debug, Clone)] // Clone hinzugefügt
struct ElementStateChange {
    link_started: bool,
    link_dropped: bool,
    link_created: bool, // Erfolgreich an Pin gesnapped und Maus losgelassen
}
impl ElementStateChange {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum ClickInteractionType {
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
enum LinkCreationType {
    #[default]
    Standard,
    FromDetach,
} // Default zu Standard

#[derive(Derivative, Debug, Clone)] // Clone hinzugefügt
#[derivative(Default)]
struct ClickInteractionStateLinkCreation {
    start_pin_idx: Option<usize>, // Option, falls Detach fehlschlägt
    end_pin_index: Option<usize>,
    #[derivative(Default(value = "LinkCreationType::default()"))]
    link_creation_type: LinkCreationType,
}
#[derive(Derivative, Debug, Clone)] // Only Clone hinzugefügt
#[derivative(Default)]
struct ClickInteractionState {
    link_creation: ClickInteractionStateLinkCreation,
    #[derivative(Default(value = "egui::Rect::ZERO"))]
    box_selection: egui::Rect,
}

#[derive(Derivative, Debug, Default)]
pub struct IO {
    /* ... unverändert ... */
    #[derivative(Default(value = "Modifier::None"))]
    pub emulate_three_button_mouse: Modifier,
    #[derivative(Default(value = "Modifier::None"))]
    pub link_detatch_with_modifier_click: Modifier,
    #[derivative(Default(value = "Some(egui::PointerButton::Middle)"))]
    pub alt_mouse_button: Option<egui::PointerButton>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Modifier {
    /* ... unverändert ... */
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
    fn get_node_title_rect_screen(&self, context: &NodesContext) -> egui::Rect {
        let grid_rect = self.get_node_title_rect_grid();
        egui::Rect::from_min_max(
            // MinMax verwenden, um sicherzugehen
            context.grid_space_to_screen_space(grid_rect.min),
            context.grid_space_to_screen_space(grid_rect.max),
        )
    }
}

// Helper für Bezier-Distanz im Quadrat (vermeidet sqrt)
impl LinkBezierData {
    pub(crate) fn get_distance_to_cubic_bezier_sq(&self, pos: &egui::Pos2) -> f32 {
        let point_on_curve = self.get_closest_point_on_cubic_bezier(pos);
        pos.distance_sq(point_on_curve)
    }
}
