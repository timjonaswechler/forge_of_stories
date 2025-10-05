pub(crate) mod sidebar_fate;
mod title_bar;

pub use crate::assets::Assets;
use gpui::{
    Action, AnyElement, AnyView, App, AppContext, Bounds, Context, Div, Entity, EventEmitter,
    Focusable, Global, Hsla, InteractiveElement, IntoElement, KeyBinding, Menu, MenuItem,
    ParentElement, Pixels, Render, RenderOnce, SharedString, Size, StatefulInteractiveElement,
    Style, StyleRefinement, Styled, Window, WindowBounds, WindowKind, WindowOptions, actions, div,
    prelude::FluentBuilder as _, px, rems, size,
};

use serde::{Deserialize, Serialize};
pub use sidebar_fate::FateSidebar;
pub use title_bar::AppTitleBar;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

use gpui_component::{
    ActiveTheme, ContextModal, IconName, Root, TitleBar,
    button::Button,
    context_menu::ContextMenuExt,
    dock::{Panel, PanelControl, PanelEvent, PanelInfo, PanelState, TitleStyle, register_panel},
    group_box::GroupBox,
    h_flex,
    notification::Notification,
    popup_menu::PopupMenu,
    scroll::ScrollbarShow,
    v_flex,
};

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = story, no_json)]
pub struct SelectScrollbarShow(ScrollbarShow);

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = story, no_json)]
pub struct SelectLocale(SharedString);

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = story, no_json)]
pub struct SelectFont(usize);

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = story, no_json)]
pub struct SelectRadius(usize);

actions!(story, [Quit, Open, CloseWindow, ToggleSearch]);

const PANEL_NAME: &str = "FateContainer";

actions!(story, [TestAction, Tab, TabPrev]);

pub struct AppState {}
impl AppState {
    fn init(cx: &mut App) {
        let state = Self {};
        cx.set_global::<AppState>(state);
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }
}

pub fn create_new_window<F, E>(title: &str, crate_view_fn: F, cx: &mut App)
where
    E: Into<AnyView>,
    F: FnOnce(&mut Window, &mut App) -> E + Send + 'static,
{
    create_new_window_with_size(title, None, crate_view_fn, cx);
}

pub fn create_new_window_with_size<F, E>(
    title: &str,
    window_size: Option<Size<Pixels>>,
    crate_view_fn: F,
    cx: &mut App,
) where
    E: Into<AnyView>,
    F: FnOnce(&mut Window, &mut App) -> E + Send + 'static,
{
    let mut window_size = window_size.unwrap_or(size(px(1600.0), px(1200.0)));
    if let Some(display) = cx.primary_display() {
        let display_size = display.bounds().size;
        window_size.width = window_size.width.min(display_size.width * 0.85);
        window_size.height = window_size.height.min(display_size.height * 0.85);
    }
    let window_bounds = Bounds::centered(None, window_size, cx);
    let title = SharedString::from(title.to_string());

    cx.spawn(async move |cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(window_bounds)),
            titlebar: Some(TitleBar::title_bar_options()),
            window_min_size: Some(gpui::Size {
                width: px(480.),
                height: px(320.),
            }),
            kind: WindowKind::Normal,
            #[cfg(target_os = "linux")]
            window_background: gpui::WindowBackgroundAppearance::Transparent,
            #[cfg(target_os = "linux")]
            window_decorations: Some(gpui::WindowDecorations::Client),
            ..Default::default()
        };

        let window = cx
            .open_window(options, |window, cx| {
                let view = crate_view_fn(window, cx);
                let root = cx.new(|cx| FateRoot::new(title.clone(), view, window, cx));

                cx.new(|cx| Root::new(root.into(), window, cx))
            })
            .expect("failed to open window");

        window
            .update(cx, |_, window, _| {
                window.activate_window();
                window.set_window_title(&title);
            })
            .expect("failed to update window");

        Ok::<_, anyhow::Error>(())
    })
    .detach();
}

struct FateRoot {
    title_bar: Entity<AppTitleBar>,
    view: AnyView,
}

impl FateRoot {
    pub fn new(
        title: impl Into<SharedString>,
        view: impl Into<AnyView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let title_bar = cx.new(|cx| AppTitleBar::new(title, window, cx));
        Self {
            title_bar,
            view: view.into(),
        }
    }
}

impl Render for FateRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let drawer_layer = Root::render_drawer_layer(window, cx);
        let modal_layer = Root::render_modal_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        div()
            .size_full()
            .child(
                v_flex()
                    .size_full()
                    .child(self.title_bar.clone())
                    .child(div().flex_1().overflow_hidden().child(self.view.clone())),
            )
            .children(drawer_layer)
            .children(modal_layer)
            .children(notification_layer)
    }
}

impl Global for AppState {}

pub fn init(cx: &mut App) {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("gpui_component=trace".parse().unwrap()),
        )
        .init();

    gpui_component::init(cx);
    AppState::init(cx);
    crate::themes::init(cx);

    cx.bind_keys([
        KeyBinding::new("/", ToggleSearch, None),
        KeyBinding::new("cmd-q", Quit, None),
    ]);

    cx.on_action(|_: &Quit, cx: &mut App| {
        cx.quit();
    });
    cx.activate(true);
}

actions!(story, [ShowPanelInfo]);

#[derive(IntoElement)]
struct FateSection {
    base: Div,
    title: SharedString,
    sub_title: Vec<AnyElement>,
    children: Vec<AnyElement>,
}

impl FateSection {
    pub fn sub_title(mut self, sub_title: impl IntoElement) -> Self {
        self.sub_title.push(sub_title.into_any_element());
        self
    }

    #[allow(unused)]
    fn max_w_md(mut self) -> Self {
        self.base = self.base.max_w(rems(48.));
        self
    }

    #[allow(unused)]
    fn max_w_lg(mut self) -> Self {
        self.base = self.base.max_w(rems(64.));
        self
    }

    #[allow(unused)]
    fn max_w_xl(mut self) -> Self {
        self.base = self.base.max_w(rems(80.));
        self
    }

    #[allow(unused)]
    fn max_w_2xl(mut self) -> Self {
        self.base = self.base.max_w(rems(96.));
        self
    }
}

impl ParentElement for FateSection {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for FateSection {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for FateSection {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        GroupBox::new()
            .outline()
            .title(
                h_flex()
                    .justify_between()
                    .w_full()
                    .gap_4()
                    .child(self.title)
                    .children(self.sub_title),
            )
            .content_style(
                StyleRefinement::default()
                    .rounded_lg()
                    .overflow_x_hidden()
                    .items_center()
                    .justify_center(),
            )
            .child(self.base.children(self.children))
    }
}

impl ContextMenuExt for FateSection {}

pub(crate) fn section(title: impl Into<SharedString>) -> FateSection {
    FateSection {
        title: title.into(),
        sub_title: vec![],
        base: h_flex()
            .flex_wrap()
            .justify_center()
            .items_center()
            .w_full()
            .gap_4(),
        children: vec![],
    }
}

pub struct FateContainer {
    focus_handle: gpui::FocusHandle,
    pub name: SharedString,
    pub title_bg: Option<Hsla>,
    pub description: SharedString,
    width: Option<gpui::Pixels>,
    height: Option<gpui::Pixels>,
    story: Option<AnyView>,
    story_klass: Option<SharedString>,
    closable: bool,
    zoomable: Option<PanelControl>,
    on_active: Option<fn(AnyView, bool, &mut Window, &mut App)>,
}

#[derive(Debug)]
pub enum ContainerEvent {
    Close,
}

pub trait Fate: Render + Sized {
    fn klass() -> &'static str {
        std::any::type_name::<Self>().split("::").last().unwrap()
    }

    fn title() -> &'static str;
    fn description() -> &'static str {
        ""
    }
    fn closable() -> bool {
        true
    }
    fn zoomable() -> Option<PanelControl> {
        Some(PanelControl::default())
    }
    fn title_bg() -> Option<Hsla> {
        None
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render>;

    fn on_active(&mut self, active: bool, window: &mut Window, cx: &mut App) {
        let _ = active;
        let _ = window;
        let _ = cx;
    }
    fn on_active_any(view: AnyView, active: bool, window: &mut Window, cx: &mut App)
    where
        Self: 'static,
    {
        if let Some(story) = view.downcast::<Self>().ok() {
            cx.update_entity(&story, |story, cx| {
                story.on_active(active, window, cx);
            });
        }
    }
}

impl EventEmitter<ContainerEvent> for FateContainer {}

impl FateContainer {
    pub fn new(_window: &mut Window, cx: &mut App) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            focus_handle,
            name: "".into(),
            title_bg: None,
            description: "".into(),
            width: None,
            height: None,
            story: None,
            story_klass: None,
            closable: true,
            zoomable: Some(PanelControl::default()),
            on_active: None,
        }
    }

    pub fn panel<S: Fate>(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let name = S::title();
        let description = S::description();
        let story = S::new_view(window, cx);
        let story_klass = S::klass();

        let view = cx.new(|cx| {
            let mut story = Self::new(window, cx)
                .story(story.into(), story_klass)
                .on_active(S::on_active_any);
            story.focus_handle = cx.focus_handle();
            story.closable = S::closable();
            story.zoomable = S::zoomable();
            story.name = name.into();
            story.description = description.into();
            story.title_bg = S::title_bg();
            story
        });

        view
    }

    pub fn width(mut self, width: gpui::Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: gpui::Pixels) -> Self {
        self.height = Some(height);
        self
    }

    pub fn story(mut self, story: AnyView, story_klass: impl Into<SharedString>) -> Self {
        self.story = Some(story);
        self.story_klass = Some(story_klass.into());
        self
    }

    pub fn on_active(mut self, on_active: fn(AnyView, bool, &mut Window, &mut App)) -> Self {
        self.on_active = Some(on_active);
        self
    }

    fn on_action_panel_info(
        &mut self,
        _: &ShowPanelInfo,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        struct Info;
        let note = Notification::new()
            .message(format!("You have clicked panel info on: {}", self.name))
            .id::<Info>();
        window.push_notification(note, cx);
    }

    fn on_action_toggle_search(
        &mut self,
        _: &ToggleSearch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.propagate();
        if window.has_focused_input(cx) {
            return;
        }

        struct Search;
        let note = Notification::new()
            .message(format!("You have toggled search on: {}", self.name))
            .id::<Search>();
        window.push_notification(note, cx);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FateState {
    pub story_klass: SharedString,
}

impl FateState {
    fn to_value(&self) -> serde_json::Value {
        serde_json::json!({
            "story_klass": self.story_klass,
        })
    }

    fn from_value(value: serde_json::Value) -> Self {
        serde_json::from_value(value).unwrap()
    }
}

impl Panel for FateContainer {
    fn panel_name(&self) -> &'static str {
        "StoryContainer"
    }

    fn title(&self, _window: &Window, _cx: &App) -> AnyElement {
        self.name.clone().into_any_element()
    }

    fn title_style(&self, cx: &App) -> Option<TitleStyle> {
        if let Some(bg) = self.title_bg {
            Some(TitleStyle {
                background: bg,
                foreground: cx.theme().foreground,
            })
        } else {
            None
        }
    }

    fn closable(&self, _cx: &App) -> bool {
        self.closable
    }

    fn zoomable(&self, _cx: &App) -> Option<PanelControl> {
        self.zoomable
    }

    fn visible(&self, cx: &App) -> bool {
        true
    }

    fn set_zoomed(&mut self, zoomed: bool, _window: &mut Window, _cx: &mut App) {
        println!("panel: {} zoomed: {}", self.name, zoomed);
    }

    fn set_active(&mut self, active: bool, _window: &mut Window, cx: &mut App) {
        println!("panel: {} active: {}", self.name, active);
        if let Some(on_active) = self.on_active {
            if let Some(story) = self.story.clone() {
                on_active(story, active, _window, cx);
            }
        }
    }

    fn popup_menu(&self, menu: PopupMenu, _window: &Window, _cx: &App) -> PopupMenu {
        menu.menu("Info", Box::new(ShowPanelInfo))
    }

    fn toolbar_buttons(&self, _window: &mut Window, _cx: &mut App) -> Option<Vec<Button>> {
        Some(vec![
            Button::new("info")
                .icon(IconName::Info)
                .on_click(|_, window, cx| {
                    window.push_notification("You have clicked info button", cx);
                }),
            Button::new("search")
                .icon(IconName::Search)
                .on_click(|_, window, cx| {
                    window.push_notification("You have clicked search button", cx);
                }),
        ])
    }

    fn dump(&self, _cx: &App) -> PanelState {
        let mut state = PanelState::new(self);
        let story_state = FateState {
            story_klass: self.story_klass.clone().unwrap(),
        };
        state.info = PanelInfo::panel(story_state.to_value());
        state
    }
}

impl EventEmitter<PanelEvent> for FateContainer {}
impl Focusable for FateContainer {
    fn focus_handle(&self, _: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}
impl Render for FateContainer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("story-container")
            .size_full()
            .overflow_y_scroll()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_action_panel_info))
            .on_action(cx.listener(Self::on_action_toggle_search))
            .when_some(self.story.clone(), |this, story| {
                this.child(
                    v_flex()
                        .id("story-children")
                        .w_full()
                        .flex_1()
                        .p_4()
                        .child(story),
                )
            })
    }
}
