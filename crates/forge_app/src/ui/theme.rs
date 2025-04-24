// crates/forge_app/src/ui/theme.rs
use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct UiTheme {
    // === Kernfarben ===
    /// Haupt-Hintergrundfarbe der Anwendung oder Bereiche.
    pub background: Color,
    /// Standard-Vordergrundfarbe (z.B. für Text auf `background`).
    pub foreground: Color,
    /// Hintergrundfarbe für Card-ähnliche Elemente.
    pub card: Color,
    /// Vordergrundfarbe (Text) auf Card-Elementen.
    pub card_foreground: Color,
    /// Hintergrundfarbe für Popover-Elemente (Tooltips, Menüs etc.).
    pub popover: Color,
    /// Vordergrundfarbe (Text) auf Popover-Elementen.
    pub popover_foreground: Color,

    // === Semantische Paletten ===
    /// Primäre Akzentfarbe (z.B. für Standard-Buttons, Links).
    pub primary: Color,
    /// Farbe für Text/Icons auf `primary`-Hintergrund.
    pub primary_foreground: Color,
    /// Sekundäre Akzentfarbe (z.B. für weniger wichtige Buttons, Hintergründe).
    pub secondary: Color,
    /// Farbe für Text/Icons auf `secondary`-Hintergrund.
    pub secondary_foreground: Color,
    /// Farbe für dezente Hintergründe oder inaktive Elemente.
    pub muted: Color,
    /// Farbe für Text/Icons auf `muted`-Hintergrund (oft grau).
    pub muted_foreground: Color,
    /// Farbe für Hover-Effekte oder spezielle Hervorhebungen.
    pub accent: Color,
    /// Farbe für Text/Icons auf `accent`-Hintergrund.
    pub accent_foreground: Color,
    /// Farbe für destruktive Aktionen (Löschen, Fehler).
    pub destructive: Color,
    /// Farbe für Text/Icons auf `destructive`-Hintergrund.
    pub destructive_foreground: Color,

    // === Ränder & Eingabe ===
    /// Standardfarbe für Ränder/Linien.
    pub border: Color,
    /// Farbe für Eingabefeld-Ränder oder -Hintergründe.
    pub input: Color,
    /// Farbe für den Fokus-Indikator (z.B. um Buttons/Inputs).
    pub ring: Color,

    // === Layout ===
    /// Standard-Eckenradius für Buttons, Cards etc.
    pub radius: Val, // Verwende Val für Flexibilität (Px, Percent, etc.)

    // === Schriftarten ===
    /// Handle zur Standard-Schriftart. Wird vom Asset-System gesetzt.
    pub default_font: Option<Handle<Font>>,
    // Optional: pub heading_font: Option<Handle<Font>>,
}

impl Default for UiTheme {
    fn default() -> Self {
        // Starten wir mit einem dunklen Theme, das sich an Shadcn's "Zinc" Palette orientiert.
        // Hinweis: Konvertierung von OKLCH zu sRGB ist nicht trivial. Dies sind Annäherungen.
        Self {
            // Dunkles Zink-ähnliches Theme
            background: Color::srgba_u8(12, 12, 12, 255), // Zinc 950
            foreground: Color::srgba_u8(250, 250, 250, 255), // Zinc 50
            card: Color::srgba_u8(39, 39, 39, 255),       // Zinc 900
            card_foreground: Color::srgba_u8(250, 250, 250, 255), // Zinc 50
            popover: Color::srgba_u8(39, 39, 39, 255),    // Zinc 900
            popover_foreground: Color::srgba_u8(250, 250, 250, 255), // Zinc 50

            primary: Color::srgba_u8(250, 250, 250, 255), // Zinc 50 (Hell für primäre Buttons)
            primary_foreground: Color::srgba_u8(24, 24, 27, 255), // Zinc 950 (Dunkler Text darauf)
            secondary: Color::srgba_u8(63, 63, 70, 255),  // Zinc 700
            secondary_foreground: Color::srgba_u8(250, 250, 250, 255), // Zinc 50
            muted: Color::srgba_u8(63, 63, 70, 255),      // Zinc 700
            muted_foreground: Color::srgba_u8(161, 161, 170, 255), // Zinc 400 (Gedämpfter Text)
            accent: Color::srgba_u8(63, 63, 70, 255),     // Zinc 700 (Kann oft wie secondary sein)
            accent_foreground: Color::srgba_u8(250, 250, 250, 255), // Zinc 50
            destructive: Color::srgba_u8(220, 38, 38, 255), // Red 600
            destructive_foreground: Color::srgba_u8(250, 250, 250, 255), // Zinc 50

            border: Color::srgba_u8(63, 63, 70, 255), // Zinc 700 / 800 -> Shadcn oft transparenter Rand (.dark)
            input: Color::srgba_u8(63, 63, 70, 255), // Zinc 700 / 800 -> Shadcn oft transparenter Rand (.dark)
            ring: Color::srgba_u8(161, 161, 170, 255), // Zinc 400 (oder Primary)

            radius: Val::Px(8.0), // Shadcn oft 0.5rem -> 8px

            default_font: None, // Wird später vom Asset Loader gesetzt
        }
    }
}
