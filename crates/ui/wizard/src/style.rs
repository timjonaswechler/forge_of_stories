#![allow(dead_code)]

/*!
Theme and terminal palette override system for the Wizard TUI.

Goals:
- Provide semantic color roles you can use across pages/components.
- Optionally override the terminal palette (ANSI 0–15, default fg/bg, cursor)
  using OSC sequences on compatible terminals.
- Ensure we reset the palette on exit via a guard (RAII), so the user's terminal
  is not left in a modified state.

Notes:
- Palette overrides require a terminal that supports ANSI and xterm-style OSC
  sequences (kitty, iTerm2, Alacritty, most xterm derivatives).
- On unsupported terminals, the guard becomes a no-op and only semantic Style/Color
  values are available for use with ratatui widgets.
- Reset sequences used:
  - OSC 104;idx BEL  -> reset palette color (idx 0..15)
  - OSC 110 BEL      -> reset default foreground
  - OSC 111 BEL      -> reset default background
  - OSC 112 BEL      -> reset cursor color
*/

// text: rgb(0xffffff),
// selected_text: rgb(0xffffff),
// disabled: rgb(0x565656),
// selected: rgb(0x2457ca),
// background: rgb(0x222222),
// border: rgb(0x000000),
// separator: rgb(0xd9d9d9),
// container: rgb(0x262626),
use std::io::{Write, stdout};

use ratatui::style::{Color, Style};

/// Semantic roles used by widgets and pages to request colors independent of a specific theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Role {
    Background,
    Surface,
    Text,
    SubtleText,
    InvertedText,
    Selection,

    Primary,
    Accent,
    Success,
    Warning,
    Danger,
    Info,
    Muted,
}

/// A mapping from semantic roles to colors for a given Theme.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RoleColors {
    pub background: Color,
    pub surface: Color,
    pub text: Color,
    pub subtle_text: Color,
    pub inverted_text: Color,
    pub selection: Color,

    pub primary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub info: Color,
    pub muted: Color,
}

impl RoleColors {
    pub fn color(&self, role: Role) -> Color {
        match role {
            Role::Background => self.background,
            Role::Surface => self.surface,
            Role::Text => self.text,
            Role::SubtleText => self.subtle_text,
            Role::InvertedText => self.inverted_text,
            Role::Selection => self.selection,

            Role::Primary => self.primary,
            Role::Accent => self.accent,
            Role::Success => self.success,
            Role::Warning => self.warning,
            Role::Danger => self.danger,
            Role::Info => self.info,
            Role::Muted => self.muted,
        }
    }
}

/// Optional terminal palette override specification.
/// - `fg`, `bg`, `cursor`: default colors
/// - `ansi`: override for indices 0..=15 (standard + bright ANSI colors)
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PaletteSpec {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub cursor: Option<Color>,
    /// ANSI palette indices 0..15, where None means "do not override this index".
    #[serde(default)]
    pub ansi: [Option<Color>; 16],
}

/// A full theme containing semantic role colors and an optional terminal palette override.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Theme {
    pub name: String,
    pub roles: RoleColors,
    pub palette: Option<PaletteSpec>,
}

impl Theme {
    /// Convenience method to turn a role into a ratatui `Style`.
    pub fn style(&self, role: Role) -> Style {
        Style::default().fg(self.roles.color(role))
    }

    /// Same as `style`, but allows custom background.
    pub fn style_on(&self, role: Role, on: Role) -> Style {
        Style::default()
            .fg(self.roles.color(role))
            .bg(self.roles.color(on))
    }
}

/// Apply the theme's palette override (if any) to the terminal, returning a guard that will
/// reset the palette on Drop. If ANSI/OSC is not supported, returns a no-op guard.
///
/// Typical usage at app startup:
/// let _palette_guard = apply_theme_palette(&my_theme);
///
/// Keep the guard alive for the lifetime of your TUI. It will reset the terminal when dropped.
pub fn apply_theme_palette(theme: &Theme) -> TerminalPaletteGuard {
    // if !ansi_supported() {
    // return TerminalPaletteGuard::noop();
    // }

    let Some(palette) = &theme.palette else {
        return TerminalPaletteGuard::noop();
    };

    // We track what we actually set so we can selectively reset later.
    let mut guard = TerminalPaletteGuard {
        supports_ansi: true,
        set_fg: false,
        set_bg: false,
        set_cursor: false,
        indices_set: Vec::new(),
    };

    if let Some(fg) = palette.fg {
        if let Some((r, g, b)) = color_to_rgb(fg) {
            send_osc(&osc_set_default_fg(r, g, b));
            guard.set_fg = true;
        }
    }
    if let Some(bg) = palette.bg {
        if let Some((r, g, b)) = color_to_rgb(bg) {
            send_osc(&osc_set_default_bg(r, g, b));
            guard.set_bg = true;
        }
    }
    if let Some(cur) = palette.cursor {
        if let Some((r, g, b)) = color_to_rgb(cur) {
            send_osc(&osc_set_cursor(r, g, b));
            guard.set_cursor = true;
        }
    }

    for (idx, maybe_color) in palette.ansi.iter().enumerate() {
        if let Some(color) = maybe_color {
            if let Some((r, g, b)) = color_to_rgb(*color) {
                send_osc(&osc_set_palette_index(idx as u8, r, g, b));
                guard.indices_set.push(idx as u8);
            }
        }
    }

    // Flush once after sending all sequences
    let _ = stdout().flush();

    guard
}

/// RAII guard that resets the terminal palette on drop.
pub struct TerminalPaletteGuard {
    supports_ansi: bool,
    set_fg: bool,
    set_bg: bool,
    set_cursor: bool,
    indices_set: Vec<u8>,
}

impl TerminalPaletteGuard {
    fn noop() -> Self {
        Self {
            supports_ansi: false,
            set_fg: false,
            set_bg: false,
            set_cursor: false,
            indices_set: Vec::new(),
        }
    }
}

impl Drop for TerminalPaletteGuard {
    fn drop(&mut self) {
        if !self.supports_ansi {
            return;
        }
        // Reset only what we set.
        for idx in self.indices_set.drain(..) {
            send_osc(&osc_reset_palette_index(idx));
        }
        if self.set_fg {
            send_osc(&osc_reset_default_fg());
        }
        if self.set_bg {
            send_osc(&osc_reset_default_bg());
        }
        if self.set_cursor {
            send_osc(&osc_reset_cursor());
        }
        let _ = stdout().flush();
    }
}

/// Returns a default dark theme with a warm accent, including an ANSI palette override
/// tuned for good contrast in TUI environments.
pub fn default_dark_theme() -> Theme {
    let roles = RoleColors {
        background: Color::Rgb(30, 15, 0),
        surface: Color::Rgb(28, 28, 34),
        text: Color::Rgb(220, 220, 220),
        subtle_text: Color::Rgb(130, 130, 130),
        inverted_text: Color::Rgb(0, 0, 0),
        selection: Color::Rgb(58, 91, 156),

        primary: Color::Rgb(255, 154, 79), // warm orange
        accent: Color::Rgb(99, 205, 218),  // teal-cyan
        success: Color::Rgb(102, 187, 106),
        warning: Color::Rgb(255, 214, 102),
        danger: Color::Rgb(239, 83, 80),
        info: Color::Rgb(144, 202, 249),
        muted: Color::Rgb(120, 120, 128),
    };

    // ANSI mapping:
    // 0..7  = normal  black, red, green, yellow, blue, magenta, cyan, white
    // 8..15 = bright  black, red, green, yellow, blue, magenta, cyan, white
    let ansi = [
        Some(Color::Rgb(0, 0, 0)),       // 0 black
        Some(Color::Rgb(204, 102, 102)), // 1 red
        Some(Color::Rgb(152, 195, 121)), // 2 green
        Some(Color::Rgb(224, 175, 104)), // 3 yellow
        Some(Color::Rgb(97, 175, 239)),  // 4 blue
        Some(Color::Rgb(198, 120, 221)), // 5 magenta
        Some(Color::Rgb(86, 182, 194)),  // 6 cyan
        Some(Color::Rgb(220, 223, 228)), // 7 white (light gray)
        Some(Color::Rgb(92, 99, 112)),   // 8 bright black (dim gray)
        Some(Color::Rgb(224, 108, 117)), // 9 bright red
        Some(Color::Rgb(152, 195, 121)), // 10 bright green (same)
        Some(Color::Rgb(229, 192, 123)), // 11 bright yellow
        Some(Color::Rgb(97, 175, 239)),  // 12 bright blue (same)
        Some(Color::Rgb(198, 120, 221)), // 13 bright magenta (same)
        Some(Color::Rgb(86, 182, 194)),  // 14 bright cyan (same)
        Some(Color::Rgb(236, 239, 244)), // 15 bright white
    ];

    let palette = PaletteSpec {
        fg: Some(roles.text),
        bg: Some(roles.background),
        cursor: Some(Color::Rgb(255, 154, 79)),
        ansi,
    };

    Theme {
        name: "Default Dark".to_string(),
        roles,
        palette: Some(palette),
    }
}

/// A higher-contrast theme useful for demos or low-quality projectors.
pub fn high_contrast_theme() -> Theme {
    let roles = RoleColors {
        background: Color::Rgb(0, 0, 0),
        surface: Color::Rgb(15, 15, 15),
        text: Color::Rgb(250, 250, 250),
        subtle_text: Color::Rgb(200, 200, 200),
        inverted_text: Color::Rgb(0, 0, 0),
        selection: Color::Rgb(70, 70, 255),

        primary: Color::Rgb(255, 200, 0),
        accent: Color::Rgb(0, 220, 255),
        success: Color::Rgb(0, 255, 100),
        warning: Color::Rgb(255, 180, 0),
        danger: Color::Rgb(255, 70, 70),
        info: Color::Rgb(130, 180, 255),
        muted: Color::Rgb(140, 140, 140),
    };

    let ansi = [
        Some(Color::Rgb(0, 0, 0)),
        Some(Color::Rgb(255, 0, 0)),
        Some(Color::Rgb(0, 255, 0)),
        Some(Color::Rgb(255, 255, 0)),
        Some(Color::Rgb(0, 128, 255)),
        Some(Color::Rgb(255, 0, 255)),
        Some(Color::Rgb(0, 255, 255)),
        Some(Color::Rgb(255, 255, 255)),
        Some(Color::Rgb(80, 80, 80)),
        Some(Color::Rgb(255, 100, 100)),
        Some(Color::Rgb(100, 255, 120)),
        Some(Color::Rgb(255, 255, 140)),
        Some(Color::Rgb(120, 180, 255)),
        Some(Color::Rgb(255, 120, 255)),
        Some(Color::Rgb(140, 255, 255)),
        Some(Color::Rgb(255, 255, 255)),
    ];

    let palette = PaletteSpec {
        fg: Some(roles.text),
        bg: Some(roles.background),
        cursor: Some(Color::Rgb(255, 200, 0)),
        ansi,
    };

    Theme {
        name: "High Contrast".to_string(),
        roles,
        palette: Some(palette),
    }
}

/// Convert a ratatui `Color` to an RGB triple. Only Rgb and named/ANSI colors are supported.
/// Indexed colors are not mapped (return None) because we cannot reliably know the palette.
fn color_to_rgb(color: Color) -> Option<(u8, u8, u8)> {
    match color {
        Color::Reset => None,
        Color::Black => Some((0x00, 0x00, 0x00)),
        Color::Red => Some((0xCD, 0x00, 0x00)),
        Color::Green => Some((0x00, 0xCD, 0x00)),
        Color::Yellow => Some((0xCD, 0xCD, 0x00)),
        Color::Blue => Some((0x00, 0x00, 0xEE)),
        Color::Magenta => Some((0xCD, 0x00, 0xCD)),
        Color::Cyan => Some((0x00, 0xCD, 0xCD)),
        Color::Gray => Some((0xE5, 0xE5, 0xE5)),

        Color::DarkGray => Some((0x7F, 0x7F, 0x7F)),
        Color::LightRed => Some((0xFF, 0x6B, 0x6B)),
        Color::LightGreen => Some((0x98, 0xFB, 0x98)),
        Color::LightYellow => Some((0xFF, 0xFF, 0xA0)),
        Color::LightBlue => Some((0xAD, 0xD8, 0xE6)),
        Color::LightMagenta => Some((0xFF, 0xAF, 0xFF)),
        Color::LightCyan => Some((0xE0, 0xFF, 0xFF)),
        Color::White => Some((0xFF, 0xFF, 0xFF)),

        Color::Rgb(r, g, b) => Some((r, g, b)),
        // We do not expand 256-color indexed codes because we don't know the target palette mapping here.
        // Callers that want specific colors for OSC should use Color::Rgb or standard named colors.
        Color::Indexed(_) => None,
    }
}

/// Whether ANSI/OSC sequences are supported by the current output.
///
/// crossterm already performs reasonable checks; on Windows terminals that don't support ANSI,
/// this will return false and we won't try to override the palette.
// fn ansi_supported() -> bool {
//     crossterm::supports_ansi()
// }

/// Send a single OSC sequence to stdout.
fn send_osc(s: &str) {
    let _ = write!(stdout(), "{s}");
}

/// Build OSC string to set palette index color: OSC 4;idx;rgb:RR/GG/BB BEL
fn osc_set_palette_index(idx: u8, r: u8, g: u8, b: u8) -> String {
    format!("\x1b]4;{};rgb:{:02x}/{:02x}/{:02x}\x07", idx, r, g, b)
}

/// Build OSC string to reset palette index idx to default: OSC 104;idx BEL
fn osc_reset_palette_index(idx: u8) -> String {
    format!("\x1b]104;{}\x07", idx)
}

/// Build OSC string to set default foreground: OSC 10;rgb:RR/GG/BB BEL
fn osc_set_default_fg(r: u8, g: u8, b: u8) -> String {
    format!("\x1b]10;rgb:{:02x}/{:02x}/{:02x}\x07", r, g, b)
}

/// Build OSC string to set default background: OSC 11;rgb:RR/GG/BB BEL
fn osc_set_default_bg(r: u8, g: u8, b: u8) -> String {
    format!("\x1b]11;rgb:{:02x}/{:02x}/{:02x}\x07", r, g, b)
}

/// Build OSC string to set cursor color: OSC 12;rgb:RR/GG/BB BEL
fn osc_set_cursor(r: u8, g: u8, b: u8) -> String {
    format!("\x1b]12;rgb:{:02x}/{:02x}/{:02x}\x07", r, g, b)
}

/// Reset default foreground to terminal's default: OSC 110 BEL
fn osc_reset_default_fg() -> String {
    "\x1b]110\x07".to_string()
}

/// Reset default background to terminal's default: OSC 111 BEL
fn osc_reset_default_bg() -> String {
    "\x1b]111\x07".to_string()
}

/// Reset cursor color to terminal's default: OSC 112 BEL
fn osc_reset_cursor() -> String {
    "\x1b]112\x07".to_string()
}

/// Helper: produce a Style for a given role, with optional modifiers.
/// Example: style_role(&theme, Role::Primary).bg(theme.roles.surface)
pub fn style_role(theme: &Theme, role: Role) -> Style {
    theme.style(role)
}

// Example integration hints (not code-invoked here):
//
// - Create and apply a theme when TUI starts:
//     let theme = default_dark_theme();
//     let _palette_guard = apply_theme_palette(&theme);
//     // keep `_palette_guard` alive until TUI exit
//
// - Use roles in components:
//     let title_style = theme.style(Role::Primary).bold();
//     let border_style = Style::default().fg(theme.roles.muted);
//
// - Switch themes at runtime:
//     drop(old_guard); // resets old palette
//     let _palette_guard = apply_theme_palette(&new_theme);
