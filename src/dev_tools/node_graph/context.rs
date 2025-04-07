// src/dev_tools/node_graph/context.rs
use bevy::color::Srgba; // Direkter Import für Srgba Konvertierung
use bevy::math::Vec2 as BevyVec2; // Für Umwandlung
use bevy::prelude::{Entity, Resource}; // Bevy Typen
use bevy_egui::egui::{self, Color32, CornerRadius, StrokeKind}; // Egui Typen, inkl. CornerRadius
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

#[derive(Debug, Clone)]
pub enum GraphChange {
    LinkCreated(usize, usize),
    LinkRemoved(usize),
    NodeMoved(usize, BevyVec2), // Grid Space Position
    NodeRemoved(usize),
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
    graph_changes: Vec<GraphChange>,
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
    nodes: HashMap<usize, Node>,
    pins: HashMap<usize, Pin>,
    links: HashMap<usize, Link>,
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

        // 2. Convert Input Data
        let mut node_specs = HashMap::new();
        let mut link_specs = Vec::new();
        for vis_node in nodes_data {
            let pins_for_this_node: Vec<PinSpec> = Vec::new(); // TODO: Pins hier erstellen!
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
                vis_node.id,
                NodeSpec {
                    id: vis_node.id,
                    name: vis_node.name,
                    origin: egui::pos2(vis_node.position.x, vis_node.position.y),
                    attributes: pins_for_this_node,
                    args: node_args,
                    subtitle: format!("E:{:?}", vis_node.entity),
                    time: None,
                    duration: None,
                    active: self.state.selected_node_indices.contains(&vis_node.id),
                },
            );
        }
        for vis_link in links_data {
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
        }

        // 3. Create Child UI
        let mut child_ui = ui.child_ui(
            self.frame_state.canvas_rect_screen_space,
            egui::Layout::top_down(egui::Align::Min).with_cross_justify(true), // Layout als zweites Argument
            None, // Füge None als drittes Argument hinzu
        );

        // 4. Draw Canvas Background & Grid
        {
            let ui_bg = &mut child_ui;
            let screen_rect = ui_bg.ctx().screen_rect();
            ui_bg.set_clip_rect(
                self.frame_state
                    .canvas_rect_screen_space
                    .intersect(screen_rect),
            );
            ui_bg.painter().rect_filled(
                self.frame_state.canvas_rect_screen_space,
                CornerRadius::ZERO,
                self.settings.style.colors[ColorStyle::GridBackground as usize],
            );
            if (self.settings.style.flags & StyleFlags::GridLines as usize) != 0 {
                self.draw_grid(self.frame_state.canvas_rect_screen_space.size(), ui_bg);
            }
        }

        // 5. Populate Internal State (add_node füllt frame_state.nodes/pins_tmp, add_link füllt self.links)
        {
            let mut node_ids: Vec<usize> = node_specs.keys().copied().collect();
            self.state
                .node_depth_order
                .retain(|id| node_ids.contains(id));
            node_ids.retain(|id| !self.state.node_depth_order.contains(id));
            node_ids.sort_unstable();
            self.state.node_depth_order.extend(node_ids);
            for link_spec in link_specs.iter() {
                self.add_link(link_spec.clone(), &mut child_ui);
            }
            for node_id in self.state.node_depth_order.clone().iter() {
                if let Some(node_spec) = node_specs.get(node_id) {
                    self.add_node(node_spec.clone(), &mut child_ui);
                }
            }
            self.nodes = std::mem::take(&mut self.frame_state.nodes_tmp);
            self.pins = std::mem::take(&mut self.frame_state.pins_tmp);
        }

        // 6. Interaction Processing
        let interact_response = child_ui.interact(
            self.frame_state.canvas_rect_screen_space,
            child_ui.id().with("NodeCanvasInteractor"),
            egui::Sense::click_and_drag(),
        );
        let hover_pos = interact_response.hover_pos();
        child_ui.ctx().input(|io| {
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
        self.click_interaction_update(&mut child_ui);
        if self.state.interaction_state.delete_pressed {
            self.handle_delete();
        }
        self.final_draw(&mut child_ui);
        self.collect_events();

        // 7. Draw Canvas Outline
        child_ui.painter().rect_stroke(
            self.frame_state.canvas_rect_screen_space,
            CornerRadius::ZERO,
            egui::Stroke::new(1.0, Color32::WHITE),
            StrokeKind::Inside,
        );

        // 8. Return Response
        interact_response
    }

    // --- Add Node/Pin/Link Methods ---
    fn add_node(&mut self, node_spec: NodeSpec, ui: &mut egui::Ui) {
        /* Implementierung wie oben */
        let node_id = node_spec.id;
        let node_state = self
            .nodes
            .get(&node_id)
            .map_or_else(NodeState::default, |n| {
                let mut state = n.state.clone();
                state.pin_indices.clear();
                state
            });
        let mut node = Node {
            spec: node_spec,
            state: node_state,
        };
        let (color_style, layout_style) = self.settings.style.format_node(node.spec.args.clone());
        node.state.color_style = color_style;
        node.state.layout_style = layout_style;
        let painter = ui.painter();
        node.state.background_shape = Some(painter.add(egui::Shape::Noop));
        node.state.titlebar_shape = Some(painter.add(egui::Shape::Noop));
        node.state.outline_shape = Some(painter.add(egui::Shape::Noop));
        node.state.pin_indices.clear(); // Wichtig: Immer leeren vor add_pin
        let node_origin_grid = node.spec.origin;
        let response = ui.allocate_ui_with_layout(
            node.state.size,
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                let mut title_rect_calc = egui::Rect::NOTHING;
                let title_response = ui
                    .vertical(|ui| {
                        ui.label(node.spec.name.clone());
                        ui.separator();
                        ui.label(node.spec.subtitle.clone());
                    })
                    .response;
                title_rect_calc = title_response.rect;
                ui.add_space(node.state.layout_style.padding.y);
                egui::Grid::new(format!("node_pins_{}", node.spec.id))
                    .num_columns(2)
                    .min_col_width(50.0)
                    .show(ui, |ui| {
                        // Collect input pins first to avoid borrowing node both immutably and mutably
                        let input_pins: Vec<_> = node
                            .spec
                            .attributes
                            .iter()
                            .filter(|p| p.kind == PinType::Input)
                            .cloned()
                            .collect();

                        // Collect output pins first to avoid borrowing node both immutably and mutably
                        let output_pins: Vec<_> = node
                            .spec
                            .attributes
                            .iter()
                            .filter(|p| p.kind == PinType::Output)
                            .cloned()
                            .collect();

                        ui.vertical(|ui| {
                            for pin_spec in input_pins {
                                self.add_pin(pin_spec, &mut node, ui);
                            }
                        });
                        ui.vertical(|ui| {
                            for pin_spec in output_pins {
                                self.add_pin(pin_spec, &mut node, ui);
                            }
                        });
                        ui.end_row();
                    });
                egui::Frame::default().show(ui, |_ui| { /* Runtime data */ });
                (ui.min_rect().size(), title_rect_calc)
            },
        );
        let (inner_size, title_rect) = response.inner;
        node.state.size = inner_size + node.state.layout_style.padding * 2.0;
        node.state.rect = egui::Rect::from_min_size(node_origin_grid, node.state.size);
        node.state.title_bar_content_rect = title_rect;
        let screen_rect = egui::Rect::from_min_size(
            self.grid_space_to_screen_space(node.state.rect.min),
            node.state.rect.size(),
        );
        if ui.rect_contains_pointer(screen_rect) {
            self.frame_state
                .node_indices_overlapping_with_mouse
                .push(node_id);
        }
        self.frame_state.nodes_tmp.insert(node_id, node);
    }

    fn add_pin(&mut self, pin_spec: PinSpec, node: &mut Node, ui: &mut egui::Ui) {
        /* Implementierung wie oben */
        let response = ui.allocate_ui(egui::vec2(ui.available_width(), 10.0), |ui| {
            let name = pin_spec.name.clone();
            let align = if pin_spec.kind == PinType::Input {
                egui::Align::LEFT
            } else {
                egui::Align::RIGHT
            };
            ui.with_layout(egui::Layout::top_down(align), |ui| ui.label(name))
        });
        let label_rect_relative_to_node_origin = response
            .inner
            .response
            .rect
            .translate(-node.state.rect.min.to_vec2());
        let pin_state = self
            .pins
            .get(&pin_spec.id)
            .map(|p| p.state.clone())
            .unwrap_or_default();
        let mut pin = Pin {
            spec: pin_spec.clone(),
            state: pin_state,
        };
        pin.state.parent_node_idx = node.spec.id;
        pin.state.attribute_rect = label_rect_relative_to_node_origin;
        pin.state.color_style = self.settings.style.format_pin(pin.spec.style_args.clone());
        pin.state.shape_gui = Some(ui.painter().add(egui::Shape::Noop));
        if !node.state.pin_indices.contains(&pin.spec.id) {
            node.state.pin_indices.push(pin.spec.id);
        }
        if ui.rect_contains_pointer(response.inner.response.rect)
            && ui.input(|i| i.pointer.primary_down())
        {
            self.frame_state.active_pin = Some(pin.spec.id);
            self.frame_state.interactive_node_index = Some(node.spec.id);
        }
        self.frame_state.pins_tmp.insert(pin.spec.id, pin);
    }

    fn add_link(&mut self, link_spec: LinkSpec, ui: &mut egui::Ui) {
        /* Implementierung wie oben */
        let link_id = link_spec.id;
        let entry = self.links.entry(link_id).or_insert_with(|| Link {
            spec: link_spec.clone(),
            state: Default::default(),
        });
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
        // Finde die erste x-Koordinate (im Grid-Space), die rechts vom linken Bildschirmrand liegt
        let grid_x_min_for_screen =
            visible_rect.min.x - canvas_origin_screen.x - self.state.panning.x;
        let x_start_index = (grid_x_min_for_screen / spacing).floor() as i32;
        let mut grid_x = x_start_index as f32 * spacing + self.state.panning.x.rem_euclid(spacing); // Erste sichtbare Linie (Grid-Space)
        loop {
            let screen_x = self.grid_space_to_screen_space(egui::pos2(grid_x, 0.0)).x; // Konvertiere zu Screen-Space
            if screen_x > visible_rect.max.x {
                break;
            } // Schleife beenden, wenn außerhalb des sichtbaren Bereichs
            if screen_x >= visible_rect.min.x {
                // Nur zeichnen, wenn im sichtbaren Bereich
                painter.line_segment(
                    [
                        egui::pos2(screen_x, visible_rect.min.y),
                        egui::pos2(screen_x, visible_rect.max.y),
                    ],
                    line_stroke,
                );
            }
            grid_x += spacing;
        }

        // Horizontale Linien
        let grid_y_min_for_screen =
            visible_rect.min.y - canvas_origin_screen.y - self.state.panning.y;
        let y_start_index = (grid_y_min_for_screen / spacing).floor() as i32;
        let mut grid_y = y_start_index as f32 * spacing + self.state.panning.y.rem_euclid(spacing); // Erste sichtbare Linie (Grid-Space)
        loop {
            let screen_y = self.grid_space_to_screen_space(egui::pos2(0.0, grid_y)).y; // Konvertiere zu Screen-Space
            if screen_y > visible_rect.max.y {
                break;
            }
            if screen_y >= visible_rect.min.y {
                painter.line_segment(
                    [
                        egui::pos2(visible_rect.min.x, screen_y),
                        egui::pos2(visible_rect.max.x, screen_y),
                    ],
                    line_stroke,
                );
            }
            grid_y += spacing;
        }
    }

    fn draw_link(&mut self, link_id: usize, ui: &mut egui::Ui) {
        let link_hovered = self.frame_state.hovered_link_idx == Some(link_id);
        if let Some(link) = self.links.get(&link_id) {
            // immutable borrow
            if let (Some(start_pin), Some(end_pin)) = (
                self.pins.get(&link.spec.start_pin_index),
                self.pins.get(&link.spec.end_pin_index),
            ) {
                let link_data = LinkBezierData::get_link_renderable(
                    start_pin.state.pos,
                    end_pin.state.pos,
                    start_pin.spec.kind,
                    self.settings.style.link_line_segments_per_length,
                );
                let mut color = link.state.style.base;
                if self.state.selected_link_indices.contains(&link_id) {
                    color = link.state.style.selected;
                } else if link_hovered {
                    color = link.state.style.hovered;
                }
                if let Some(shape_idx) = link.state.shape {
                    ui.painter().set(
                        shape_idx,
                        link_data.draw((link.state.style.thickness, color)),
                    );
                }
            }
        }
    }

    fn draw_node(&mut self, node_id: usize, ui: &mut egui::Ui) {
        let mut pin_ids = Vec::new();
        // Nimmt &mut self
        if let Some(node) = self.nodes.get(&node_id) {
            let node_hovered = self.frame_state.hovered_node_index == Some(node_id);
            let is_selected = self.state.selected_node_indices.contains(&node_id);
            let mut bg_col = node.state.color_style.background;
            let mut title_col = node.state.color_style.titlebar;
            let mut outline_col = node.state.color_style.outline;
            if is_selected {
                bg_col = node.state.color_style.background_selected;
                title_col = node.state.color_style.titlebar_selected;
                outline_col = node.state.color_style.background_selected;
            }
            // War node.state.color_style.background_selected
            else if node_hovered {
                bg_col = node.state.color_style.background_hovered;
                title_col = node.state.color_style.titlebar_hovered;
            }
            if node.spec.active {
                outline_col = self.settings.style.colors[ColorStyle::NodeOutlineActive as usize];
            }

            let painter = ui.painter();
            let screen_rect = egui::Rect::from_min_size(
                self.grid_space_to_screen_space(node.state.rect.min),
                node.state.rect.size(),
            );
            let screen_title_rect = node.state.get_node_title_rect_screen(self);
            let rounding = CornerRadius::from(node.state.layout_style.corner_rounding);

            if let Some(idx) = node.state.background_shape {
                painter.set(idx, egui::Shape::rect_filled(screen_rect, rounding, bg_col));
            }
            if node.state.title_bar_content_rect.height() > 0.0 {
                if let Some(idx) = node.state.titlebar_shape {
                    painter.set(
                        idx,
                        egui::Shape::rect_filled(screen_title_rect, rounding, title_col),
                    );
                }
            }
            if (self.settings.style.flags & StyleFlags::NodeOutline as usize) != 0 {
                if let Some(idx) = node.state.outline_shape {
                    painter.set(
                        idx,
                        egui::Shape::Rect(egui::epaint::RectShape {
                            rect: screen_rect,
                            corner_radius: rounding,
                            fill: Color32::TRANSPARENT,
                            stroke: egui::Stroke::new(
                                node.state.layout_style.border_thickness,
                                outline_col,
                            ),
                            stroke_kind: StrokeKind::Outside, // Default is Outer
                            round_to_pixels: Some(true),      // Wrap in Some()
                            blur_width: 0.0,                  // Default is 0.0
                            brush: None,                      // No gradient brush
                        }),
                    );
                }
            }

            pin_ids = node.state.pin_indices.clone();
        }
        // Jetzt über die geklonten IDs iterieren und mutable draw_pin aufrufen
        for pin_id in pin_ids {
            // Verwende die Liste aus dem äußeren Scope
            if self.pins.contains_key(&pin_id) {
                // Sicherstellen, dass Pin existiert
                self.draw_pin(pin_id, ui); // Ruft mutable Methode auf
            }
        }
    }

    fn draw_pin(&mut self, pin_idx: usize, ui: &mut egui::Ui) {
        /* Wie oben korrigiert */
        let pin_hovered = self.frame_state.hovered_pin_index == Some(pin_idx);
        let mut screen_pos = egui::Pos2::ZERO;
        let mut needs_drawing = false;
        let mut draw_color = Color32::MAGENTA;
        let mut draw_shape = PinShape::Circle;
        let mut shape_idx_opt = None;
        let mut pin_spec_flags = 0;

        // Daten lesen (immutable leihen)
        if let Some(pin) = self.pins.get(&pin_idx) {
            pin_spec_flags = pin.spec.flags; // Flags speichern für später
            if let Some(parent_node) = self.nodes.get(&pin.state.parent_node_idx) {
                let parent_screen_rect = egui::Rect::from_min_size(
                    self.grid_space_to_screen_space(parent_node.state.rect.min),
                    parent_node.state.rect.size(),
                );
                let attr_screen_min = self.grid_space_to_screen_space(
                    parent_node.state.rect.min + pin.state.attribute_rect.min.to_vec2(),
                );
                let attr_screen_rect =
                    egui::Rect::from_min_size(attr_screen_min, pin.state.attribute_rect.size());
                screen_pos = self.settings.style.get_screen_space_pin_coordinates(
                    &parent_screen_rect,
                    &attr_screen_rect,
                    pin.spec.kind,
                );
                draw_color = if pin_hovered {
                    pin.state.color_style.hovered
                } else {
                    pin.state.color_style.background
                };
                draw_shape = pin.state.color_style.shape;
                shape_idx_opt = pin.state.shape_gui;
                needs_drawing = true;
            }
        }

        // Zeichnen und State Update (braucht &mut self.pins)
        if needs_drawing {
            if let Some(shape_idx) = shape_idx_opt {
                self.settings
                    .style
                    .draw_pin_shape(screen_pos, draw_shape, draw_color, shape_idx, ui);
            }
            if let Some(pin_mut) = self.pins.get_mut(&pin_idx) {
                pin_mut.state.pos = screen_pos;
            } // Update Position
            if pin_hovered {
                self.frame_state.hovered_pin_flags = pin_spec_flags;
            } // Flags nur setzen wenn hovered
        }
    }

    fn draw_temporary_elements(&self, ui: &mut egui::Ui) {
        /* Implementierung wie oben */
        if self.state.click_interaction_type == ClickInteractionType::LinkCreation {
            if let Some(start_pin) = self.pins.get(
                &self
                    .state
                    .click_interaction_state
                    .link_creation
                    .start_pin_idx,
            ) {
                let start_pos = start_pin.state.pos;
                let end_pos = self
                    .state
                    .click_interaction_state
                    .link_creation
                    .end_pin_index
                    .and_then(|id| self.pins.get(&id))
                    .map_or(self.state.interaction_state.mouse_pos, |p| p.state.pos);
                let link_data = LinkBezierData::get_link_renderable(
                    start_pos,
                    end_pos,
                    start_pin.spec.kind,
                    self.settings.style.link_line_segments_per_length,
                );
                ui.painter().add(link_data.draw((
                    self.settings.style.link_thickness,
                    self.settings.style.colors[ColorStyle::Link as usize],
                )));
            }
        }
        if self.state.click_interaction_type == ClickInteractionType::BoxSelection {
            let selection_rect = self.state.click_interaction_state.box_selection;
            ui.painter().rect_filled(
                selection_rect,
                CornerRadius::ZERO,
                self.settings.style.colors[ColorStyle::BoxSelector as usize],
            );
            ui.painter().rect_stroke(
                selection_rect,
                CornerRadius::ZERO,
                egui::Stroke::new(
                    1.0,
                    self.settings.style.colors[ColorStyle::BoxSelectorOutline as usize],
                ),
                StrokeKind::Outside, // Add the missing StrokeKind argument
            );
        }
    }

    // --- Coordinate Systems ---
    fn screen_space_to_grid_space(&self, v: egui::Pos2) -> egui::Pos2 {
        v - self.frame_state.canvas_origin_screen_space() - self.state.panning
    }
    fn grid_space_to_screen_space(&self, v: egui::Pos2) -> egui::Pos2 {
        v + self.frame_state.canvas_origin_screen_space() + self.state.panning
    }
    fn editor_space_to_screen_space(&self, v: egui::Pos2) -> egui::Pos2 {
        v + self.frame_state.canvas_origin_screen_space()
    }

    fn get_screen_space_pin_coordinates(&self, pin: &Pin) -> egui::Pos2 {
        let Some(parent_node) = self.nodes.get(&pin.state.parent_node_idx) else {
            return pin.state.pos;
        }; // Return current if node gone
        let node_rect_screen = egui::Rect::from_min_size(
            self.grid_space_to_screen_space(parent_node.state.rect.min),
            parent_node.state.rect.size(),
        );
        // attribute_rect is relative to node origin (grid space)
        let attr_rect_min_grid =
            parent_node.state.rect.min + pin.state.attribute_rect.min.to_vec2();
        let attr_rect_screen = egui::Rect::from_min_size(
            self.grid_space_to_screen_space(attr_rect_min_grid),
            pin.state.attribute_rect.size(),
        );

        self.settings.style.get_screen_space_pin_coordinates(
            &node_rect_screen,
            &attr_rect_screen,
            pin.spec.kind,
        )
    }

    // --- Resolves (Mit eingefügtem Code) ---
    fn resolve_hover_state(&mut self) {
        if !self.state.interaction_state.mouse_in_canvas {
            return;
        }
        self.resolve_occluded_pins();
        self.resolve_hovered_pin();
        if self.frame_state.hovered_pin_index.is_none() {
            self.resolve_hovered_node();
        }
        if self.frame_state.hovered_pin_index.is_none()
            && self.frame_state.hovered_node_index.is_none()
        {
            self.resolve_hovered_link();
        }
    }

    fn resolve_occluded_pins(&mut self) {
        self.frame_state.occluded_pin_indices.clear();
        let depth_stack = &self.state.node_depth_order;
        if depth_stack.len() < 2 {
            return;
        }

        for i in 0..(depth_stack.len() - 1) {
            if let Some(node_below) = self.nodes.get(&depth_stack[i]) {
                // Klonen der Pin-IDs, um Borrowing-Konflikt zu vermeiden, falls Pin-Map später geändert wird
                let pin_indices_below = node_below.state.pin_indices.clone();
                for j in (i + 1)..depth_stack.len() {
                    if let Some(node_above) = self.nodes.get(&depth_stack[j]) {
                        // Node Rect ist Grid Space -> Konvertieren zu Screen Space
                        let screen_rect_above = egui::Rect::from_min_size(
                            self.grid_space_to_screen_space(node_above.state.rect.min),
                            node_above.state.rect.size(),
                        );
                        for pin_id in &pin_indices_below {
                            if let Some(pin) = self.pins.get(pin_id) {
                                // pin.state.pos ist Screen Space (wird in draw_pin gesetzt)
                                if screen_rect_above.contains(pin.state.pos) {
                                    self.frame_state.occluded_pin_indices.push(*pin_id);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn resolve_hovered_pin(&mut self) {
        self.frame_state.hovered_pin_index = None; // Start with no hovered pin
        let mut smallest_dist_sq = self.settings.style.pin_hover_radius.powi(2);
        let mouse_pos = self.state.interaction_state.mouse_pos;

        for (id, pin) in self.pins.iter() {
            // Skip if pin is occluded by another node
            if self.frame_state.occluded_pin_indices.contains(id) {
                continue;
            }
            // pin.state.pos is already in Screen Space (set by draw_pin)
            let dist_sq = pin.state.pos.distance_sq(mouse_pos);
            if dist_sq < smallest_dist_sq {
                smallest_dist_sq = dist_sq;
                self.frame_state.hovered_pin_index = Some(*id);
            }
        }
    }

    fn resolve_hovered_node(&mut self) {
        self.frame_state.hovered_node_index = None; // Reset
        let mut max_depth = -1_isize;

        // Iterate over nodes previously identified as under the mouse pointer
        for node_id in self.frame_state.node_indices_overlapping_with_mouse.iter() {
            // Find the depth of this node in the drawing order
            if let Some(depth) = self
                .state
                .node_depth_order
                .iter()
                .position(|id| id == node_id)
            {
                let depth_isize = depth as isize;
                // If this node is higher (later in the list) than the current max, it's the hovered one
                if depth_isize > max_depth {
                    max_depth = depth_isize;
                    self.frame_state.hovered_node_index = Some(*node_id);
                }
            }
        }
    }

    fn resolve_hovered_link(&mut self) {
        self.frame_state.hovered_link_idx = None; // Reset
        let mut smallest_dist = self.settings.style.link_hover_distance;
        let mouse_pos = self.state.interaction_state.mouse_pos;

        for (id, link) in self.links.iter() {
            // Ensure pins exist
            if let (Some(start_pin), Some(end_pin)) = (
                self.pins.get(&link.spec.start_pin_index),
                self.pins.get(&link.spec.end_pin_index),
            ) {
                // Priority to pin hover: if mouse is hovering over either pin of this link, link is not hovered.
                if Some(link.spec.start_pin_index) == self.frame_state.hovered_pin_index
                    || Some(link.spec.end_pin_index) == self.frame_state.hovered_pin_index
                {
                    continue; // Skip link if a pin is hovered
                }

                // Calculate Bezier curve using screen-space pin positions
                let link_data = LinkBezierData::get_link_renderable(
                    start_pin.state.pos,
                    end_pin.state.pos,
                    start_pin.spec.kind,
                    self.settings.style.link_line_segments_per_length,
                );
                // Coarse check: Is mouse within the bounding box of the curve?
                let containing_rect = link_data
                    .bezier
                    .get_containing_rect_for_bezier_curve(smallest_dist);
                if containing_rect.contains(mouse_pos) {
                    // Fine check: Calculate actual distance to the curve
                    let dist = link_data.get_distance_to_cubic_bezier(&mouse_pos);
                    if dist < smallest_dist {
                        smallest_dist = dist;
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
                self.state.click_interaction_type = ClickInteractionType::BoxSelection;
                self.state.click_interaction_state.box_selection = egui::Rect::from_min_max(
                    self.state.interaction_state.mouse_pos,
                    self.state.interaction_state.mouse_pos,
                );
            }
        }
    }

    fn begin_link_interaction(&mut self, link_id: usize) {
        let detach_mod = self
            .state
            .interaction_state
            .link_detatch_with_modifier_click;
        let hovered_pin_id = self.frame_state.hovered_pin_index;
        let pin_flags = hovered_pin_id
            .and_then(|id| self.pins.get(&id))
            .map_or(0, |p| p.spec.flags);

        // Check for detach on pin click first
        if (pin_flags & AttributeFlags::EnableLinkDetachWithDragClick as usize) != 0 {
            if let Some(pin_id) = hovered_pin_id {
                self.begin_link_detach(link_id, pin_id);
                return;
            }
        }
        // Check for detach with modifier
        if detach_mod {
            if let Some(link) = self.links.get(&link_id) {
                if let (Some(start_pin), Some(end_pin)) = (
                    self.pins.get(&link.spec.start_pin_index),
                    self.pins.get(&link.spec.end_pin_index),
                ) {
                    let dist_start_sq = start_pin
                        .state
                        .pos
                        .distance_sq(self.state.interaction_state.mouse_pos);
                    let dist_end_sq = end_pin
                        .state
                        .pos
                        .distance_sq(self.state.interaction_state.mouse_pos);
                    let closest = if dist_start_sq < dist_end_sq {
                        link.spec.start_pin_index
                    } else {
                        link.spec.end_pin_index
                    };
                    self.begin_link_detach(link_id, closest);
                    return;
                }
            }
        }
        // Default: select link
        self.begin_link_selection(link_id);
    }

    fn begin_link_creation(&mut self, pin_id: usize) {
        self.state.click_interaction_type = ClickInteractionType::LinkCreation;
        self.state
            .click_interaction_state
            .link_creation
            .start_pin_idx = pin_id;
        self.state
            .click_interaction_state
            .link_creation
            .end_pin_index = None;
        self.state
            .click_interaction_state
            .link_creation
            .link_creation_type = LinkCreationType::Standard;
        self.frame_state.element_state_change.link_started = true;
    }

    fn begin_link_selection(&mut self, link_id: usize) {
        if self.state.click_interaction_type != ClickInteractionType::Link
            || !self.state.selected_link_indices.contains(&link_id)
        {
            self.state.click_interaction_type = ClickInteractionType::Link;
            self.state.selected_node_indices.clear();
            self.state.selected_link_indices.clear();
            self.state.selected_link_indices.push(link_id);
        }
    }

    fn begin_node_selection(&mut self, node_id: usize) {
        if self.state.click_interaction_type != ClickInteractionType::None {
            return;
        }
        self.state.click_interaction_type = ClickInteractionType::Node;
        if !self.state.selected_node_indices.contains(&node_id) {
            self.state.selected_node_indices.clear();
            self.state.selected_link_indices.clear();
            self.state.selected_node_indices.push(node_id);
            self.frame_state.just_selected_node = true;
        }
        if let Some(pos) = self
            .state
            .node_depth_order
            .iter()
            .position(|x| *x == node_id)
        {
            let id = self.state.node_depth_order.remove(pos);
            self.state.node_depth_order.push(id);
        } else {
            self.state.node_depth_order.push(node_id);
        }
    }

    fn begin_link_detach(&mut self, link_id: usize, detach_pin_idx: usize) {
        if let Some(link) = self.links.remove(&link_id) {
            // Remove immediately
            let other_pin = if link.spec.start_pin_index == detach_pin_idx {
                link.spec.end_pin_index
            } else {
                link.spec.start_pin_index
            };
            if self.pins.contains_key(&other_pin) {
                // Only proceed if other pin exists
                self.state.click_interaction_type = ClickInteractionType::LinkCreation;
                self.state
                    .click_interaction_state
                    .link_creation
                    .start_pin_idx = other_pin;
                self.state
                    .click_interaction_state
                    .link_creation
                    .end_pin_index = None;
                self.state
                    .click_interaction_state
                    .link_creation
                    .link_creation_type = LinkCreationType::FromDetach;
                self.frame_state.element_state_change.link_started = true; // Also a link start
            }
            self.frame_state.deleted_link_idx = Some(link_id); // Always record deletion
            self.frame_state
                .graph_changes
                .push(GraphChange::LinkRemoved(link_id));
        }
    }

    // --- Update/Logic Methods ---
    fn translate_selected_nodes(&mut self) {
        /* Implementierung wie oben */
        if self.state.interaction_state.left_mouse_dragging {
            let delta_screen = self.state.interaction_state.mouse_delta;
            // Wandel Delta zu Grid-Space Delta um (ignoriert Panning-Änderung während Drag)
            let delta_grid = delta_screen; // Panning ändert sich während Drag nicht

            if delta_grid.length_sq() > 0.0 {
                let mut changes = Vec::new();
                for idx in &self.state.selected_node_indices {
                    if let Some(node) = self.nodes.get_mut(idx) {
                        if node.state.draggable {
                            node.spec.origin += delta_grid; // Update spec origin (grid space)
                            node.state.rect = node.state.rect.translate(delta_grid); // Update state rect (grid space)
                            changes.push((
                                *idx,
                                BevyVec2::new(node.spec.origin.x, node.spec.origin.y),
                            ));
                            // Pin state.pos (screen space) wird in draw_pin neu berechnet
                        }
                    }
                }
                // Sende alle Änderungen nach der Schleife
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
        start_pin_id: usize,
        hovered_pin_id: usize,
        duplicate_link: Option<usize>,
    ) -> bool {
        /* Implementierung wie oben */
        let Some(start_pin) = self.pins.get(&start_pin_id) else {
            return false;
        };
        let Some(end_pin) = self.pins.get(&hovered_pin_id) else {
            return false;
        };
        if start_pin.state.parent_node_idx == end_pin.state.parent_node_idx {
            return false;
        }
        if start_pin.spec.kind == end_pin.spec.kind {
            return false;
        }
        if duplicate_link.is_some() {
            return false;
        }
        true
    }

    fn box_selector_update_selection(&mut self) -> egui::Rect {
        let mut box_rect_screen = self.state.click_interaction_state.box_selection; // Nimm den Wert direkt
                                                                                    // Manuelles Normalisieren (bleibt bestehen, da wir es hier brauchen!)
        if box_rect_screen.min.x > box_rect_screen.max.x {
            std::mem::swap(&mut box_rect_screen.min.x, &mut box_rect_screen.max.x);
        }
        if box_rect_screen.min.y > box_rect_screen.max.y {
            std::mem::swap(&mut box_rect_screen.min.y, &mut box_rect_screen.max.y);
        }

        // Wandle Box zu Grid-Space für Vergleich mit Node Rects
        let box_rect_grid = egui::Rect::from_min_size(
            self.screen_space_to_grid_space(box_rect_screen.min),
            box_rect_screen.size(),
        );

        let old_selected_nodes = self.state.selected_node_indices.clone();
        self.state.selected_node_indices.clear();
        self.state.selected_link_indices.clear();
        for (id, node) in self.nodes.iter() {
            if box_rect_grid.intersects(node.state.rect) {
                self.state.selected_node_indices.push(*id);
            }
        } // Vergleiche Grid-Space Rects
        self.state.selected_node_indices.sort_by_key(|id| {
            old_selected_nodes
                .iter()
                .position(|&oid| oid == *id)
                .unwrap_or(usize::MAX)
        });
        for (id, link) in self.links.iter() {
            if let (Some(start_pin), Some(end_pin)) = (
                self.pins.get(&link.spec.start_pin_index),
                self.pins.get(&link.spec.end_pin_index),
            ) {
                if self.nodes.contains_key(&start_pin.state.parent_node_idx)
                    && self.nodes.contains_key(&end_pin.state.parent_node_idx)
                {
                    if self.rectangle_overlaps_link(
                        &box_rect_screen,
                        &start_pin.state.pos,
                        &end_pin.state.pos,
                        start_pin.spec.kind,
                    ) {
                        self.state.selected_link_indices.push(*id);
                    }
                }
            }
        } // Link check braucht Screen-Space
        box_rect_screen // Gibt Screen-Rect zurück
    }

    fn rectangle_overlaps_link(
        &self,
        rect: &egui::Rect,
        start: &egui::Pos2,
        end: &egui::Pos2,
        start_type: PinType,
    ) -> bool {
        /* Implementierung wie oben */
        let mut lrect = egui::Rect::from_min_max(*start, *end);
        if lrect.min.x > lrect.max.x {
            std::mem::swap(&mut lrect.min.x, &mut lrect.max.x);
        }
        if lrect.min.y > lrect.max.y {
            std::mem::swap(&mut lrect.min.y, &mut lrect.max.y);
        }
        if rect.intersects(lrect) {
            if rect.contains(*start) || rect.contains(*end) {
                return true;
            }
            let link_data = LinkBezierData::get_link_renderable(
                *start,
                *end,
                start_type,
                self.settings.style.link_line_segments_per_length,
            );
            return link_data.rectangle_overlaps_bezier(rect);
        }
        false
    }
    fn find_duplicate_link(&self, start_pin_id: usize, end_pin_id: usize) -> Option<usize> {
        /* Implementierung wie oben */
        let (p1, p2) = if start_pin_id < end_pin_id {
            (start_pin_id, end_pin_id)
        } else {
            (end_pin_id, start_pin_id)
        };
        for (id, link) in self.links.iter() {
            let (l1, l2) = if link.spec.start_pin_index < link.spec.end_pin_index {
                (link.spec.start_pin_index, link.spec.end_pin_index)
            } else {
                (link.spec.end_pin_index, link.spec.start_pin_index)
            };
            if p1 == l1 && p2 == l2 {
                return Some(*id);
            }
        }
        None
    }

    fn click_interaction_update(&mut self, _ui: &mut egui::Ui) {
        /* Implementierung von oben hier einfügen */
        // Beinhaltet: BoxSelection Größe ändern + Selection updaten, Node Dragging (ruft translate...), Link Creation Dragging + Snap Logic, Panning Update, Interaction beenden
        match self.state.click_interaction_type {
            ClickInteractionType::BoxSelection => {
                self.state.click_interaction_state.box_selection.max =
                    self.state.interaction_state.mouse_pos;
                self.box_selector_update_selection();
                if self.state.interaction_state.left_mouse_released {
                    self.state.click_interaction_type = ClickInteractionType::None;
                    let s = self.state.selected_node_indices.clone();
                    self.state.node_depth_order.retain(|id| !s.contains(id));
                    self.state.node_depth_order.extend(s);
                }
            }
            ClickInteractionType::Node => {
                self.translate_selected_nodes();
                if self.state.interaction_state.left_mouse_released {
                    self.state.click_interaction_type = ClickInteractionType::None;
                }
            }
            ClickInteractionType::Link => {
                if self.state.interaction_state.left_mouse_released {
                    self.state.click_interaction_type = ClickInteractionType::None;
                }
            }
            ClickInteractionType::LinkCreation => {
                let start_pin_id = self
                    .state
                    .click_interaction_state
                    .link_creation
                    .start_pin_idx;
                if self.pins.get(&start_pin_id).is_none() {
                    self.state.click_interaction_type = ClickInteractionType::None;
                    return;
                };
                let mut snapped_pin_id = None;
                if let Some(hovered_pin_id) = self.frame_state.hovered_pin_index {
                    let duplicate_link = self.find_duplicate_link(start_pin_id, hovered_pin_id);
                    if self.should_link_snap_to_pin(start_pin_id, hovered_pin_id, duplicate_link) {
                        snapped_pin_id = Some(hovered_pin_id);
                    }
                }
                self.state
                    .click_interaction_state
                    .link_creation
                    .end_pin_index = snapped_pin_id;
                if self.state.interaction_state.left_mouse_released {
                    if snapped_pin_id.is_some() {
                        self.frame_state.element_state_change.link_created = true;
                    } else {
                        self.frame_state.element_state_change.link_dropped = true;
                    }
                    self.state.click_interaction_type = ClickInteractionType::None;
                }
            }
            ClickInteractionType::Panning => {
                if self.state.interaction_state.alt_mouse_dragging {
                    self.state.panning += self.state.interaction_state.mouse_delta;
                } else if !self.state.interaction_state.alt_mouse_clicked
                    && !self.state.interaction_state.alt_mouse_dragging
                {
                    self.state.click_interaction_type = ClickInteractionType::None;
                }
            }
            ClickInteractionType::None => {}
        }
    }

    fn handle_delete(&mut self) {
        /* Implementierung von oben hier einfügen */
        let nodes_to_remove: Vec<usize> = self.state.selected_node_indices.drain(..).collect();
        let links_to_remove: Vec<usize> = self.state.selected_link_indices.drain(..).collect();
        let mut pins_to_remove = Vec::new();
        for node_id in &nodes_to_remove {
            if let Some(node) = self.nodes.get(node_id) {
                pins_to_remove.extend(node.state.pin_indices.iter().copied());
            }
        }
        self.links.retain(|link_id, link| {
            let explicit_delete = links_to_remove.contains(link_id);
            let pin_deleted = pins_to_remove.contains(&link.spec.start_pin_index)
                || pins_to_remove.contains(&link.spec.end_pin_index);
            if explicit_delete || pin_deleted {
                self.frame_state
                    .graph_changes
                    .push(GraphChange::LinkRemoved(*link_id));
                false
            } else {
                true
            }
        });
        for node_id in nodes_to_remove {
            self.frame_state
                .graph_changes
                .push(GraphChange::NodeRemoved(node_id));
            self.nodes.remove(&node_id);
        }
        for pin_id in pins_to_remove {
            self.pins.remove(&pin_id);
        }
        self.state
            .node_depth_order
            .retain(|id| self.nodes.contains_key(id));
    }

    fn final_draw(&mut self, ui_draw: &mut egui::Ui) {
        // Nodes - draw_node nimmt jetzt &mut self
        let node_order = self.state.node_depth_order.clone(); // Clone wegen mutable borrow in draw_node
        for node_id in node_order.iter() {
            self.draw_node(*node_id, ui_draw);
        }
        // Links
        let link_ids: Vec<usize> = self.links.keys().copied().collect();
        for link_id in link_ids {
            self.draw_link(link_id, ui_draw);
        } // draw_link nimmt &mut self
          // Temporary elements
        self.draw_temporary_elements(ui_draw); // Nimmt &self
    }

    fn collect_events(&mut self) {
        /* Implementierung von oben hier einfügen */
        if self.frame_state.element_state_change.link_created {
            if let Some((start, end, _)) = self.link_created() {
                self.frame_state
                    .graph_changes
                    .push(GraphChange::LinkCreated(start, end));
            }
        }
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
    }
    #[allow(dead_code)]
    pub fn link_started(&self) -> Option<usize> {
        if self.frame_state.element_state_change.link_started {
            Some(
                self.state
                    .click_interaction_state
                    .link_creation
                    .start_pin_idx,
            )
        } else {
            None
        }
    }
    #[allow(dead_code)]
    pub fn link_dropped(&self, include_detached: bool) -> Option<usize> {
        if self.frame_state.element_state_change.link_dropped
            && (include_detached
                || self
                    .state
                    .click_interaction_state
                    .link_creation
                    .link_creation_type
                    != LinkCreationType::FromDetach)
        {
            Some(
                self.state
                    .click_interaction_state
                    .link_creation
                    .start_pin_idx,
            )
        } else {
            None
        }
    }
    pub fn link_created(&self) -> Option<(usize, usize, bool)> {
        /* Implementierung von oben */
        if self.frame_state.element_state_change.link_created {
            if let Some(end_pin_id) = self
                .state
                .click_interaction_state
                .link_creation
                .end_pin_index
            {
                let start_pin_id = self
                    .state
                    .click_interaction_state
                    .link_creation
                    .start_pin_idx;
                let start_pin_kind = self
                    .pins
                    .get(&start_pin_id)
                    .map_or(PinType::None, |p| p.spec.kind);
                let (output_pin_id, input_pin_id) = if start_pin_kind == PinType::Output {
                    (start_pin_id, end_pin_id)
                } else {
                    (end_pin_id, start_pin_id)
                };
                let created_from_snap = true;
                return Some((output_pin_id, input_pin_id, created_from_snap));
            }
        }
        None
    }
    #[allow(dead_code)]
    pub fn link_created_node(&self) -> Option<(usize, usize, usize, usize, bool)> {
        /* Implementierung von oben */
        if let Some((start_pin_id, end_pin_id, created_from_snap)) = self.link_created() {
            if let (Some(start_pin), Some(end_pin)) =
                (self.pins.get(&start_pin_id), self.pins.get(&end_pin_id))
            {
                let start_node_id = start_pin.state.parent_node_idx;
                let end_node_id = end_pin.state.parent_node_idx;
                return Some((
                    start_pin_id,
                    start_node_id,
                    end_pin_id,
                    end_node_id,
                    created_from_snap,
                ));
            }
        }
        None
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
        /* Implementierung wie oben */
        if !self.state.interaction_state.mouse_in_canvas {
            return;
        }
        if self.state.interaction_state.left_mouse_clicked {
            if let Some(pin_idx) = self.frame_state.hovered_pin_index {
                self.begin_link_creation(pin_idx);
            } else if let Some(node_idx) = self.frame_state.hovered_node_index {
                if self.frame_state.interactive_node_index != Some(node_idx) {
                    self.begin_node_selection(node_idx);
                }
            } else if let Some(link_idx) = self.frame_state.hovered_link_idx {
                self.begin_link_interaction(link_idx);
            } else {
                self.begin_canvas_interaction(false);
            }
        } else if self.state.interaction_state.alt_mouse_clicked {
            if self.frame_state.hovered_node_index.is_none()
                && self.frame_state.hovered_pin_index.is_none()
                && self.frame_state.hovered_link_idx.is_none()
            {
                self.begin_canvas_interaction(true);
            }
        }
    }
} // === Ende impl NodesContext ===

// === Hilfsstrukturen/Enums am Ende ===
#[derive(Derivative, Default, Debug)] // Default hinzugefügt
struct ElementStateChange {
    link_started: bool,
    link_dropped: bool,
    link_created: bool,
}
impl ElementStateChange {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum ClickInteractionType {
    Node,
    Link,
    LinkCreation,
    Panning,
    BoxSelection,
    None,
}
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
enum LinkCreationType {
    #[default]
    Standard,
    FromDetach,
} // Default zu Standard

#[derive(Derivative, Debug)]
#[derivative(Default)]
struct ClickInteractionStateLinkCreation {
    start_pin_idx: usize,
    end_pin_index: Option<usize>,
    #[derivative(Default(value = "LinkCreationType::default()"))]
    link_creation_type: LinkCreationType,
} // Verwende Default von Enum
#[derive(Derivative, Debug)]
#[derivative(Default)]
struct ClickInteractionState {
    link_creation: ClickInteractionStateLinkCreation,
    #[derivative(Default(value = "egui::Rect::ZERO"))]
    box_selection: egui::Rect,
} // Expliziter Default für Rect

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
            Modifier::None => mods.is_none(),
        }
    }
}

// === NodeState Helfer (mit Context für Konvertierung) ===
impl NodeState {
    fn get_node_title_rect_grid(&self) -> egui::Rect {
        let pad_y = self.layout_style.padding.y;
        egui::Rect::from_min_size(
            self.rect.min + egui::vec2(0.0, pad_y),
            egui::vec2(self.rect.width(), self.title_bar_content_rect.height()),
        )
    }
    fn get_node_title_rect_screen(&self, context: &NodesContext) -> egui::Rect {
        let grid_rect = self.get_node_title_rect_grid();
        egui::Rect::from_min_size(
            context.grid_space_to_screen_space(grid_rect.min),
            grid_rect.size(),
        )
    }
}
