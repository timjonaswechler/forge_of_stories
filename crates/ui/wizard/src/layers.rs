pub(crate) mod help;
pub(crate) mod notify;
pub(crate) mod page;
pub(crate) mod popup;

use crate::ui::components::{Component, ComponentKey, ComponentStore};
use indexmap::IndexMap;
use notify::{Notification, NotificationKey, NotificationKind};
use page::{Page, PageBuilder, PageKey, PageMeta, PageSpec};
use popup::{Popup, PopupBuilder, PopupKey, PopupMeta, PopupSpec};
use ratatui::layout::Rect;
use slotmap::SlotMap;
use std::collections::HashMap;

pub struct LayerSystem {
    pub pages: SlotMap<PageKey, Page>,
    pub popups: SlotMap<PopupKey, Popup>,
    pub notifications: SlotMap<NotificationKey, Notification>,
    pub components: ComponentStore,
    pub active: ActiveLayers,
}

pub struct ActiveLayers {
    pub page: Option<PageKey>,   // genau 1 erwartet (Option für Startup)
    pub popup: Option<PopupKey>, // 0..1
    pub show_help: bool,         // Help als Flag
    pub notification_order: Vec<NotificationKey>, // Anzeige-Reihenfolge
    pub focus: FocusPath,        // immer konsistent zu page/popup
}

#[derive(Clone, Copy, Debug)]
pub enum Surface {
    Page(PageKey),
    Popup(PopupKey),
}

#[derive(Clone, Copy, Debug)]
pub struct FocusPath {
    pub surface: Option<Surface>,        // None z. B. beim Startup
    pub component: Option<ComponentKey>, // None = Container-Fokus
}

pub struct RenderPlan<'a> {
    pub page: Option<&'a Page>,
    pub popup: Option<&'a Popup>,
    pub help: bool,
    pub notifications: Vec<&'a Notification>,
}
pub struct FocusDescriptor<'a> {
    pub surface_label: &'a str, // z.B. "Page: Dashboard" oder "Popup: Confirm"
    pub component_label: &'a str, // z.B. "Button: Save"
}

impl LayerSystem {
    pub fn new() -> Self {
        Self {
            pages: SlotMap::with_key(),
            popups: SlotMap::with_key(),
            components: ComponentStore::new(),
            notifications: SlotMap::with_key(),
            active: ActiveLayers {
                page: None,
                popup: None,
                show_help: false,
                notification_order: Vec::new(),
                focus: FocusPath {
                    surface: None,
                    component: None,
                },
            },
        }
    }

    pub fn register_page(&mut self, page: Page) -> PageKey {
        self.pages.insert(page)
    }
    pub fn register_popup(&mut self, popup: Popup) -> PopupKey {
        self.popups.insert(popup)
    }

    pub fn activate_page(&mut self, page: PageKey) {
        self.active.page = Some(page);
        // Fokus auf Page-First-Focusable:
        self.focus_first_on_surface(Surface::Page(page));
        // Popup bleibt unberührt (falls offen)
    }

    pub fn show_popup(&mut self, popup: PopupKey) {
        self.active.popup = Some(popup);
        self.focus_first_on_surface(Surface::Popup(popup));
    }

    pub fn close_popup(&mut self) {
        self.active.popup = None;
        // Fokus zurück zur Page
        if let Some(p) = self.active.page {
            self.focus_restore_or_first(Surface::Page(p));
        } else {
            self.active.focus = FocusPath {
                surface: None,
                component: None,
            };
        }
    }
    pub fn notify(
        &mut self,
        kind: NotificationKind,
        msg: impl Into<String>,
        ttl_ms: u64,
    ) -> NotificationKey {
        let key = self.notifications.insert_with_key(|k| Notification {
            id: k,
            kind,
            message: msg.into(),
            created_at: std::time::Instant::now(),
            ttl: std::time::Duration::from_millis(ttl_ms),
        });
        self.active.notification_order.push(key);
        key
    }

    pub fn tick_notifications(&mut self) {
        let now = std::time::Instant::now();
        self.active.notification_order.retain(|k| {
            if let Some(n) = self.notifications.get(*k) {
                if now.duration_since(n.created_at) < n.ttl {
                    return true;
                }
            }
            // Abgelaufen → entfernen
            self.notifications.remove(*k); // SlotMap remove
            false
        });
    }

    pub fn render_plan(&self) -> RenderPlan<'_> {
        RenderPlan {
            page: self.active.page.and_then(|k| self.pages.get(k)),
            popup: self.active.popup.and_then(|k| self.popups.get(k)),
            help: self.active.show_help,
            notifications: self
                .active
                .notification_order
                .iter()
                .filter_map(|k| self.notifications.get(*k))
                .collect(),
        }
    }

    pub fn focus_next(&mut self) {
        let surface = if self.active.popup.is_some() {
            Surface::Popup(self.active.popup.unwrap())
        } else {
            Surface::Page(self.active.page.unwrap())
        };

        let list: Vec<ComponentKey> = self.components_of(surface).to_vec();
        self.cycle_focus(&list, 1);
    }

    pub fn focus_prev(&mut self) {
        let surface = if self.active.popup.is_some() {
            Surface::Popup(self.active.popup.unwrap())
        } else {
            Surface::Page(self.active.page.unwrap())
        };

        let list: Vec<ComponentKey> = self.components_of(surface).to_vec();
        self.cycle_focus(&list, -1);
    }
    pub fn focus_component(&mut self, k: ComponentKey) {
        self.active.focus.component = Some(k);
    }
    pub fn focus_first_in_slot(&mut self, slot_hash: u64) {
        if let Some(pk) = self.active.page {
            if let Some(page) = self.pages.get(pk) {
                if let Some(list) = page.slot_map.get(&slot_hash) {
                    self.active.focus.component = list.first().copied();
                }
            }
        }
    }
    fn components_of(&self, s: Surface) -> &[ComponentKey] {
        match s {
            Surface::Page(k) => &self.pages.get(k).unwrap().components,
            Surface::Popup(k) => &self.popups.get(k).unwrap().components,
        }
    }

    fn cycle_focus(&mut self, list: &[ComponentKey], step: i32) {
        if list.is_empty() {
            self.active.focus.component = None;
            return;
        }
        let idx = match self.active.focus.component {
            Some(cur) => list.iter().position(|&c| c == cur).unwrap_or(0),
            None => 0,
        };
        let n = list.len() as i32;
        let next = ((idx as i32 + step).rem_euclid(n)) as usize;
        self.active.focus.component = Some(list[next]);
    }

    fn focus_first_on_surface(&mut self, s: Surface) {
        let list = self.components_of(s);
        self.active.focus.component = list.first().copied();
    }

    fn focus_restore_or_first(&mut self, s: Surface) {
        // Optional: pro Surface letzten Focus merken (Map<Surface, ComponentKey>)
        self.focus_first_on_surface(s)
    }

    pub fn describe_focus(&self) -> Option<FocusDescriptor<'_>> {
        let surface = if self.active.popup.is_some() {
            Surface::Popup(self.active.popup.unwrap())
        } else {
            Surface::Page(self.active.page.unwrap())
        };
        let (surface_label, comps) = match surface {
            Surface::Page(k) => {
                let p = self.pages.get(k)?;
                (p.meta.title.as_str(), &p.components[..])
            }
            Surface::Popup(k) => {
                let p = self.popups.get(k)?;
                (p.meta.title.as_str(), &p.components[..])
            }
        };
        let comp_label = self
            .active
            .focus
            .component
            .and_then(|ck| self.components.items.get(ck))
            .map(|c| c.name())
            .unwrap_or("-");
        Some(FocusDescriptor {
            surface_label,
            component_label: comp_label,
        })
    }
    pub fn create_page<S: PageSpec>(&mut self, name: &str, spec: S) -> PageKey {
        let key = self
            .pages
            .insert_with_key(|k| Page::empty(k, "page"));
        {
            let mut builder = PageBuilder::new(&mut self.components, key, "page", name);
            spec.build(name, &mut builder);
            let page = builder.finish();
            self.pages[key] = page;
        }
        key
    }

    pub fn create_popup<S: PopupSpec>(&mut self, name: &str, spec: S) -> PopupKey {
        let key = self
            .popups
            .insert_with_key(|k| Popup::empty(k, "popup"));
        {
            let mut builder = PopupBuilder::new(&mut self.components, key, "popup", name);
            spec.build(name, &mut builder);
            let popup = builder.finish();
            self.popups[key] = popup;
        }
        key
    }
}

pub trait SlotId: Eq + std::hash::Hash + Copy + 'static {}
impl<T: Eq + std::hash::Hash + Copy + 'static> SlotId for T {}
pub struct SlotsAny {
    pub map: HashMap<u64, Rect>,
} // typ-erased (u64 hash der Slot-Enum)

fn slot_key<K: SlotId>(k: K) -> u64 {
    // robuste Hash-Konversion (z. B. durch ahash), hier vereinfachend:
    // Safety: K: Copy; cast via mem::transmute wenn es kleine Enums sind.
    // Besser: eigener Hasher auf Bytes von K.
    use std::hash::{Hash, Hasher};
    let mut h = ahash::AHasher::default();
    k.hash(&mut h);
    h.finish()
}

impl SlotsAny {
    pub fn from_typed<K: SlotId>(s: &Slots<K>) -> Self {
        let mut map = HashMap::new();
        for (k, r) in s.iter() {
            map.insert(slot_key(*k), *r);
        }
        Self { map }
    }
    pub fn rect<K: SlotId>(&self, k: K) -> Option<Rect> {
        self.map.get(&slot_key(k)).copied()
    }
}
#[derive(Clone)]
pub struct Slots<K: SlotId> {
    map: HashMap<K, Rect>,
}
impl<K: SlotId> Slots<K> {
    pub fn empty() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn with(mut self, k: K, r: Rect) -> Self {
        self.map.insert(k, r);
        self
    }
    pub fn rect(&self, k: K) -> Option<Rect> {
        self.map.get(&k).copied()
    }
    pub fn iter(&self) -> impl Iterator<Item = (&K, &Rect)> {
        self.map.iter()
    }
}

pub enum LayerAction {
    OpenPage(PageKey), // optional: per Factory
    ShowPopup(PopupKey),
    ClosePopup,
    FocusNext,
    FocusPrev,
    ToggleHelp,
}
pub enum ActionOutcome {
    Consumed,
    NotHandled,
    RequestFocus(ComponentKey),
    Emit(/* domain events */),
}

impl LayerSystem {
    pub fn apply(&mut self, a: LayerAction) {
        match a {
            LayerAction::OpenPage(k) => self.activate_page(k),
            LayerAction::ShowPopup(k) => self.show_popup(k),
            LayerAction::ClosePopup => self.close_popup(),
            LayerAction::FocusNext => self.focus_next(),
            LayerAction::FocusPrev => self.focus_prev(),
            LayerAction::ToggleHelp => self.active.show_help = !self.active.show_help,
        }
    }
}

pub fn centered_box(area: Rect, w: u16, h: u16) -> Rect {
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect {
        x,
        y,
        width: w.min(area.width),
        height: h.min(area.height),
    }
}

pub fn default_popup_layout(area: Rect) -> Rect {
    centered_box(area, area.width.saturating_mul(60) / 100, 12) // 60% Breite, 12 Zeilen
}
