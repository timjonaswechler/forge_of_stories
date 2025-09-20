pub(crate) mod help;
pub(crate) mod notify;
pub(crate) mod page;
pub(crate) mod popup;

use crate::components::{Component, ComponentKey, ComponentStore};
use notify::{Notification, NotificationKey, NotificationKind};
use page::{Page, PageBuilder, PageKey, PageSpec};
use popup::{Popup, PopupBuilder, PopupKey, PopupSpec};
use ratatui::layout::Rect;
use slotmap::SlotMap;
use std::collections::HashMap;

pub struct LayerSystem {
    pub pages: SlotMap<PageKey, Page>,
    pub popups: SlotMap<PopupKey, Popup>,
    pub notifications: SlotMap<NotificationKey, Notification>,
    pub components: ComponentStore,
    pub active: ActiveLayers,
    page_order: Vec<PageKey>,
    page_lookup: HashMap<String, PageKey>,
    popup_lookup: HashMap<String, PopupKey>,
}

pub struct ActiveLayers {
    pub page: Option<PageKey>,   // genau 1 erwartet (Option für Startup)
    pub popup: Option<PopupKey>, // 0..1
    pub notification_order: Vec<NotificationKey>, // Anzeige-Reihenfolge
    pub focus: FocusPath,        // immer konsistent zu page/popup
    pub help_visible: bool,
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
    pub notifications: Vec<&'a Notification>,
    pub help_visible: bool,
}

pub struct FocusDescriptor<'a> {
    pub surface_label: &'a str, // z.B. "Page: Dashboard" oder "Popup: Confirm"
    pub component_label: &'a str, // z.B. "Button: Save"
}

impl LayerSystem {
    fn normalize_id(id: &str) -> String {
        id.trim().to_ascii_lowercase()
    }

    pub fn new() -> Self {
        Self {
            pages: SlotMap::with_key(),
            popups: SlotMap::with_key(),
            components: ComponentStore::new(),
            notifications: SlotMap::with_key(),
            active: ActiveLayers {
                page: None,
                popup: None,
                notification_order: Vec::new(),
                focus: FocusPath {
                    surface: None,
                    component: None,
                },
                help_visible: false,
            },
            page_order: Vec::new(),
            page_lookup: HashMap::new(),
            popup_lookup: HashMap::new(),
        }
    }

    pub fn activate_page(&mut self, page: PageKey) {
        self.active.page = Some(page);
        // Fokus auf Page-First-Focusable:
        self.focus_first_on_surface(Surface::Page(page));
        self.active.focus.surface = Some(Surface::Page(page));
        // Popup bleibt unberührt (falls offen)
    }

    pub fn show_popup(&mut self, popup: PopupKey) {
        self.active.popup = Some(popup);
        self.focus_first_on_surface(Surface::Popup(popup));
        self.active.focus.surface = Some(Surface::Popup(popup));
    }

    pub fn close_popup(&mut self) {
        self.active.popup = None;
        // Fokus zurück zur Page
        if let Some(p) = self.active.page {
            self.focus_restore_or_first(Surface::Page(p));
            self.active.focus.surface = Some(Surface::Page(p));
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
            notifications: self
                .active
                .notification_order
                .iter()
                .filter_map(|k| self.notifications.get(*k))
                .collect(),
            help_visible: self.active.help_visible,
        }
    }

    pub fn focus_next(&mut self) {
        if let Some(surface) = self.active_surface() {
            if let Some(list) = self.components_of(surface) {
                let list: Vec<ComponentKey> = list.to_vec();
                self.cycle_focus(&list, 1);
            }
        }
    }

    pub fn focus_prev(&mut self) {
        if let Some(surface) = self.active_surface() {
            if let Some(list) = self.components_of(surface) {
                let list: Vec<ComponentKey> = list.to_vec();
                self.cycle_focus(&list, -1);
            }
        }
    }
    pub fn focus_component(&mut self, k: ComponentKey) {
        self.active.focus.component = Some(k);
    }

    fn active_surface(&self) -> Option<Surface> {
        self.active
            .popup
            .map(Surface::Popup)
            .or_else(|| self.active.page.map(Surface::Page))
    }

    fn components_of(&self, s: Surface) -> Option<&[ComponentKey]> {
        match s {
            Surface::Page(k) => self.pages.get(k).map(|p| p.components.as_slice()),
            Surface::Popup(k) => self.popups.get(k).map(|p| p.components.as_slice()),
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
        self.active.focus.component = self.components_of(s).and_then(|list| list.first().copied());
    }

    fn focus_restore_or_first(&mut self, s: Surface) {
        // Optional: pro Surface letzten Focus merken (Map<Surface, ComponentKey>)
        self.focus_first_on_surface(s)
    }

    pub fn create_page<S: PageSpec>(&mut self, name: &str, spec: S) -> PageKey {
        let key = self.pages.insert_with_key(|k| Page::empty(k));
        {
            let mut builder = PageBuilder::new(&mut self.components, key, "page");
            spec.build(name, &mut builder);
            let mut page = builder.finish();
            if page.context.is_empty() {
                page.context = name.to_ascii_lowercase();
            }
            page.context = Self::normalize_id(&page.context);
            self.page_lookup.insert(page.context.clone(), key);
            self.page_lookup.insert(Self::normalize_id(name), key);
            self.page_order.push(key);
            self.pages[key] = page;
        }
        key
    }

    pub fn create_popup<S: PopupSpec>(&mut self, name: &str, spec: S) -> PopupKey {
        let key = self.popups.insert_with_key(|k| Popup::empty(k, "popup"));
        {
            let mut builder = PopupBuilder::new(&mut self.components, key, "popup", name);
            spec.build(name, &mut builder);
            let mut popup = builder.finish();
            if popup.meta.title.is_empty() {
                popup.meta.title = name.to_string();
            }
            let ident = Self::normalize_id(&popup.meta.title);
            self.popup_lookup.insert(ident, key);
            self.popup_lookup.insert(Self::normalize_id(name), key);
            self.popups[key] = popup;
        }
        key
    }

    pub fn lookup_page(&self, id: &str) -> Option<PageKey> {
        self.page_lookup
            .get(&Self::normalize_id(id))
            .copied()
            .or_else(|| self.page_lookup.get(id).copied())
    }

    pub fn lookup_popup(&self, id: &str) -> Option<PopupKey> {
        self.popup_lookup
            .get(&Self::normalize_id(id))
            .copied()
            .or_else(|| self.popup_lookup.get(id).copied())
    }

    pub fn lookup_component(&self, id: &str) -> Option<ComponentKey> {
        self.components.find_by_name(id)
    }

    pub fn toggle_help(&mut self) {
        self.active.help_visible = !self.active.help_visible;
    }

    pub fn help_visible(&self) -> bool {
        self.active.help_visible
    }

    pub fn activate_next_page(&mut self) -> Option<PageKey> {
        let next_key = match self.active.page {
            Some(current) => {
                if self.page_order.is_empty() {
                    None
                } else {
                    let idx = self
                        .page_order
                        .iter()
                        .position(|&k| k == current)
                        .unwrap_or(0);
                    let next = (idx + 1) % self.page_order.len();
                    Some(self.page_order[next])
                }
            }
            None => self.page_order.first().copied(),
        };
        if let Some(key) = next_key {
            self.activate_page(key);
            return Some(key);
        }
        None
    }

    pub fn activate_previous_page(&mut self) -> Option<PageKey> {
        let prev_key = match self.active.page {
            Some(current) => {
                if self.page_order.is_empty() {
                    None
                } else {
                    let idx = self
                        .page_order
                        .iter()
                        .position(|&k| k == current)
                        .unwrap_or(0);
                    let next = (idx + self.page_order.len() - 1) % self.page_order.len();
                    Some(self.page_order[next])
                }
            }
            None => self.page_order.last().copied(),
        };
        if let Some(key) = prev_key {
            self.activate_page(key);
            return Some(key);
        }
        None
    }

    pub fn page_context(&self, key: PageKey) -> Option<&str> {
        self.pages.get(key).map(|p| p.context.as_str())
    }

    pub fn component_name(&self, key: ComponentKey) -> Option<&str> {
        self.components.items.get(key).map(|c| c.name())
    }

    pub fn focus_labels(&self) -> (Option<String>, Option<String>) {
        let surface = self.active.focus.surface.and_then(|surface| match surface {
            Surface::Page(k) => self.pages.get(k).map(|p| p.meta.title.clone()),
            Surface::Popup(k) => self.popups.get(k).map(|p| p.meta.title.clone()),
        });

        let component = self
            .active
            .focus
            .component
            .and_then(|key| self.components.items.get(key))
            .map(|component| component.name().to_string());

        (surface, component)
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
    use std::hash::Hasher;
    let mut h = ahash::AHasher::default();
    std::hash::Hash::hash(&k, &mut h);
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn rect(&self, k: K) -> Option<Rect> {
        self.map.get(&k).copied()
    }
    pub fn iter(&self) -> impl Iterator<Item = (&K, &Rect)> {
        self.map.iter()
    }
}

pub enum LayerAction {
    FocusNext,
    FocusPrev,
    FocusComponent(ComponentKey),
    ActivatePage(PageKey),
    ActivateNextPage,
    ActivatePreviousPage,
    ShowPopup(PopupKey),
    ClosePopup,
}
pub enum ActionOutcome {
    Consumed,
    NotHandled,
    #[allow(dead_code)]
    RequestFocus(ComponentKey),
    #[allow(dead_code)]
    Emit(/* domain events */),
}

impl LayerSystem {
    pub fn apply(&mut self, a: LayerAction) {
        match a {
            LayerAction::FocusNext => self.focus_next(),
            LayerAction::FocusPrev => self.focus_prev(),
            LayerAction::FocusComponent(key) => self.focus_component(key),
            LayerAction::ActivatePage(key) => self.activate_page(key),
            LayerAction::ActivateNextPage => {
                let _ = self.activate_next_page();
            }
            LayerAction::ActivatePreviousPage => {
                let _ = self.activate_previous_page();
            }
            LayerAction::ShowPopup(k) => self.show_popup(k),
            LayerAction::ClosePopup => self.close_popup(),
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

pub fn help_box(area: Rect) -> Rect {
    let w = area.width.saturating_mul(80) / 100;
    let h = area.height.saturating_mul(80) / 100;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.height.saturating_sub(h);
    Rect {
        x,
        y,
        width: w.min(area.width),
        height: h.min(area.height),
    }
}
