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
