// crates/forge_ui/src/label.rs

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*; // Für den Rückgabetyp von spawn

use super::theme::UiTheme; // Theme für Styling

// --- Marker Komponente ---
/// Marker-Komponente für UI-Labels.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct LabelMarker;

// --- Styling-Optionen (Optional, oft direkt vom Theme gesteuert) ---
// Ähnlich wie bei Card-Elementen, könnten wir Stile definieren,
// aber für einfache Labels reicht oft die Theme-Farbe.
// #[derive(Clone, Copy, Default, PartialEq, Eq)]
// pub enum LabelStyle {
//     #[default] Normal,
//     Muted,
//     Error, // Beispiel für semantische Farben
// }

// --- Label Builder ---

pub struct LabelBuilder {
    text: String,
    // Optional: Spezifische Farbe überschreiben?
    color: Option<Color>,
    // Optional: Spezifische Schriftgröße überschreiben?
    font_size: Option<f32>,
    // Optional: Styling-Variante (falls LabelStyle Enum verwendet wird)
    // label_style: LabelStyle,
    // Optional: Textausrichtung
    alignment: JustifyText,
}

impl LabelBuilder {
    /// Erstellt einen neuen LabelBuilder mit dem gegebenen Textinhalt.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: None,
            font_size: None,
            // label_style: LabelStyle::default(),
            alignment: JustifyText::Left, // Standardmäßig linksbündig
        }
    }

    /// Überschreibt die Textfarbe, die sonst vom Theme kommt.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Überschreibt die Schriftgröße, die sonst Standard (z.B. 14.0) ist.
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    /// Setzt die Textausrichtung (Links, Zentriert, Rechts).
    pub fn align(mut self, alignment: JustifyText) -> Self {
        self.alignment = alignment;
        self
    }

    // Optional: Methode für Styling-Enum
    // pub fn style(mut self, style: LabelStyle) -> Self {
    //     self.label_style = style;
    //     self
    // }

    /// Spawnt das Label als Kind des gegebenen UI-Parents.
    #[must_use = "Commands should generally be used, e.g. to get the entity ID"]
    pub fn spawn<'w, 'a>(
        self,
        parent: &'a mut ChildBuilder<'w>,
        theme: &UiTheme,
        font_handle: &Handle<Font>,
    ) -> Entity {
        // Bestimme Farbe und Größe basierend auf Builder-Optionen und Theme
        let final_color = self.color.unwrap_or(theme.foreground); // Standard: Normale Vordergrundfarbe
        let final_font_size = self.font_size.unwrap_or(14.0); // Standardgröße für Labels

        // Optional: Farbe basierend auf LabelStyle anpassen
        // let final_color = match self.label_style {
        //     LabelStyle::Normal => self.color.unwrap_or(theme.foreground),
        //     LabelStyle::Muted => self.color.unwrap_or(theme.muted_foreground),
        //     LabelStyle::Error => self.color.unwrap_or(theme.destructive), // Beispiel
        // };

        parent
            .spawn((Node {
                justify_content: JustifyContent::Center,
                overflow: Overflow::visible(),
                max_width: Val::Px(0.0),

                ..default()
            },))
            .with_children(|builder| {
                builder.spawn((
                    Text::new(self.text.clone()),
                    TextFont {
                        font_size: final_font_size,
                        font: font_handle.clone(),
                        ..default()
                    },
                    TextLayout::new_with_justify(self.alignment).with_no_wrap(),
                    TextColor(final_color),
                ));
            })
            .id()
    }
}
