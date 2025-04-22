// src/ui/systems.rs (NEUE DATEI)
use super::{
    theme::UiTheme,
    widgets::button::{ButtonPressedEvent, WidgetButton}, // Importiere Marker und Event
};
use bevy::prelude::*;

/// System, das auf Interaktionen mit `WidgetButton`s reagiert
/// und deren Aussehen basierend auf dem `UiTheme` anpasst.
/// Sendet ein `ButtonPressedEvent`, wenn ein Button gedrückt wird.
pub fn button_interaction_system(
    theme: Res<UiTheme>,
    mut interaction_query: Query<
        (Entity, &Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<WidgetButton>), // Reagiere nur auf unsere Buttons
    >,
    mut button_press_ew: EventWriter<ButtonPressedEvent>, // Event Writer
) {
    let button_style = &theme.button_style; // Hole Style einmal

    for (entity, interaction, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = button_style.pressed_background.into();
                *border_color = button_style.pressed_border.into();
                // Sende das Event, wenn der Button gedrückt wird
                button_press_ew.send(ButtonPressedEvent(entity));
                // info!("Button pressed: {:?}", entity); // Debug Log
            }
            Interaction::Hovered => {
                *bg_color = button_style.hovered_background.into();
                *border_color = button_style.hovered_border.into();
            }
            Interaction::None => {
                *bg_color = button_style.normal_background.into();
                *border_color = button_style.normal_border.into();
            }
        }
    }
}

// Läuft jedes Frame, reagiert aber nur, wenn UiTheme verändert wurde
pub fn update_widget_button_style(
    theme: Res<UiTheme>,
    mut button_query: Query<(&Children, &mut Node, &mut BorderColor), With<WidgetButton>>,
    mut text_font_query: Query<&mut TextFont>,
    mut text_color_query: Query<&mut TextColor>,
) {
    // Sobald UiTheme neu geladen wurde:
    if !theme.is_changed() {
        return;
    }

    let style = &theme.button_style;

    for (children, mut node, mut border_color) in button_query.iter_mut() {
        // Padding & Rahmen aktualisieren
        node.padding = style.padding;
        node.border = style.border;
        *border_color = style.normal_border.into();

        // Text-Fonts der Kinder updaten
        for &child in children.iter() {
            if let Ok(mut tf) = text_font_query.get_mut(child) {
                tf.font_size = style.font_size;
                // Optional: falls du auch das Font-Handle live ändern willst:
                // tf.font = theme.default_font().clone();
            }
            if let Ok(mut tc) = text_color_query.get_mut(child) {
                tc.0 = style.text_color; // Passe bei Bedarf auch die Textfarbe an
            }
        }
    }
}
