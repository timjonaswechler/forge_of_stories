use bevy::prelude::*;
use std::collections::HashMap;

use super::ui_link::Link;
use super::ui_node::Node;
use super::ui_pin::Pin;

#[derive(Resource, Debug, Default)]
pub struct GraphStorage {
    pub nodes: HashMap<usize, Node>,
    pub pins: HashMap<usize, Pin>,
    pub links: HashMap<usize, Link>,
    // Später hier Methoden hinzufügen: add_node, get_node, remove_link etc.
}

impl GraphStorage {
    // --- Node Methoden ---
    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.spec.id, node);
    }

    pub fn get_node(&self, node_id: usize) -> Option<&Node> {
        self.nodes.get(&node_id)
    }

    pub fn get_node_mut(&mut self, node_id: usize) -> Option<&mut Node> {
        self.nodes.get_mut(&node_id)
    }

    pub fn remove_node(&mut self, node_id: usize) -> Option<Node> {
        self.nodes.remove(&node_id)
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = (&usize, &Node)> {
        self.nodes.iter()
    }

    // --- Pin Methoden ---
    pub fn add_pin(&mut self, pin: Pin) {
        self.pins.insert(pin.spec.id, pin);
    }

    pub fn get_pin(&self, pin_id: usize) -> Option<&Pin> {
        self.pins.get(&pin_id)
    }

    pub fn get_pin_mut(&mut self, pin_id: usize) -> Option<&mut Pin> {
        self.pins.get_mut(&pin_id)
    }

    pub fn remove_pin(&mut self, pin_id: usize) -> Option<Pin> {
        self.pins.remove(&pin_id)
    }

    pub fn contains_pin(&self, pin_id: usize) -> bool {
        self.pins.contains_key(&pin_id)
    }

    pub fn iter_pins(&self) -> impl Iterator<Item = (&usize, &Pin)> {
        self.pins.iter()
    }

    // --- Link Methoden ---
    pub fn add_link(&mut self, link: Link) {
        self.links.insert(link.spec.id, link);
    }

    pub fn get_link(&self, link_id: usize) -> Option<&Link> {
        self.links.get(&link_id)
    }

    pub fn get_link_mut(&mut self, link_id: usize) -> Option<&mut Link> {
        self.links.get_mut(&link_id)
    }

    pub fn remove_link(&mut self, link_id: usize) -> Option<Link> {
        self.links.remove(&link_id)
    }

    pub fn iter_links(&self) -> impl Iterator<Item = (&usize, &Link)> {
        self.links.iter()
    }

    // --- Andere Hilfsmethoden ---
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.pins.clear();
        self.links.clear();
    }
}
