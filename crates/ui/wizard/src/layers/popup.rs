#![allow(dead_code)]
//! Popup system for modal dialogs and overlays
//!
//! Popups work similar to Pages but are modal and overlay the current content.
//! They support flexible sizing and positioning.

use crate::layers::{Component, ComponentKey, ComponentStore, SlotId, Slots, SlotsAny};
use indexmap::IndexMap;
use ratatui::layout::Rect;
use slotmap::new_key_type;
use std::collections::HashMap;

new_key_type! { pub struct PopupKey; }

pub struct PopupMeta {
    pub title: String, /* … */
}

pub struct PopupGeometry {
    pub container: ratatui::layout::Rect,
    pub slots: Vec<ratatui::layout::Rect>,
}

pub type PopupLayout = fn(ratatui::layout::Rect) -> PopupGeometry;

pub trait PopupSpec {
    fn build(self, name: &str, b: &mut PopupBuilder<'_>);
}

pub struct Popup {
    pub key: PopupKey,
    pub kind_name: &'static str,
    pub components: Vec<ComponentKey>, // Tab-Reihenfolge
    pub slot_map: IndexMap<u64, Vec<ComponentKey>>, // slot-hash → comp-ids
    pub meta: PopupMeta,
    pub layout_any: Box<dyn Fn(Rect) -> SlotsAny + Send + Sync + 'static>, // typ-erased Slots
}

pub struct PopupBuilder<'a> {
    pub comps: &'a mut ComponentStore,
    pub popup_key: PopupKey,
    pub kind_name: &'static str,
    pub meta: PopupMeta,
    pub layout_any: Option<Box<dyn Fn(Rect) -> SlotsAny + Send + Sync + 'static>>,
    pub slot_map: IndexMap<u64, Vec<ComponentKey>>,
    pub components: Vec<ComponentKey>,
}

impl<'a> PopupBuilder<'a> {
    pub fn new(
        comps: &'a mut ComponentStore,
        popup_key: PopupKey,
        kind_name: &'static str,
        title: impl Into<String>,
    ) -> Self {
        Self {
            comps,
            popup_key,
            kind_name,
            meta: PopupMeta {
                title: title.into(),
            },
            layout_any: None,
            slot_map: IndexMap::new(),
            components: Vec::new(),
        }
    }
    pub fn title(&mut self, t: impl Into<String>) {
        self.meta.title = t.into();
    }
    pub fn layout<K: SlotId>(&mut self, f: fn(Rect) -> Slots<K>) {
        self.layout_any = Some(Box::new(move |area| SlotsAny::from_typed(&f(area))));
    }
    pub fn component<T: Component + 'static>(&mut self, c: T) -> ComponentKey {
        let k = self.comps.insert(c);
        self.components.push(k);
        k
    }
    pub fn place_in_slot<K: SlotId>(&mut self, id: ComponentKey, slot: K) {
        let key = crate::layers::slot_key(slot);
        self.slot_map.entry(key).or_default().push(id);
    }
    pub fn finish(self) -> Popup {
        Popup {
            key: self.popup_key,
            kind_name: self.kind_name,
            components: self.components,
            slot_map: self.slot_map,
            meta: self.meta,
            layout_any: self
                .layout_any
                .unwrap_or_else(|| Box::new(default_popup_slots)),
        }
    }
}

pub trait PopupWithSlots {
    type Slot: SlotId;
    fn layout(&self, area: Rect) -> Slots<Self::Slot>;
}

fn default_popup_slots(_: Rect) -> SlotsAny {
    SlotsAny {
        map: HashMap::new(),
    }
}

impl Popup {
    pub fn empty(key: PopupKey, kind_name: &'static str) -> Self {
        Self {
            key,
            kind_name,
            components: Vec::new(),
            slot_map: IndexMap::new(),
            meta: PopupMeta {
                title: String::new(),
            },
            layout_any: Box::new(default_popup_slots),
        }
    }
}
