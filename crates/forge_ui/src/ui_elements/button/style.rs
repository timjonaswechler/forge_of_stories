// crates/forge_ui/src/ui_elements/button/style.rs

use super::enums::{ButtonSize, ButtonVariant};
use crate::theme::{self, UiTheme};
use bevy::prelude::*; // Import ColorManipulator

// --- NEU ---
#[derive(Debug, Clone)]
pub struct ButtonStyleDef {
    // Basis-Styling
    pub background_color: Color,
    pub text_color: Color,
    pub border_color: Color,

    // Overlay-Farben für Interaktion (werden über die Basis gelegt)
    // Enthält typischerweise Alpha für Transparenz. z.B. Color::WHITE.with_alpha(0.1)
    pub hover_bg_overlay: Color,
    pub pressed_bg_overlay: Color,
    pub hover_border_overlay: Color, // Kann Color::NONE sein, wenn Rand sich nicht ändert
    pub pressed_border_overlay: Color, // Kann Color::NONE sein, wenn Rand sich nicht ändert

    // Styling für deaktivierten Zustand
    pub disabled_bg_alpha: f32, // Wie transparent soll der BG im deaktivierten Zustand sein?
    pub disabled_border_alpha: f32, // Wie transparent soll der Rand sein?
    pub disabled_text_alpha: f32, // Wie transparent soll der Text sein?
}

// --- Implementierung für ButtonStyleDef ---
impl ButtonStyleDef {
    // Mischt die Basisfarbe mit einer Overlay-Farbe.
    // Simuliert das Drüberlegen der Overlay-Farbe über die Basisfarbe.
    fn apply_overlay(base: Color, overlay: Color) -> Color {
        if overlay.alpha() == 0.0 {
            return base; // Kein Overlay -> keine Änderung
        }

        // Mische die Farben mit dem Alpha-Wert des Overlays als Mischfaktor
        let result = base.mix(&overlay.with_alpha(1.0), overlay.alpha());

        // Setze den Alpha-Wert auf 1.0 (vollständig undurchsichtig)
        result.with_alpha(1.0)
    }

    // Berechnet die finale Hintergrundfarbe
    pub fn background(&self, interaction: Interaction, disabled: bool) -> BackgroundColor {
        if disabled {
            // Wende den Disabled-Alpha auf die Basisfarbe an
            return BackgroundColor(self.background_color.with_alpha(self.disabled_bg_alpha));
        }

        let base = self.background_color;
        let final_color = match interaction {
            Interaction::Hovered => Self::apply_overlay(base, self.hover_bg_overlay),
            Interaction::Pressed => Self::apply_overlay(base, self.pressed_bg_overlay),
            Interaction::None => base,
        };
        BackgroundColor(final_color)
    }

    // Berechnet die finale Randfarbe
    pub fn border(&self, interaction: Interaction, disabled: bool) -> BorderColor {
        if disabled {
            // Wende den Disabled-Alpha auf die Basis-Randfarbe an
            return BorderColor(self.border_color.with_alpha(self.disabled_border_alpha));
        }

        let base = self.border_color;
        // Wenn die Basis schon transparent ist (NONE), nicht versuchen zu überlagern
        if base == Color::NONE
            && self.hover_border_overlay == Color::NONE
            && self.pressed_border_overlay == Color::NONE
        {
            return BorderColor(base);
        }

        let final_color = match interaction {
            Interaction::Hovered => Self::apply_overlay(base, self.hover_border_overlay),
            Interaction::Pressed => Self::apply_overlay(base, self.pressed_border_overlay),
            Interaction::None => base,
        };
        BorderColor(final_color)
    }

    // Gibt die Textfarbe (ggf. für Disabled angepasst) zurück
    pub fn text_color(&self, disabled: bool) -> Color {
        if disabled {
            self.text_color.with_alpha(self.disabled_text_alpha)
        } else {
            self.text_color
        }
    }
}

// Holt die Styling-Definition basierend auf Variante und Theme
pub fn get_button_style_def(variant: ButtonVariant, theme: &UiTheme) -> ButtonStyleDef {
    // Definiere Standard Overlays und Disabled Alphas hier zur Wiederverwendung
    // Passe die Farben/Alphas nach Bedarf an!
    let hover_overlay = theme.color.white.interaction_primary; // z.B. Srgba(1.0, 1.0, 1.0, 0.1)
    let pressed_overlay = theme.color.black.interaction_secondary; // z.B. Srgba(0.0, 0.0, 0.0, 0.1)
    let default_disabled_bg_alpha = 0.5;
    let default_disabled_border_alpha = 0.5;
    let default_disabled_text_alpha = 0.5;

    match variant {
        ButtonVariant::Default => ButtonStyleDef {
            // --- Base Colors ---
            background_color: theme.color.gray.background_primary, // Standard gray background
            text_color: theme.color.gray.text_primary,             // Standard text color
            border_color: theme.color.gray.border_primary,         // Matching border

            // --- Interaction Overlays ---
            hover_bg_overlay: hover_overlay, // Standard hover effect
            pressed_bg_overlay: pressed_overlay, // Standard press effect
            hover_border_overlay: hover_overlay, // Border lightens/darkens with background
            pressed_border_overlay: pressed_overlay,

            // --- Disabled State ---
            disabled_bg_alpha: default_disabled_bg_alpha,
            disabled_border_alpha: default_disabled_border_alpha,
            disabled_text_alpha: default_disabled_text_alpha,
        },
        ButtonVariant::Destructive => ButtonStyleDef {
            background_color: theme.color.tomato.interaction_primary,
            text_color: theme.color.tomato.text_primary,
            border_color: theme.color.tomato.border_primary, // Ggf. expliziter Rand
            hover_bg_overlay: hover_overlay,
            pressed_bg_overlay: pressed_overlay,
            hover_border_overlay: hover_overlay,
            pressed_border_overlay: pressed_overlay,
            disabled_bg_alpha: default_disabled_bg_alpha,
            disabled_border_alpha: default_disabled_border_alpha,
            disabled_text_alpha: default_disabled_text_alpha,
        },
        ButtonVariant::Outline => ButtonStyleDef {
            // --- Base Colors ---
            background_color: Color::NONE, // Key feature: No background initially
            text_color: theme.color.gray.text_primary, // Standard text color
            border_color: theme.color.gray.border_secondary, // Visible border required

            // --- Interaction Overlays ---
            // Outline buttons often get a slight background on hover/press
            hover_bg_overlay: hover_overlay.with_alpha(0.05), // Very subtle background appears
            pressed_bg_overlay: hover_overlay.with_alpha(0.1), // Slightly stronger background
            hover_border_overlay: hover_overlay,              // Border can also react
            pressed_border_overlay: pressed_overlay,

            // --- Disabled State ---
            disabled_bg_alpha: 0.0, // Keep background transparent
            disabled_border_alpha: default_disabled_border_alpha, // Fade border
            disabled_text_alpha: default_disabled_text_alpha, // Fade text
        },
        ButtonVariant::Secondary => ButtonStyleDef {
            background_color: theme.color.gray.background_secondary,
            text_color: theme.color.gray.text_secondary,
            border_color: theme.color.gray.border_secondary,
            hover_bg_overlay: hover_overlay,
            pressed_bg_overlay: pressed_overlay,
            hover_border_overlay: hover_overlay,
            pressed_border_overlay: pressed_overlay,
            disabled_bg_alpha: default_disabled_bg_alpha,
            disabled_border_alpha: default_disabled_border_alpha,
            disabled_text_alpha: default_disabled_text_alpha,
        },
        ButtonVariant::Ghost => ButtonStyleDef {
            background_color: Color::NONE,
            text_color: theme.color.gray.text_primary,
            border_color: Color::NONE,
            hover_bg_overlay: hover_overlay, // Nur BG-Effekt
            pressed_bg_overlay: pressed_overlay,
            hover_border_overlay: Color::NONE, // Kein Rand-Effekt
            pressed_border_overlay: Color::NONE,
            disabled_bg_alpha: 0.0,
            disabled_border_alpha: 0.0,
            disabled_text_alpha: default_disabled_text_alpha,
        },
        ButtonVariant::Link => ButtonStyleDef {
            background_color: Color::NONE,
            text_color: theme.color.blue.text_primary, // Spezifische Link-Farbe
            border_color: Color::NONE,
            hover_bg_overlay: Color::NONE, // Kein BG-Overlay für Links
            pressed_bg_overlay: Color::NONE,
            hover_border_overlay: Color::NONE,
            pressed_border_overlay: Color::NONE,
            // TODO: Text-Unterstreichung bei Hover (dies geht nicht nur über Farbe)
            // Links oft leicht dunkler/heller bei Press
            // pressed_text_color: theme.color.blue.text_primary.darker(0.1), // Hypothetisch
            disabled_bg_alpha: 0.0,
            disabled_border_alpha: 0.0,
            disabled_text_alpha: default_disabled_text_alpha,
        },
    }
}

// --- base_style und apply_size_style bleiben unverändert ---
pub fn base_style() -> Node {
    Node {
        display: Display::Flex,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        column_gap: Val::Px(8.0),          // TODO: Aus Theme
        border: UiRect::all(Val::Px(1.0)), // Breite, Farbe kommt von BorderColor
        ..default()
    }
}

#[derive(Bundle, Clone)]
pub struct ButtonStyle {
    pub node: Node,
    pub border_radius: BorderRadius,
    pub transform: Transform,
    pub background_color: BackgroundColor,
    // … weitere Komponenten nach Bedarf
}

impl ButtonStyle {
    /// Erstellt einen ButtonStyle basierend auf der UiTheme-Resource
    pub fn from_theme(theme: &UiTheme) -> Self {
        // Standard-Radius aus Theme
        let default_radius = Val::Px(theme.layout.radius.base);
        ButtonStyle {
            node: Node {
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(theme.layout.padding.base), Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..Default::default()
            },
            border_radius: BorderRadius::all(default_radius),
            transform: Transform::default(),
            background_color: BackgroundColor(theme.color.gray.background_primary),
            // … Defaults für weitere Felder
        }
    }
}

// Default-Implementierung nutzt die Standard-UiTheme
impl Default for ButtonStyle {
    fn default() -> Self {
        // Nutzt die Default-Resource des Themes
        let theme = UiTheme::default();
        ButtonStyle::from_theme(&theme)
    }
}

// --- Function to apply size-specific styling ---
// Modifies Style and returns the font size
pub fn apply_size_style(style: &mut Node, size: ButtonSize, theme: &UiTheme) -> f32 {
    // Setze BorderRadius basierend auf Theme
    // TODO: Radius an Größe anpassen? Shadcn macht das oft nicht.
    // Wir verwenden einen festen Basisradius, aber spezifische Größen könnten dies überschreiben.
    // style.border_radius = BorderRadius::all(Val::Px(theme.layout.radius.base)); // Bevy 0.15+

    match size {
        ButtonSize::Default => {
            style.min_height = Val::Px(36.); // Feste Höhe (wie h-9)
            style.padding = UiRect::axes(Val::Px(theme.layout.padding.base), Val::Px(8.)); // ~px-4 py-2
            theme.font.font_size.base // text-sm
        }
        ButtonSize::Small => {
            style.min_height = Val::Px(32.); // Feste Höhe (wie h-8)
            style.padding = UiRect::axes(Val::Px(theme.layout.padding.sm), Val::Px(4.)); // ~px-3 py-1
            theme.font.font_size.sm // text-xs
        }
        ButtonSize::Large => {
            style.min_height = Val::Px(40.); // Feste Höhe (wie h-10)
            style.padding = UiRect::axes(Val::Px(theme.layout.padding.lg), Val::Px(8.)); // ~px-8 py-2
            theme.font.font_size.lg // text-base (oder lg?)
        }
        ButtonSize::Icon => {
            style.width = Val::Px(36.); // Feste Größe (wie h-9 w-9)
            style.height = Val::Px(36.);
            style.padding = UiRect::all(Val::Px(0.0));
            style.justify_content = JustifyContent::Center;
            style.align_items = AlignItems::Center;
            // Radius für Icon-Button oft 'full' oder der gleiche wie Default

            theme.font.font_size.base // Irrelevant für Icon, aber als Default
        }
    }
}
