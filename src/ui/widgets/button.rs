use crate::ui::theme::{ButtonStyle, UiTheme};
use bevy::prelude::*;

/// Marker-Komponente für unsere benutzerdefinierten Buttons.
#[derive(Component)]
pub struct WidgetButton;

/// Event, das gesendet wird, wenn ein WidgetButton gedrückt wird.
/// Enthält die Entity des Buttons.
#[derive(Event, Debug, Clone)]
pub struct ButtonPressedEvent(pub Entity);

/// Ein Builder zum Erstellen von standardisierten Buttons.
pub struct ButtonWidgetBuilder {
    text: String,
    width: Option<Val>,
    height: Option<Val>,
    // NEU: beliebige Marker-Komponenten
    markers: Vec<Box<dyn FnOnce(&mut EntityCommands) + Send + Sync>>,
}

impl ButtonWidgetBuilder {
    /// Erstellt einen neuen Button Builder mit dem gegebenen Text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            width: None,                 // Standardmäßig Auto-Breite
            height: Some(Val::Px(50.0)), // Standard-Höhe (Beispiel)
            markers: Vec::new(),
        }
    }

    /// Setzt eine feste Breite für den Button.
    pub fn with_width(mut self, width: Val) -> Self {
        self.width = Some(width);
        self
    }

    /// Setzt eine feste Höhe für den Button.
    pub fn with_height(mut self, height: Val) -> Self {
        self.height = Some(height);
        self
    }

    /// Fügt eine Marker-Komponente hinzu (z.B. StartGameButtonMarker).
    pub fn with_marker<T: Component + Default + Send + Sync + 'static>(mut self) -> Self {
        self.markers.push(Box::new(|ec| {
            ec.insert(T::default());
        }));
        self
    }

    /// Alternative: Marker mit custom value
    pub fn with_marker_value<T: Component + Send + Sync + 'static>(mut self, marker: T) -> Self {
        self.markers.push(Box::new(move |ec| {
            ec.insert(marker);
        }));
        self
    }

    /// Spawnt den Button in der UI-Hierarchie.
    /// Gibt die Entity des erstellten Buttons zurück.
    pub fn spawn(self, parent: &mut ChildBuilder, theme: &UiTheme) -> Entity {
        let style = &theme.button_style;

        let mut entity_commands = parent.spawn((
            Button,
            Node {
                width: self.width.unwrap_or(Val::Auto),
                height: self.height.unwrap_or(Val::Auto),
                border: style.border,
                padding: style.padding,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(style.normal_background),
            BorderColor(style.normal_border),
            Interaction::default(),
            WidgetButton,
        ));

        // Marker-Komponenten hinzufügen
        for marker in self.markers {
            marker(&mut entity_commands);
        }

        entity_commands.with_children(|builder| {
            builder.spawn((
                Text::new(self.text.clone()),
                TextFont {
                    font: theme.default_font().clone(),
                    font_size: style.font_size,
                    ..default()
                },
                TextColor(style.text_color),
            ));
        });

        entity_commands.id()
    }
}
