// crates/forge_ui/src/checkbox.rs

use super::theme::UiTheme;
use bevy::{ecs::system::EntityCommands, prelude::*, ui::FocusPolicy};

// --- Komponenten ---

/// Marker-Komponente für die Checkbox selbst (der klickbare Bereich).
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct CheckboxMarker;

/// Speichert den aktuellen Zustand der Checkbox.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckboxState {
    pub checked: bool,
    pub disabled: bool,
}

impl Default for CheckboxState {
    fn default() -> Self {
        Self {
            checked: false,
            disabled: false,
        }
    }
}

/// Komponente, die die Entity des internen Checkmark-Icons speichert.
/// Wird verwendet, um dessen Sichtbarkeit zu steuern.
#[derive(Component, Debug, Clone, Copy)]
pub struct CheckmarkIconEntity(Entity);

// --- Events ---

/// Wird gesendet, wenn sich der Zustand der Checkbox ändert.
#[derive(Event, Debug, Clone)]
pub struct CheckboxChangedEvent {
    pub checkbox_entity: Entity,
    pub is_checked: bool,
}

// --- Builder ---

pub struct CheckboxBuilder {
    checked: bool,
    disabled: bool,
    // Optional: ID für Label-Verknüpfung (HtmlFor)
    // id: Option<String>, // In Bevy weniger direkt relevant, aber für Struktur möglich
    markers: Vec<Box<dyn FnOnce(&mut EntityCommands) + Send + Sync>>,
}

impl Default for CheckboxBuilder {
    fn default() -> Self {
        Self {
            checked: false,
            disabled: false,
            // id: None,
            markers: Vec::new(),
        }
    }
}

impl CheckboxBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Setzt den initialen Zustand der Checkbox.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Deaktiviert die Checkbox (visuell und funktional).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    // Optional: Methode zum Hinzufügen einer ID
    // pub fn id(mut self, id: impl Into<String>) -> Self {
    //     self.id = Some(id.into());
    //     self
    // }

    /// Fügt eine generische Closure hinzu, um die EntityCommands nach dem Spawnen zu modifizieren.
    pub fn add_marker(
        mut self,
        func: impl FnOnce(&mut EntityCommands) + Send + Sync + 'static,
    ) -> Self {
        self.markers.push(Box::new(func));
        self
    }

    /// Spawnt die Checkbox als Kind des UI-Parents.
    #[must_use = "Commands should generally be used"]
    pub fn spawn<'w, 'a>(
        self,
        parent: &'a mut ChildBuilder<'w>,
        theme: &UiTheme,
        checkmark_icon_handle: &Handle<Image>,
    ) -> EntityCommands<'a> {
        // Styling-Konstanten (könnten auch aus Theme kommen)
        let checkbox_size = Val::Px(16.0); // Entspricht h-4 w-4
        let checkmark_size = Val::Px(16.0); // Icon-Größe
        let border_width = 1.0;

        // Entität für das Checkmark-Icon (wird gleich gespawnt)
        let mut checkmark_entity = Entity::PLACEHOLDER;

        let mut checkbox_cmd = parent.spawn((
            CheckboxMarker, // Marker
            Button,         // Damit sie klickbar ist
            Node {
                // Quadratische Box
                width: checkbox_size,
                height: checkbox_size,
                // Wichtig für Icon-Zentrierung
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                // Rand
                border: UiRect::all(Val::Px(border_width)),
                // Rundung (angepasst von 'rounded-sm')
                ..default()
            },
            BorderRadius::all(theme.radius * 0.5), // Kleinerer Radius als bei Buttons
            // Start-Styling (wird im System aktualisiert)
            BackgroundColor(Color::NONE),
            BorderColor(theme.primary),
            // FocusPolicy
            if self.disabled {
                FocusPolicy::Pass
            } else {
                FocusPolicy::Block
            },
            Interaction::default(),
            CheckboxState {
                checked: self.checked,
                disabled: self.disabled,
            },
            // Anfangs Platzhalter, wird unten aktualisiert
            CheckmarkIconEntity(checkmark_entity),
        ));

        // Spawn Checkmark Icon als Kind
        checkbox_cmd.with_children(|builder| {
            builder.spawn((
                Node {
                    width: checkmark_size,
                    height: checkmark_size,
                    margin: UiRect::axes(Val::Px(0.0), Val::Px(2.0)),
                    ..default()
                },
                ImageNode {
                    image: checkmark_icon_handle.clone(),
                    ..default()
                },
                BackgroundColor(theme.primary_foreground),
                FocusPolicy::Pass,
                // Startet unsichtbar, wenn nicht checked
                if self.checked {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                // Farbe/Tint kommt aus dem Theme (Primär-Vordergrund für Kontrast)
            ));
        });

        // Update CheckmarkIconEntity mit der echten Entity ID
        // TODO: Wenn hier ein Fehler auftritt, könnte es sein, dass die Entity nicht existiert oder so einen Playeholder ID nutzen.

        checkbox_cmd.insert(CheckmarkIconEntity(checkbox_cmd.id()));

        // Marker hinzufügen
        for marker_fn in self.markers {
            marker_fn(&mut checkbox_cmd);
        }

        checkbox_cmd
    }
}

// --- Systeme ---

/// Aktualisiert das Aussehen der Checkbox basierend auf Zustand und Interaktion.
pub fn update_checkbox_visuals(
    theme: Res<UiTheme>,
    mut checkbox_query: Query<
        (
            &Interaction,
            &CheckboxState,
            &mut BackgroundColor,
            &mut BorderColor,
            &CheckmarkIconEntity, // Zum Holen der Icon-Entität
        ),
        (Changed<Interaction>, With<CheckboxMarker>),
    >, // Reaktion auf Klick oder Statusänderung (letzteres noch nicht hier)
    mut icon_visibility_query: Query<&mut Visibility>, // Um Icon ein/auszublenden
) {
    for (interaction, state, mut bg_color, mut border_color, checkmark_icon) in
        checkbox_query.iter_mut()
    {
        // Basis-Styling basierend auf checked/disabled
        let base_bg_color = if state.checked {
            theme.primary
        } else {
            Color::NONE
        };
        let base_border_color = theme.primary; // Oder theme.input / theme.border ?

        // Endgültige Farben (Disabled und Hover/Pressed)
        if state.disabled {
            *bg_color = BackgroundColor(base_bg_color.with_alpha(0.5));
            *border_color = BorderColor(base_border_color.with_alpha(0.5));
        } else {
            // Einfacher Hover-Effekt (optional)
            let hover_factor = 0.1;
            let pressed_factor = -0.1;

            *bg_color = match *interaction {
                Interaction::Hovered => BackgroundColor(base_bg_color.lighter(hover_factor)),
                Interaction::Pressed => BackgroundColor(base_bg_color.darker(-pressed_factor)),
                Interaction::None => BackgroundColor(base_bg_color),
            };
            *border_color = match *interaction {
                Interaction::Hovered => BorderColor(base_border_color.lighter(hover_factor)),
                Interaction::Pressed => BorderColor(base_border_color.darker(-pressed_factor)),
                Interaction::None => BorderColor(base_border_color),
            };
        }

        // Sichtbarkeit des Icons aktualisieren (dies sollte idealerweise reagieren,
        // wenn sich CheckboxState *ändert*, nicht nur bei Interaction)
        if let Ok(mut icon_visibility) = icon_visibility_query.get_mut(checkmark_icon.0) {
            *icon_visibility = if state.checked {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        } else {
            error!(
                "CheckmarkIconEntity {:?} not found for checkbox!",
                checkmark_icon.0
            );
        }
    }
}

/// Reagiert auf Klicks auf die Checkbox, ändert den Zustand und sendet ein Event.
pub fn handle_checkbox_clicks(
    mut checkbox_query: Query<
        (Entity, &Interaction, &mut CheckboxState),
        (Changed<Interaction>, With<CheckboxMarker>),
    >,
    mut ev_checkbox_changed: EventWriter<CheckboxChangedEvent>,
    // Wir brauchen hier jetzt KEINE icon_visibility_query mehr,
    // das Aussehen wird vom update_checkbox_visuals System gesteuert,
    // sobald sich der CheckboxState geändert hat (im nächsten Frame).
) {
    for (entity, interaction, mut state) in checkbox_query.iter_mut() {
        // Umschalten bei Klick (Released = Interaction geht von Pressed weg)
        if *interaction == Interaction::Pressed && !state.disabled {
            // Den Zustand direkt hier umschalten
            state.checked = !state.checked;
            info!("Checkbox {:?} toggled to {}", entity, state.checked);

            // Event senden, damit andere Systeme reagieren können
            ev_checkbox_changed.send(CheckboxChangedEvent {
                checkbox_entity: entity,
                is_checked: state.checked,
            });

            // WICHTIG: Der update_checkbox_visuals muss auch reagieren,
            // wenn sich *nur* der CheckboxState ändert, nicht nur die Interaction.
            // Dies geschieht implizit, wenn der nächste Frame gerendert wird, ODER
            // man fügt Changed<CheckboxState> zur Query in update_checkbox_visuals hinzu.
        }
    }
}

/// Separates System, das nur auf die Änderung des CheckboxState reagiert,
/// um sicherzustellen, dass das Icon immer den korrekten Zustand widerspiegelt.
pub fn update_checkmark_visibility_on_state_change(
    mut checkbox_query: Query<(&CheckboxState, &CheckmarkIconEntity), Changed<CheckboxState>>, // Nur wenn State sich ändert
    mut icon_visibility_query: Query<&mut Visibility>,
) {
    for (state, checkmark_icon) in checkbox_query.iter_mut() {
        if let Ok(mut icon_visibility) = icon_visibility_query.get_mut(checkmark_icon.0) {
            *icon_visibility = if state.checked {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        } else {
            error!(
                "CheckmarkIconEntity {:?} not found when state changed!",
                checkmark_icon.0
            );
        }
    }
}
