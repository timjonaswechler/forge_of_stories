mod dashboard;
mod welcome;
pub(crate) use dashboard::DashboardPage;
pub(crate) use welcome::WelcomePage;

use crate::{
    layers::{SlotId, Slots, SlotsAny},
    ui::components::{Component, ComponentKey, ComponentStore},
};
use indexmap::IndexMap;
use ratatui::layout::Rect;
use slotmap::new_key_type;
use std::collections::HashMap;

new_key_type! { pub struct PageKey; }

pub type PageLayout = fn(ratatui::layout::Rect) -> Vec<ratatui::layout::Rect>;

pub struct PageMeta {
    pub title: String, /* … breadcrumb, icon, … */
}

pub trait PageSpec {
    fn build(self, name: &str, b: &mut PageBuilder<'_>);
}

pub struct Page {
    pub key: PageKey,
    pub kind_name: &'static str,
    pub components: Vec<ComponentKey>, // Tab-Reihenfolge
    pub slot_map: IndexMap<u64, Vec<ComponentKey>>, // slot-hash → comp-ids
    pub meta: PageMeta,
    pub layout_any: Box<dyn Fn(Rect) -> SlotsAny + Send + Sync + 'static>, // typ-erased Slots
}

pub struct PageBuilder<'a> {
    pub comps: &'a mut ComponentStore,
    pub page_key: PageKey,
    pub kind_name: &'static str,
    pub meta: PageMeta,
    pub layout_any: Option<Box<dyn Fn(Rect) -> SlotsAny + Send + Sync + 'static>>,
    pub slot_map: IndexMap<u64, Vec<ComponentKey>>,
    pub components: Vec<ComponentKey>,
}

impl<'a> PageBuilder<'a> {
    pub fn new(
        comps: &'a mut ComponentStore,
        page_key: PageKey,
        kind_name: &'static str,
        title: impl Into<String>,
    ) -> Self {
        Self {
            comps,
            page_key,
            kind_name,
            meta: PageMeta {
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

    pub fn component<T: Component + 'static>(&mut self, c: T) -> ComponentKey {
        let k = self.comps.insert(c);
        self.components.push(k);
        k
    }
    pub fn layout<K: SlotId>(&mut self, f: fn(Rect) -> Slots<K>) {
        self.layout_any = Some(Box::new(move |area| SlotsAny::from_typed(&f(area))));
    }
    pub fn place_in_slot<K: SlotId>(&mut self, id: ComponentKey, slot: K) {
        let key = crate::layers::slot_key(slot);
        self.slot_map.entry(key).or_default().push(id);
    }
    pub fn finish(self) -> Page {
        Page {
            key: self.page_key,
            kind_name: self.kind_name,
            components: self.components,
            slot_map: self.slot_map,
            meta: self.meta,
            layout_any: self
                .layout_any
                .unwrap_or_else(|| Box::new(default_page_layout)),
        }
    }
}

pub trait PageWithSlots {
    type Slot: SlotId;
    fn layout(&self, area: Rect) -> Slots<Self::Slot>;
}

fn default_page_layout(_: Rect) -> SlotsAny {
    SlotsAny {
        map: HashMap::new(),
    }
}

impl Page {
    pub fn empty(key: PageKey, kind_name: &'static str) -> Self {
        Self {
            key,
            kind_name,
            components: Vec::new(),
            slot_map: IndexMap::new(),
            meta: PageMeta {
                title: String::new(),
            },
            layout_any: Box::new(default_page_layout),
        }
    }
}
