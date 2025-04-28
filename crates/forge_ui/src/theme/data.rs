// crates/forge_ui/src/theme/data.rs

use bevy::{asset::Asset, reflect::TypePath};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
pub struct UiThemeData {
    pub ui_scaling: f32,
    pub font: UiTypographyData,
    pub layout: UiLayoutData,
    pub color: UiColorDatas,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiTypographyData {
    pub font_size: UiFontSizeData,
    pub font_family: UiFontFamiliesData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiFontSizeData {
    pub xs: f32,
    pub sm: f32,
    pub base: f32,
    pub lg: f32,
    pub xl: f32,
    pub x2l: f32,
    pub x3l: f32,
    pub x4l: f32,
    pub x5l: f32,
    pub x6l: f32,
    pub x7l: f32,
    pub x8l: f32,
    pub x9l: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiFontFamiliesData {
    pub default: String,
    pub sans: FontVariantsData,
    pub serif: FontVariantsData,
    pub mono: FontVariantsData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FontVariantsData {
    pub light: String,
    pub light_italic: String,
    pub regular: String,
    pub regular_italic: String,
    pub medium: String,
    pub medium_italic: String,
    pub bold: String,
    pub bold_italic: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiLayoutData {
    pub spacing: f32,
    pub padding: UiSpacingData,
    pub margin: UiSpacingData,
    pub gap: UiSpacingData,
    pub radius: UiRadiusData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiSpacingData {
    pub xs: f32,
    pub sm: f32,
    pub base: f32,
    pub lg: f32,
    pub xl: f32,
    pub x2l: f32,
    pub x3l: f32,
    pub x4l: f32,
    pub x5l: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiRadiusData {
    pub xs: f32,
    pub sm: f32,
    pub base: f32,
    pub lg: f32,
    pub xl: f32,
    pub x2l: f32,
    pub x3l: f32,
    pub x4l: f32,
    pub full: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiColorData {
    pub background_primary: [f32; 4],
    pub background_secondary: [f32; 4],
    pub interaction_primary: [f32; 4],
    pub interaction_secondary: [f32; 4],
    pub interaction_tertiary: [f32; 4],
    pub border_primary: [f32; 4],
    pub border_secondary: [f32; 4],
    pub border_tertiary: [f32; 4],
    pub solid_primary: [f32; 4],
    pub solid_secondary: [f32; 4],
    pub text_primary: [f32; 4],
    pub text_secondary: [f32; 4],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiColorDatas {
    pub white: UiColorData,
    pub black: UiColorData,
    pub gray: UiColorData,
    pub mauve: UiColorData,
    pub slate: UiColorData,
    pub sage: UiColorData,
    pub olive: UiColorData,
    pub sand: UiColorData,
    pub tomato: UiColorData,
    pub red: UiColorData,
    pub ruby: UiColorData,
    pub crimson: UiColorData,
    pub pink: UiColorData,
    pub plum: UiColorData,
    pub purple: UiColorData,
    pub violet: UiColorData,
    pub iris: UiColorData,
    pub indigo: UiColorData,
    pub blue: UiColorData,
    pub cyan: UiColorData,
    pub teal: UiColorData,
    pub jade: UiColorData,
    pub green: UiColorData,
    pub grass: UiColorData,
    pub bronze: UiColorData,
    pub gold: UiColorData,
    pub brown: UiColorData,
    pub orange: UiColorData,
    pub amber: UiColorData,
    pub yellow: UiColorData,
    pub lime: UiColorData,
    pub mint: UiColorData,
    pub sky: UiColorData,
}
