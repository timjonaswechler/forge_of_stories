mod aether_status_component;
mod logo;
mod status_bar;

pub(crate) use aether_status_component::AetherStatusListComponent;
pub(crate) use logo::{Info, Logo};
pub(crate) use status_bar::StatusBar;

use crate::{action::Action, layers::ActionOutcome};
use slotmap::SlotMap;
use slotmap::new_key_type;

new_key_type! { pub struct ComponentKey; }

pub struct ComponentStore {
    pub items: SlotMap<ComponentKey, Box<dyn Component>>,
}

impl ComponentStore {
    pub fn new() -> Self {
        Self {
            items: SlotMap::with_key(),
        }
    }

    pub fn insert<T: Component + 'static>(&mut self, mut c: T) -> ComponentKey {
        let key = self.items.insert_with_key(|k| {
            c.set_id(k);
            Box::new(c) as Box<dyn Component>
        });
        key
    }
    pub fn get(&self, k: ComponentKey) -> &dyn Component {
        &*self.items[k]
    }
    pub fn get_mut(&mut self, k: ComponentKey) -> &mut dyn Component {
        &mut *self.items[k]
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
