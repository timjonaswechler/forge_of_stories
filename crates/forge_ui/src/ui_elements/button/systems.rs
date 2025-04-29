use super::components::{ButtonMarker, ButtonState, OnClick};
use super::events::ButtonClickedEvent;
use bevy::prelude::*;

/// System to update the button's background, border, and children's colors
pub fn update_button_visuals(
    mut buttons: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &ButtonState, // << Enthält die style_def
            Option<&Children>,
        ),
        (Changed<Interaction>, With<ButtonMarker>),
    >,
    mut text_query: Query<&mut TextColor>, // TextColor direkt ändern
    // Zugriff auf BackgroundColor der ImageBundle Node
    mut image_bg_color_query: Query<&mut BackgroundColor, (With<ImageNode>, Without<ButtonMarker>)>,
) {
    for (interaction, mut bg_color, mut border_color, state, children_opt) in buttons.iter_mut() {
        // 1. Update Button Background and Border using ButtonState.style_def methods
        // Diese Methoden enthalten jetzt die neue Overlay-Logik
        *bg_color = state.style_def.background(*interaction, state.disabled);
        *border_color = state.style_def.border(*interaction, state.disabled);

        // 2. Update Children Colors
        if let Some(children) = children_opt {
            // Hole die korrekte *aktuelle* Textfarbe basierend auf dem Disabled-Status
            // (Interaktion beeinflusst Textfarbe normalerweise nicht, nur Disabled)
            let child_target_color = state.style_def.text_color(state.disabled);

            for &child_entity in children.iter() {
                // Text aktualisieren (direkt TextColor Komponente)
                if let Ok(mut text_color_component) = text_query.get_mut(child_entity) {
                    *text_color_component = TextColor(child_target_color);
                }

                // Icon (ImageBundle BackgroundColor) aktualisieren
                if let Ok(mut image_bg) = image_bg_color_query.get_mut(child_entity) {
                    *image_bg = BackgroundColor(child_target_color);
                }
            }
        }
    }
}

/// System that detects button presses and sends ButtonClickedEvent
pub fn handle_button_clicks_event(
    interactions: Query<
        (Entity, &Interaction, &ButtonState),
        (Changed<Interaction>, With<ButtonMarker>),
    >,
    mut button_clicked_events: EventWriter<ButtonClickedEvent>,
) {
    for (entity, interaction, state) in interactions.iter() {
        // .iter() statt .iter_mut()
        if *interaction == Interaction::Pressed && !state.disabled {
            debug!("Button {:?} pressed, sending event.", entity);
            button_clicked_events.send(ButtonClickedEvent {
                button_entity: entity,
            });
        }
    }
}

/// Optional: System to handle direct `fn()` callbacks
pub fn handle_button_clicks_fn(
    buttons: Query<
        (&Interaction, &OnClick<fn()>, &ButtonState),
        (Changed<Interaction>, With<ButtonMarker>),
    >,
) {
    for (interaction, on_click, state) in buttons.iter() {
        if *interaction == Interaction::Pressed && !state.disabled {
            debug!("Calling fn() callback for button.");
            on_click.call();
        }
    }
}
