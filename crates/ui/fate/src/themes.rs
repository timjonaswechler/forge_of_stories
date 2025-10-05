use std::path::PathBuf;

use gpui::{App, SharedString};
use gpui_component::{ActiveTheme, Theme, ThemeRegistry, scroll::ScrollbarShow};
use serde::{Deserialize, Serialize};

const STATE_FILE: &str = "target/state.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct State {
    theme: SharedString,
    scrollbar_show: Option<ScrollbarShow>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            theme: "Ayu Light".into(),
            scrollbar_show: None,
        }
    }
}

pub fn init(cx: &mut App) {
    // Load last theme state
    let json = std::fs::read_to_string(STATE_FILE).unwrap_or(String::default());
    tracing::info!("Load themes...");
    let state = serde_json::from_str::<State>(&json).unwrap_or_default();
    if let Err(err) = ThemeRegistry::watch_dir(PathBuf::from("./themes"), cx, move |cx| {
        if let Some(theme) = ThemeRegistry::global(cx)
            .themes()
            .get(&state.theme)
            .cloned()
        {
            Theme::global_mut(cx).apply_config(&theme);
        }
    }) {
        tracing::error!("Failed to watch themes directory: {}", err);
    }

    if let Some(scrollbar_show) = state.scrollbar_show {
        Theme::global_mut(cx).scrollbar_show = scrollbar_show;
    }
    cx.refresh_windows();

    cx.observe_global::<Theme>(|cx| {
        let state = State {
            theme: cx.theme().theme_name().clone(),
            scrollbar_show: Some(cx.theme().scrollbar_show),
        };

        let json = serde_json::to_string_pretty(&state).unwrap();
        std::fs::write(STATE_FILE, json).unwrap();
    })
    .detach();
}
