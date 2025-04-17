// src/ui/theme.rs
use bevy::asset::{Asset, Handle};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;

#[derive(Debug, Clone)]
pub struct ButtonStyle {
    pub font_size: f32,
    pub padding: UiRect,
    pub border: UiRect,
    pub text_color: Color,
    pub normal_background: Color,
    pub hovered_background: Color,
    pub pressed_background: Color,
    pub normal_border: Color,
    pub hovered_border: Color,
    pub pressed_border: Color,
    // Optional: font handle, etc.
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            font_size: 24.0,
            padding: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(2.0)),
            text_color: Color::WHITE,
            normal_background: Color::srgb(0.15, 0.15, 0.15),
            hovered_background: Color::srgb(0.25, 0.25, 0.25),
            pressed_background: Color::srgb(0.10, 0.45, 0.10),
            normal_border: Color::BLACK,
            hovered_border: Color::WHITE,
            pressed_border: Color::srgb(0.20, 0.90, 0.20),
        }
    }
}

impl From<&ButtonStyleAsset> for ButtonStyle {
    fn from(asset: &ButtonStyleAsset) -> Self {
        Self {
            font_size: asset.font_size,
            padding: UiRect::all(Val::Px(asset.padding)),
            border: UiRect::all(Val::Px(asset.border)),
            text_color: Color::srgba(
                asset.text_color.0,
                asset.text_color.1,
                asset.text_color.2,
                asset.text_color.3,
            ),
            normal_background: Color::srgba(
                asset.normal_background.0,
                asset.normal_background.1,
                asset.normal_background.2,
                asset.normal_background.3,
            ),
            hovered_background: Color::srgba(
                asset.hovered_background.0,
                asset.hovered_background.1,
                asset.hovered_background.2,
                asset.hovered_background.3,
            ),
            pressed_background: Color::srgba(
                asset.pressed_background.0,
                asset.pressed_background.1,
                asset.pressed_background.2,
                asset.pressed_background.3,
            ),
            normal_border: Color::srgba(
                asset.normal_border.0,
                asset.normal_border.1,
                asset.normal_border.2,
                asset.normal_border.3,
            ),
            hovered_border: Color::srgba(
                asset.hovered_border.0,
                asset.hovered_border.1,
                asset.hovered_border.2,
                asset.hovered_border.3,
            ),
            pressed_border: Color::srgba(
                asset.pressed_border.0,
                asset.pressed_border.1,
                asset.pressed_border.2,
                asset.pressed_border.3,
            ),
        }
    }
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
pub struct ThemeAsset {
    pub button_style: ButtonStyleAsset,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ButtonStyleAsset {
    pub font_size: f32,
    pub padding: f32,
    pub border: f32,
    pub text_color: (f32, f32, f32, f32),
    pub normal_background: (f32, f32, f32, f32),
    pub hovered_background: (f32, f32, f32, f32),
    pub pressed_background: (f32, f32, f32, f32),
    pub normal_border: (f32, f32, f32, f32),
    pub hovered_border: (f32, f32, f32, f32),
    pub pressed_border: (f32, f32, f32, f32),
}

/// Ressource, die das allgemeine UI-Theme enthält.
#[derive(Resource, Debug)]
pub struct UiTheme {
    pub button_style: ButtonStyle,
    pub fonts: HashMap<String, Handle<Font>>,
    pub default_font: Option<Handle<Font>>,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            button_style: ButtonStyle::default(),
            fonts: HashMap::new(),
            default_font: None,
        }
    }
}

impl UiTheme {
    pub fn default_font(&self) -> &Handle<Font> {
        self.default_font
            .as_ref()
            .expect("Default-Font im Theme nicht gesetzt!")
    }
}

#[derive(Resource, Deref, Clone)]
pub struct ThemeAssetHandle(pub Handle<ThemeAsset>);

pub fn load_theme(asset_server: Res<AssetServer>, mut commands: Commands) {
    let theme_handle: Handle<ThemeAsset> = asset_server.load("theme/default.ron");
    commands.insert_resource(ThemeAssetHandle(theme_handle));
}

pub fn apply_theme_on_change(
    theme_handle: Res<ThemeAssetHandle>,
    theme_assets: Res<Assets<ThemeAsset>>,
    mut theme: ResMut<UiTheme>,
) {
    if let Some(theme_asset) = theme_assets.get(&**theme_handle) {
        info!("Theme loaded: {:?}", theme_asset);
        theme.button_style = ButtonStyle::from(&theme_asset.button_style);
    }
}
