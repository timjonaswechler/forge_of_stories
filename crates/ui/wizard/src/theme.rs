use std::env;

use ratatui::style::{Color, Modifier, Style};
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    TrueColor,
    Ansi256,
    Ansi16,
}

impl ColorMode {
    pub fn detect_auto() -> Self {
        // Override via env if provided
        if let Ok(v) = env::var("FOS_WIZARD_COLOR_MODE") {
            return match v.to_ascii_lowercase().as_str() {
                "24bit" | "truecolor" | "rgb" => ColorMode::TrueColor,
                "256" | "ansi256" => ColorMode::Ansi256,
                "16" | "ansi16" | "ansi" => ColorMode::Ansi16,
                _ => ColorMode::Auto,
            };
        }

        // Autodetect by common env vars
        if let Ok(v) = env::var("COLORTERM") {
            let l = v.to_ascii_lowercase();
            if l.contains("truecolor") || l.contains("24bit") {
                return ColorMode::TrueColor;
            }
        }
        if let Ok(v) = env::var("TERM") {
            let l = v.to_ascii_lowercase();
            if l.contains("256color") {
                return ColorMode::Ansi256;
            }
        }
        ColorMode::Ansi16
    }

    pub fn label(&self) -> &'static str {
        match self {
            ColorMode::Auto => "auto",
            ColorMode::TrueColor => "24-bit",
            ColorMode::Ansi256 => "256",
            ColorMode::Ansi16 => "16",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UiGroup {
    Border,
    Dimmed,
    Statusline,
    Title,
    ModeNormal,
    ModeInsert,
    ModeVisual,
    Success,
    Error,
    Warn,
    Info,
}

#[derive(Clone, Copy, Debug)]
pub struct Rgb(pub u8, pub u8, pub u8);

#[derive(Clone, Debug)]
pub struct Palette {
    pub fg: Rgb,
    pub dim: Rgb,
    pub border: Rgb,
    pub normal: Rgb,
    pub insert: Rgb,
    pub visual: Rgb,
    pub error: Rgb,
    pub warn: Rgb,
    pub info: Rgb,
}

impl Default for Palette {
    fn default() -> Self {
        // Subtiles dunkles Standard-Theme (nvim-ähnlich)
        Self {
            fg: Rgb(192, 202, 245),
            dim: Rgb(107, 112, 137),
            border: Rgb(59, 63, 81),
            normal: Rgb(125, 207, 255),
            insert: Rgb(158, 206, 106),
            visual: Rgb(187, 154, 247),
            error: Rgb(247, 118, 142),
            warn: Rgb(224, 175, 104),
            info: Rgb(122, 162, 247),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub mode: ColorMode,
    pub palette: Palette,
    pub flash_ms: u64,
}

impl Theme {
    pub fn from_env_auto() -> Self {
        let mode = ColorMode::detect_auto();
        Self { mode, palette: Palette::default(), flash_ms: 150 }
    }

    pub fn mode_label(&self) -> &'static str { self.mode.label() }

    pub fn style(&self, group: UiGroup) -> Style {
        match group {
            UiGroup::Border => Style::default().fg(self.color(self.palette.border)),
            UiGroup::Dimmed => Style::default().fg(self.color(self.palette.dim)),
            UiGroup::Statusline => Style::default().fg(self.color(self.palette.fg)),
            UiGroup::Title => Style::default().fg(self.color(self.palette.fg)).add_modifier(Modifier::BOLD),
            UiGroup::ModeNormal => Style::default().fg(self.color(self.palette.normal)).add_modifier(Modifier::BOLD),
            UiGroup::ModeInsert => Style::default().fg(self.color(self.palette.insert)).add_modifier(Modifier::BOLD),
            UiGroup::ModeVisual => Style::default().fg(self.color(self.palette.visual)).add_modifier(Modifier::BOLD),
            UiGroup::Success => Style::default().fg(self.color(self.palette.insert)).add_modifier(Modifier::BOLD),
            UiGroup::Error => Style::default().fg(self.color(self.palette.error)).add_modifier(Modifier::BOLD),
            UiGroup::Warn => Style::default().fg(self.color(self.palette.warn)).add_modifier(Modifier::BOLD),
            UiGroup::Info => Style::default().fg(self.color(self.palette.info)),
        }
    }

    fn color(&self, rgb: Rgb) -> Color {
        match self.mode {
            ColorMode::Auto => self.rgb_to_best(rgb),
            ColorMode::TrueColor => Color::Rgb(rgb.0, rgb.1, rgb.2),
            ColorMode::Ansi256 => Color::Indexed(rgb_to_ansi256(rgb.0, rgb.1, rgb.2)),
            ColorMode::Ansi16 => ansi16_from_rgb(rgb.0, rgb.1, rgb.2),
        }
    }

    fn rgb_to_best(&self, rgb: Rgb) -> Color {
        let detected = ColorMode::detect_auto();
        match detected {
            ColorMode::TrueColor => Color::Rgb(rgb.0, rgb.1, rgb.2),
            ColorMode::Ansi256 => Color::Indexed(rgb_to_ansi256(rgb.0, rgb.1, rgb.2)),
            ColorMode::Ansi16 | ColorMode::Auto => ansi16_from_rgb(rgb.0, rgb.1, rgb.2),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
}

impl Mode {
    pub fn next(self) -> Self {
        match self {
            Mode::Normal => Mode::Insert,
            Mode::Insert => Mode::Visual,
            Mode::Visual => Mode::Normal,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Visual => "VISUAL",
        }
    }
    pub fn style(&self, theme: &Theme) -> Style {
        match self {
            Mode::Normal => theme.style(UiGroup::ModeNormal),
            Mode::Insert => theme.style(UiGroup::ModeInsert),
            Mode::Visual => theme.style(UiGroup::ModeVisual),
        }
    }

    /// Statusline segment style with background colored by mode and readable foreground.
    pub fn status_segment_style(&self, theme: &Theme) -> Style {
        let bg = match self {
            Mode::Normal => theme.color(theme.palette.normal),
            Mode::Insert => theme.color(theme.palette.insert),
            Mode::Visual => theme.color(theme.palette.visual),
        };
        // Choose a contrasting foreground; for simplicity use Black for bright modes
        Style::default().bg(bg).fg(Color::Black).add_modifier(Modifier::BOLD)
    }
}

impl Theme {
    /// Chip style for small right-aligned badges in the statusline.
    pub fn chip_style(&self) -> Style {
        Style::default()
            .bg(self.color(self.palette.border))
            .fg(self.color(self.palette.fg))
    }

    pub fn chip_bg_color(&self) -> Color { self.color(self.palette.border) }
    pub fn chip_fg_color(&self) -> Color { self.color(self.palette.fg) }
    pub fn mode_bg_color(&self, m: Mode) -> Color {
        match m {
            Mode::Normal => self.color(self.palette.normal),
            Mode::Insert => self.color(self.palette.insert),
            Mode::Visual => self.color(self.palette.visual),
        }
    }

    pub fn supports_powerline(&self) -> bool {
        // opt-in via env; defaults to false for broad compatibility
        if let Ok(v) = env::var("FOS_WIZARD_POWERLINE") {
            let l = v.to_ascii_lowercase();
            return matches!(l.as_str(), "1"|"true"|"yes"|"on");
        }
        if let Ok(v) = env::var("FOS_NERD_FONT") {
            let l = v.to_ascii_lowercase();
            return matches!(l.as_str(), "1"|"true"|"yes"|"on");
        }
        if let Ok(v) = env::var("NERD_FONT") { // some shells export this
            let l = v.to_ascii_lowercase();
            return matches!(l.as_str(), "1"|"true"|"yes"|"on");
        }
        false
    }

    pub fn sep_left(&self) -> &'static str { if self.supports_powerline() { "" } else { ">" } }
    pub fn sep_right(&self) -> &'static str { if self.supports_powerline() { "" } else { "<" } }
}

fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    // Map RGB to xterm 256 color palette (6x6x6 cube + grayscale ramp)
    // Heuristic based on nearest cube; good enough for UI accents
    let r_idx = (r as f32 / 255.0 * 5.0).round() as u8;
    let g_idx = (g as f32 / 255.0 * 5.0).round() as u8;
    let b_idx = (b as f32 / 255.0 * 5.0).round() as u8;
    let color_idx = 16 + 36 * r_idx + 6 * g_idx + b_idx; // 16..231

    // Decide if grayscale is closer
    let avg = (r as u16 + g as u16 + b as u16) as f32 / 3.0;
    let gray_idx = (avg / 255.0 * 23.0).round() as u8; // 0..23
    let gray_color = 232 + gray_idx; // 232..255

    // Rough distance measure
    let cube_r = r_idx as f32 * 255.0 / 5.0;
    let cube_g = g_idx as f32 * 255.0 / 5.0;
    let cube_b = b_idx as f32 * 255.0 / 5.0;
    let dcube = (cube_r - r as f32).abs() + (cube_g - g as f32).abs() + (cube_b - b as f32).abs();
    let gval = gray_idx as f32 * 255.0 / 23.0;
    let dgray = (gval - r as f32).abs() + (gval - g as f32).abs() + (gval - b as f32).abs();
    if dgray + 15.0 < dcube { gray_color } else { color_idx }
}

fn ansi16_from_rgb(r: u8, g: u8, b: u8) -> Color {
    // Map to nearest of 16 ANSI colors; simple heuristic
    // Order: 0-7 normal, 8-15 bright
    // We'll choose among 8 bright for higher average brightness
    let avg = (r as u16 + g as u16 + b as u16) / 3;
    let bright = avg > 128;
    let (idx, _) = [
        (Color::Black, (0, 0, 0)),
        (Color::Red, (205, 0, 0)),
        (Color::Green, (0, 205, 0)),
        (Color::Yellow, (205, 205, 0)),
        (Color::Blue, (0, 0, 238)),
        (Color::Magenta, (205, 0, 205)),
        (Color::Cyan, (0, 205, 205)),
        (Color::Gray, (229, 229, 229)), // approximate white
    ]
    .into_iter()
    .map(|(c, (cr, cg, cb))| {
        let d = (cr as i32 - r as i32).abs()
            + (cg as i32 - g as i32).abs()
            + (cb as i32 - b as i32).abs();
        (c, d)
    })
    .min_by_key(|(_, d)| *d)
    .unwrap();
    match (idx, bright) {
        (Color::Black, false) => Color::Black,
        (Color::Red, false) => Color::Red,
        (Color::Green, false) => Color::Green,
        (Color::Yellow, false) => Color::Yellow,
        (Color::Blue, false) => Color::Blue,
        (Color::Magenta, false) => Color::Magenta,
        (Color::Cyan, false) => Color::Cyan,
        (Color::Gray, false) => Color::Gray,
        (Color::Black, true) => Color::DarkGray,
        (Color::Red, true) => Color::LightRed,
        (Color::Green, true) => Color::LightGreen,
        (Color::Yellow, true) => Color::LightYellow,
        (Color::Blue, true) => Color::LightBlue,
        (Color::Magenta, true) => Color::LightMagenta,
        (Color::Cyan, true) => Color::LightCyan,
        (Color::Gray, true) => Color::White,
        _ => Color::White,
    }
}
