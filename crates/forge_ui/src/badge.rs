// crates/forge_ui/src/badge.rs

use super::theme::UiTheme;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

// --- Komponenten ---

/// Marker-Komponente für Badges.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct BadgeMarker;

// Für Badges brauchen wir oft keine separate State-Komponente,
// es sei denn, sie sollen anklickbar oder anderweitig interaktiv sein.

// --- Varianten-Enum ---
// Definiert die verschiedenen visuellen Stile des Badges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BadgeVariant {
    #[default]
    Default, // Primärfarbe
    Secondary,
    Destructive,
    Outline,
}

// --- Builder ---

pub struct BadgeBuilder {
    text: String,
    variant: BadgeVariant,
    // Optional: Icons vor/nach dem Text (erfordert komplexeres Spawnen)
    // leading_icon: Option<Handle<Image>>,
    // trailing_icon: Option<Handle<Image>>,
}

impl BadgeBuilder {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::default(), // Startet mit Default-Variante
                                              // leading_icon: None,
                                              // trailing_icon: None,
        }
    }

    /// Setzt die visuelle Variante des Badges.
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    // Optional: Methoden für Icons
    // pub fn leading_icon(mut self, handle: Handle<Image>) -> Self { ... }
    // pub fn trailing_icon(mut self, handle: Handle<Image>) -> Self { ... }

    /// Spawnt das Badge als Kind des UI-Parents.
    #[must_use]
    pub fn spawn<'w, 'a>(
        self,
        parent: &'a mut ChildBuilder<'w>,
        theme: &UiTheme,
        font_handle: &Handle<Font>,
    ) -> Entity {
        // Styling ableiten
        let (bg_color, text_color, border_color) = get_badge_colors(&self.variant, theme);

        // Badge ist im Grunde ein Node mit einem Text-Kind
        parent
            .spawn((
                BadgeMarker,
                Node {
                    // Flexbox, um Text (und ggf. Icons) anzuordnen
                    display: Display::Flex,
                    align_items: AlignItems::Center, // Vertikal zentrieren
                    justify_content: JustifyContent::Center, // Horizontal zentrieren

                    // Padding & Border für den Look
                    // Shadcn: px-2.5 py-0.5 -> ca. 10px horizontal, 2px vertikal
                    padding: UiRect {
                        left: Val::Px(10.0),
                        right: Val::Px(10.0),
                        top: Val::Px(2.0),
                        bottom: Val::Px(2.0),
                    },
                    border: UiRect::all(Val::Px(1.0)), // Immer 1px Randbreite
                    // Volle Rundung ('rounded-full')
                    // Vollständig abrunden
                    // Sicherstellen, dass die Größe sich an den Inhalt anpasst
                    width: Val::Auto,
                    height: Val::Auto,
                    // Ggf. min_height für Konsistenz
                    min_height: Val::Px(18.0), // Höhe anpassen
                    ..default()
                },
                BorderRadius::percent(50.0, 50.0, 50.0, 50.0), // Vollständig abrunden
                BackgroundColor(bg_color),
                BorderColor(border_color),
            ))
            .with_children(|badge_node| {
                // Optional: Leading Icon hier spawnen

                // Text-Kind spawnen
                badge_node.spawn((
                    Text::new(self.text), // Text aus Builder
                    TextFont {
                        font: font_handle.clone(),
                        // Kleinere Schrift für Badges ('text-xs')
                        font_size: theme.font.font_size.base,
                        ..default()
                    },
                    TextColor(text_color), // Textfarbe
                ));

                // Optional: Trailing Icon hier spawnen
            })
            .id()
    }
}

// --- Helferfunktion für Farben ---
fn get_badge_colors(variant: &BadgeVariant, theme: &UiTheme) -> (Color, Color, Color) {
    // Gibt zurück: (Hintergrund, Text, Rand)
    match variant {
        BadgeVariant::Default => (
            theme.color.gray.background_primary, // Hintergrund: Primär
            theme.color.gray.text_primary,       // Text: Passend zu Primär
            theme.color.gray.border_primary,     // Rand: Gleich wie Hintergrund (wirkt transparent)
        ),
        BadgeVariant::Secondary => (
            theme.color.gray.background_secondary,
            theme.color.gray.text_secondary,
            theme.color.gray.border_secondary, // Rand: Gleich wie Hintergrund (wirkt transparent)
        ),
        BadgeVariant::Destructive => (
            theme.color.tomato.background_primary,
            theme.color.tomato.text_primary,
            theme.color.tomato.border_primary,
        ),
        BadgeVariant::Outline => (
            Color::NONE,                     // Kein Hintergrund
            theme.color.gray.text_primary,   // Text: Passend zu Primär
            theme.color.gray.border_primary, // Sichtbarer Rand
        ),
    }
}

// --- Systeme ---
// Aktuell benötigen Badges keine eigenen Update-Systeme,
// es sei denn, sie werden interaktiv (Hover-Effekte etc.).
