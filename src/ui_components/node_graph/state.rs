// src/ui_components/node_graph/state.rs
use super::{
    context::GraphChange,
    interaction::{
        ClickInteractionState, ClickInteractionType, ElementStateChange, InteractionState,
    },
};
use super::{
    interaction::Modifier,
    ui_node::Node,
    ui_pin::{AttributeFlags, Pin},
};
use bevy::prelude::Resource;
use bevy_egui::egui;
use std::collections::HashMap;

// Manager für den persistenten Zustand (bleibt über Frames hinweg bestehen)
#[derive(Debug)]
pub struct PersistentStateManager {
    // Ursprünglich aus NodesContext.state
    pub interaction_state: InteractionState,
    pub selected_node_indices: Vec<usize>,
    pub selected_link_indices: Vec<usize>,
    pub node_depth_order: Vec<usize>,
    pub panning: egui::Vec2,
    pub click_interaction_type: ClickInteractionType,
    pub click_interaction_state: ClickInteractionState,
}
impl Default for PersistentStateManager {
    fn default() -> Self {
        Self {
            interaction_state: InteractionState::default(),
            selected_node_indices: Vec::new(),
            selected_link_indices: Vec::new(),
            node_depth_order: Vec::new(),
            panning: egui::Vec2::ZERO,
            click_interaction_type: ClickInteractionType::None,
            click_interaction_state: ClickInteractionState::default(),
        }
    }
}

// Manager für den Frame-spezifischen Zustand (wird pro Frame zurückgesetzt)
#[derive(Debug)]
pub struct FrameStateManager {
    // Ursprünglich aus NodesContext.frame_state
    pub canvas_rect_screen_space: egui::Rect,
    pub node_indices_overlapping_with_mouse: Vec<usize>,
    pub occluded_pin_indices: Vec<usize>,
    pub hovered_node_index: Option<usize>,
    pub interactive_node_index: Option<usize>,
    pub hovered_link_idx: Option<usize>,
    pub hovered_pin_index: Option<usize>,
    pub hovered_pin_flags: usize,
    pub deleted_link_idx: Option<usize>,
    pub snap_link_idx: Option<usize>,
    pub element_state_change: ElementStateChange,
    pub active_pin: Option<usize>,
    pub graph_changes: Vec<GraphChange>,
    pub pins_tmp: HashMap<usize, Pin>,
    pub nodes_tmp: HashMap<usize, Node>,
    pub just_selected_node: bool,
}
impl Default for FrameStateManager {
    fn default() -> Self {
        Self {
            canvas_rect_screen_space: egui::Rect::from_min_max(
                [0.0, 0.0].into(),
                [0.0, 0.0].into(),
            ),
            node_indices_overlapping_with_mouse: Vec::new(),
            occluded_pin_indices: Vec::new(),
            hovered_node_index: None,
            interactive_node_index: None,
            hovered_link_idx: None,
            hovered_pin_index: None,
            hovered_pin_flags: 0,
            deleted_link_idx: None,
            snap_link_idx: None,
            element_state_change: ElementStateChange::default(),
            active_pin: None,
            graph_changes: Vec::new(),
            pins_tmp: HashMap::new(),
            nodes_tmp: HashMap::new(),
            just_selected_node: false,
        }
    }
}

// Kombinierter Manager für beide Zustände
#[derive(Resource, Debug)]
pub struct GraphUiStateManager {
    pub persistent: PersistentStateManager,
    pub frame: FrameStateManager,
}

impl Default for GraphUiStateManager {
    fn default() -> Self {
        Self {
            persistent: PersistentStateManager::default(),
            frame: FrameStateManager::default(),
        }
    }
}

impl GraphUiStateManager {
    // === Frame-State Methoden ===

    pub fn reset_frame_state(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        self.frame.canvas_rect_screen_space = rect;
        self.frame.node_indices_overlapping_with_mouse.clear();
        self.frame.occluded_pin_indices.clear();
        self.frame.hovered_node_index = None;
        self.frame.interactive_node_index = None;
        self.frame.hovered_link_idx = None;
        self.frame.hovered_pin_index = None;
        self.frame.hovered_pin_flags = AttributeFlags::None as usize;
        self.frame.deleted_link_idx = None;
        self.frame.snap_link_idx = None;
        self.frame.element_state_change.reset();
        self.frame.active_pin = None;
        self.frame.graph_changes.clear();
        self.frame.just_selected_node = false;
        self.frame.pins_tmp.clear();
        self.frame.nodes_tmp.clear();
    }

    pub fn canvas_origin_screen_space(&self) -> egui::Vec2 {
        self.frame.canvas_rect_screen_space.min.to_vec2()
    }

    // Getter und Setter für die verschiedenen Eigenschaften

    pub fn add_node_overlapping_with_mouse(&mut self, node_id: usize) {
        self.frame.node_indices_overlapping_with_mouse.push(node_id);
    }

    pub fn set_hovered_pin_index(&mut self, index: Option<usize>) {
        self.frame.hovered_pin_index = index;
    }

    pub fn get_hovered_pin_index(&self) -> Option<usize> {
        self.frame.hovered_pin_index
    }

    pub fn set_hovered_pin_flags(&mut self, flags: usize) {
        self.frame.hovered_pin_flags = flags;
    }

    pub fn get_hovered_pin_flags(&self) -> usize {
        self.frame.hovered_pin_flags
    }

    pub fn set_hovered_node_index(&mut self, index: Option<usize>) {
        self.frame.hovered_node_index = index;
    }

    pub fn get_hovered_node_index(&self) -> Option<usize> {
        self.frame.hovered_node_index
    }

    pub fn set_hovered_link_idx(&mut self, index: Option<usize>) {
        self.frame.hovered_link_idx = index;
    }

    pub fn get_hovered_link_idx(&self) -> Option<usize> {
        self.frame.hovered_link_idx
    }

    pub fn add_occluded_pin_index(&mut self, pin_id: usize) {
        self.frame.occluded_pin_indices.push(pin_id);
    }

    pub fn get_occluded_pin_indices(&self) -> &[usize] {
        &self.frame.occluded_pin_indices
    }

    pub fn mark_link_started(&mut self) {
        self.frame.element_state_change.link_started = true;
    }

    pub fn mark_link_dropped(&mut self) {
        self.frame.element_state_change.link_dropped = true;
    }

    pub fn mark_link_created(&mut self) {
        self.frame.element_state_change.link_created = true;
    }

    pub fn mark_just_selected_node(&mut self, selected: bool) {
        self.frame.just_selected_node = selected;
    }

    pub fn is_node_just_selected(&self) -> bool {
        self.frame.just_selected_node
    }

    pub fn add_graph_change(&mut self, change: GraphChange) {
        self.frame.graph_changes.push(change);
    }

    pub fn get_graph_changes(&self) -> &[GraphChange] {
        &self.frame.graph_changes
    }

    pub fn get_temp_nodes_mut(&mut self) -> &mut HashMap<usize, Node> {
        &mut self.frame.nodes_tmp
    }

    pub fn get_temp_pins_mut(&mut self) -> &mut HashMap<usize, Pin> {
        &mut self.frame.pins_tmp
    }

    // === Persistent-State Methoden ===

    pub fn update_interaction_state(
        &mut self,
        io: &egui::InputState,
        opt_hover_pos: Option<egui::Pos2>,
        emulate_three_button_mouse: Modifier,
        link_detatch_with_modifier_click: Modifier,
        alt_mouse_button: Option<egui::PointerButton>,
    ) {
        self.persistent.interaction_state = self.persistent.interaction_state.update(
            io,
            opt_hover_pos,
            emulate_three_button_mouse,
            link_detatch_with_modifier_click,
            alt_mouse_button,
        );
    }

    pub fn get_interaction_state(&self) -> &InteractionState {
        &self.persistent.interaction_state
    }

    pub fn get_panning(&self) -> egui::Vec2 {
        self.persistent.panning
    }

    pub fn set_panning(&mut self, panning: egui::Vec2) {
        self.persistent.panning = panning;
    }

    pub fn get_selected_nodes(&self) -> &[usize] {
        &self.persistent.selected_node_indices
    }

    pub fn get_selected_links(&self) -> &[usize] {
        &self.persistent.selected_link_indices
    }

    pub fn add_selected_node(&mut self, node_id: usize) {
        self.persistent.selected_node_indices.push(node_id);
    }

    pub fn add_selected_link(&mut self, link_id: usize) {
        self.persistent.selected_link_indices.push(link_id);
    }

    pub fn clear_node_selection(&mut self) {
        self.persistent.selected_node_indices.clear();
    }

    pub fn clear_link_selection(&mut self) {
        self.persistent.selected_link_indices.clear();
    }

    pub fn is_node_selected(&self, node_id: usize) -> bool {
        self.persistent.selected_node_indices.contains(&node_id)
    }

    pub fn is_link_selected(&self, link_id: usize) -> bool {
        self.persistent.selected_link_indices.contains(&link_id)
    }

    pub fn set_click_interaction_type(&mut self, interaction_type: ClickInteractionType) {
        self.persistent.click_interaction_type = interaction_type;
    }

    pub fn get_click_interaction_type(&self) -> ClickInteractionType {
        self.persistent.click_interaction_type
    }

    pub fn get_click_interaction_state_mut(&mut self) -> &mut ClickInteractionState {
        &mut self.persistent.click_interaction_state
    }

    pub fn get_click_interaction_state(&self) -> &ClickInteractionState {
        &self.persistent.click_interaction_state
    }

    pub fn get_node_depth_order(&self) -> &[usize] {
        &self.persistent.node_depth_order
    }

    pub fn get_node_depth_order_mut(&mut self) -> &mut Vec<usize> {
        &mut self.persistent.node_depth_order
    }

    pub fn add_to_node_depth_order(&mut self, node_id: usize) {
        self.persistent.node_depth_order.push(node_id);
    }

    pub fn remove_from_node_depth_order(&mut self, node_id: usize) {
        self.persistent.node_depth_order.retain(|id| *id != node_id);
    }

    pub fn move_node_to_top(&mut self, node_id: usize) {
        if let Some(pos) = self
            .persistent
            .node_depth_order
            .iter()
            .position(|x| *x == node_id)
        {
            let id_to_move = self.persistent.node_depth_order.remove(pos);
            self.persistent.node_depth_order.push(id_to_move);
        }
    }
}
