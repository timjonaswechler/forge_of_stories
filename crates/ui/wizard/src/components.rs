mod aether_status_component;
mod info;
mod logo;
mod status_bar;

pub(crate) use aether_status_component::AetherStatusListComponent;
pub(crate) use info::Info;
pub(crate) use logo::Logo;
pub(crate) use status_bar::StatusBar;

use crate::{action::Action, layers::ActionOutcome};
use slotmap::SlotMap;
use slotmap::new_key_type;
use std::collections::HashMap;

new_key_type! { pub struct ComponentKey; }

pub struct ComponentStore {
    pub items: SlotMap<ComponentKey, Box<dyn Component>>,
    name_index: HashMap<String, ComponentKey>,
}

impl ComponentStore {
    pub fn new() -> Self {
        Self {
            items: SlotMap::with_key(),
            name_index: HashMap::new(),
        }
    }

    pub fn insert<T: Component + 'static>(&mut self, mut c: T) -> ComponentKey {
        let component_name = c.name().to_string();
        let normalized_name = component_name.to_ascii_lowercase();
        let key = self.items.insert_with_key(|k| {
            c.set_id(k);
            Box::new(c) as Box<dyn Component>
        });
        self.name_index.insert(component_name, key);
        self.name_index.insert(normalized_name, key);
        key
    }
    pub fn get(&self, k: ComponentKey) -> &dyn Component {
        &*self.items[k]
    }
    pub fn get_mut(&mut self, k: ComponentKey) -> &mut dyn Component {
        &mut *self.items[k]
    }
    pub fn find_by_name(&self, name: &str) -> Option<ComponentKey> {
        self.name_index
            .get(name)
            .or_else(|| self.name_index.get(&name.to_ascii_lowercase()))
            .copied()
    }
    pub fn is_focusable(&self, k: ComponentKey) -> bool {
        self.items[k].focusable()
    }
}

pub trait Component {
    fn name(&self) -> &str;
    fn id(&self) -> ComponentKey;
    fn set_id(&mut self, id: ComponentKey);
    fn focusable(&self) -> bool {
        true
    }
    fn on_focus(&mut self, _gained: bool) {}
    fn handle_action(&mut self, action: &Action) -> ActionOutcome;
    fn render(&self, f: &mut ratatui::Frame, area: ratatui::layout::Rect);
}
