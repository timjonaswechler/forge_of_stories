// forge_of_stories/crates/my_ui/src/button.rs

use bevy::{color::*, ecs::system::EntityCommands, image::Image, prelude::*, ui::FocusPolicy};
use std::marker::PhantomData;
// ======== Varianten-System ========

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ButtonVariant {
    #[default]
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ButtonSize {
    #[default]
    Default,
    Small,
    Large,
    Icon,
}

// Eine Struktur, um Theme-Werte zu kapseln (ähnlich wie im vorherigen Beispiel)
// Dies könnte optional als Ressource bereitgestellt werden.
#[derive(Debug, Clone)]
pub struct ButtonStyleDef {
    // Kann pub bleiben, falls man sie von außen inspizieren will, oder private sein
    pub background_color: Color,
    pub text_color: Color,
    pub border_color: Color,            // Normalzustand des Borders
    pub hovered_background_factor: f32, // Multiplikator oder Additiv für Hover BG
    pub pressed_background_factor: f32, // Multiplikator oder Additiv für Pressed BG
    pub hovered_border_factor: f32,     // Optional: Borderfarbe bei Hover
    pub pressed_border_factor: f32,     // Optional: Borderfarbe bei Pressed
}

impl ButtonStyleDef {
    // Diese Methode berechnet die HINTERGRUNDFARBE basierend auf Interaktion
    pub fn background(&self, interaction: Interaction, disabled: bool) -> BackgroundColor {
        if disabled {
            // Verdunkelt UND macht leicht transparent für disabled
            return BackgroundColor(self.background_color.darker(0.4).with_alpha(0.6));
        }
        match interaction {
            // Heller machen für Hover
            Interaction::Hovered => BackgroundColor(
                self.background_color
                    .lighter(self.hovered_background_factor),
            ),
            // Dunkler machen für Pressed
            Interaction::Pressed => {
                BackgroundColor(self.background_color.darker(self.pressed_background_factor))
            }
            Interaction::None => BackgroundColor(self.background_color),
        }
    }

    // Optional: Methode zum Berechnen der RANDFARBE bei Interaktion
    pub fn border(&self, interaction: Interaction, disabled: bool) -> BorderColor {
        let mut final_border_color = self.border_color; // Starte mit der Basisfarbe

        if disabled {
            // Mache den Rand transparent, wenn deaktiviert
            final_border_color = self.border_color.with_alpha(0.6);
        } else {
            // Wende Faktoren nur an, wenn die Basisfarbe NICHT NONE ist
            if final_border_color != Color::NONE {
                match interaction {
                    Interaction::Hovered => {
                        // Wende den Hover-Faktor an (positiv = heller, negativ = dunkler)
                        if self.hovered_border_factor > 0.0 {
                            final_border_color =
                                self.border_color.lighter(self.hovered_border_factor);
                        } else if self.hovered_border_factor < 0.0 {
                            final_border_color =
                                self.border_color.darker(-self.hovered_border_factor);
                        }
                        // Bei Faktor 0.0 bleibt die Farbe unverändert
                    }
                    Interaction::Pressed => {
                        // Wende den Pressed-Faktor an
                        if self.pressed_border_factor > 0.0 {
                            final_border_color =
                                self.border_color.lighter(self.pressed_border_factor);
                        } else if self.pressed_border_factor < 0.0 {
                            final_border_color =
                                self.border_color.darker(-self.pressed_border_factor);
                        }
                    }
                    Interaction::None => {
                        // Keine Änderung im None-Zustand
                    }
                }
            }
            // Wenn die Basisfarbe NONE war, bleibt sie NONE (es sei denn, disabled hat sie geändert)
        }

        BorderColor(final_border_color)
    }
}

fn get_button_style_def(variant: ButtonVariant) -> ButtonStyleDef {
    match variant {
        ButtonVariant::Default => ButtonStyleDef {
            background_color: Color::srgb(0.2, 0.5, 0.9),
            text_color: Color::WHITE,
            border_color: Color::NONE,
            hovered_background_factor: 0.15,
            pressed_background_factor: 0.15,
            hovered_border_factor: 0.0, // Kein Rand -> Kein Effekt
            pressed_border_factor: 0.0, // Kein Rand -> Kein Effekt
        },
        ButtonVariant::Destructive => ButtonStyleDef {
            background_color: Color::srgb(0.8, 0.15, 0.15),
            text_color: Color::WHITE,
            border_color: Color::NONE,
            hovered_background_factor: 0.15,
            pressed_background_factor: 0.15,
            hovered_border_factor: 0.0,
            pressed_border_factor: 0.0,
        },
        ButtonVariant::Outline => ButtonStyleDef {
            background_color: Color::NONE,
            text_color: Color::srgb(0.8, 0.8, 0.8),
            border_color: Color::srgb(0.5, 0.5, 0.5), // Sichtbarer Rand
            hovered_background_factor: -0.8,          // Zeigt leichten Hintergrund
            pressed_background_factor: -0.7,          // Etwas stärkerer Hintergrund
            // Rand wird leicht heller bei Hover, leicht dunkler bei Pressed
            hovered_border_factor: 0.3,  // Hellerer Rand
            pressed_border_factor: -0.1, // Dunklerer Rand
        },
        ButtonVariant::Secondary => ButtonStyleDef {
            background_color: Color::srgb(0.4, 0.4, 0.45),
            text_color: Color::WHITE,
            border_color: Color::NONE,
            hovered_background_factor: 0.15,
            pressed_background_factor: 0.15,
            hovered_border_factor: 0.0,
            pressed_border_factor: 0.0,
        },
        ButtonVariant::Ghost => ButtonStyleDef {
            background_color: Color::NONE,
            text_color: Color::srgb(0.8, 0.8, 0.8),
            border_color: Color::NONE,
            hovered_background_factor: -0.9, // Leichter Akzent
            pressed_background_factor: -0.8, // Stärkerer Akzent
            hovered_border_factor: 0.0,
            pressed_border_factor: 0.0,
        },
        ButtonVariant::Link => ButtonStyleDef {
            background_color: Color::NONE,
            text_color: Color::srgb(0.4, 0.6, 1.0),
            border_color: Color::NONE,
            hovered_background_factor: 0.0,
            pressed_background_factor: -0.1, // Evtl. Text abdunkeln? (passiert hier nicht)
            hovered_border_factor: 0.0,
            pressed_border_factor: 0.0,
            // TODO: Überlegen, ob Links bei Hover/Press eine Text-Deko bekommen sollen
        },
    }
}

fn base_style() -> Node {
    Node {
        display: Display::Flex,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        column_gap: Val::Px(8.0),
        // border_radius: BorderRadius::all(Val::Px(5.0)),
        ..default()
    }
}

fn apply_size_style(node: &mut Node, size: ButtonSize, font_size: &mut f32) {
    match size {
        ButtonSize::Default => {
            node.min_height = Val::Px(36.);
            node.padding = UiRect::axes(Val::Px(16.), Val::Px(8.));
            *font_size = 14.;
        }
        ButtonSize::Small => {
            node.min_height = Val::Px(32.);
            node.padding = UiRect::axes(Val::Px(12.), Val::Px(4.));
            *font_size = 12.;
        }
        ButtonSize::Large => {
            node.min_height = Val::Px(40.);
            node.padding = UiRect::axes(Val::Px(32.), Val::Px(8.));
            *font_size = 16.;
        }
        ButtonSize::Icon => {
            node.width = Val::Px(36.);
            node.height = Val::Px(36.);
            node.padding = UiRect::all(Val::Px(8.));
            node.justify_content = JustifyContent::Center;
            *font_size = 16.;
        }
    }
}

// ======== Komponenten ========

/// Marker-Komponente für unsere Buttons
#[derive(Component, Default, Debug)]
pub struct ButtonMarker;

/// Speichert den konfigurierten Stil und Zustand des Buttons
#[derive(Component, Debug)]
pub struct ButtonState {
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub disabled: bool,
    pub style_def: ButtonStyleDef,
}

/// Callback-Komponente. **Achtung: Generisches F hat Nachteile in Systemen!**
/// Systeme, die dies abfragen (`Query<&OnClick<F>>`), funktionieren nur für einen *konkreten* Typ F pro Systemregistrierung.
/// Ein Event-basierter Ansatz ist oft flexibler.
#[derive(Component)]
pub struct OnClick<F: Fn() + Send + Sync + 'static> {
    callback: F,
    _marker: PhantomData<F>,
}

impl<F: Fn() + Send + Sync + 'static> OnClick<F> {
    pub fn new(callback: F) -> Self {
        Self {
            callback,
            _marker: PhantomData,
        }
    }

    pub fn call(&self) {
        if cfg!(debug_assertions) {
            info!("Button clicked!"); // Logging zum Debuggen
        }
        (self.callback)();
    }
}

// Alternative: Event statt Callback-Komponente
#[derive(Event, Debug)]
pub struct ButtonClickedEvent {
    pub button_entity: Entity,
    // Evtl. weitere Daten hinzufügen
}

// ======== Kinder-Definitionen ========

// Enum statt loser Strings/Entities, um Kinder strukturiert zu definieren
#[derive(Clone)]
pub enum ButtonChild {
    Text(String),
    Icon(Handle<Image>),
}

// ======== Builder ========

pub struct ButtonBuilder<F: Fn() + Send + Sync + 'static = fn()> {
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    children_defs: Vec<ButtonChild>, // Liste der Kinder-Definitionen
    on_click: Option<F>,

    width: Option<Val>, // <<< Optional: Breite hinzufügen
    height: Option<Val>,
    markers: Vec<Box<dyn FnOnce(&mut EntityCommands) + Send + Sync>>,
}

// Standardmäßig keinen Callback, Typ F = fn()
impl ButtonBuilder<fn()> {
    pub fn new() -> Self {
        Self {
            variant: ButtonVariant::Default,
            size: ButtonSize::Default,
            disabled: false,
            children_defs: Vec::new(),
            on_click: None,

            width: None,  // Default: Auto
            height: None, // Default: Wird durch Size bestimmt
            markers: Vec::new(),
        }
    }
}

impl<F: Fn() + Send + Sync + 'static> ButtonBuilder<F> {
    // Ermöglicht das Setzen eines beliebigen Callbacks, ändert den Typ des Builders
    pub fn on_click<NewF: Fn() + Send + Sync + 'static>(
        self,
        callback: NewF,
    ) -> ButtonBuilder<NewF> {
        ButtonBuilder {
            variant: self.variant,
            size: self.size,
            disabled: self.disabled,
            children_defs: self.children_defs,
            on_click: Some(callback),

            width: self.width,
            height: self.height,
            markers: self.markers,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    // Icon hinzufügen (nimmt Handle<Image>)
    pub fn with_icon(mut self, icon_handle: Handle<Image>) -> Self {
        // Sicherstellen, dass bei reinem Icon-Button nur das Icon drin ist
        if self.size == ButtonSize::Icon {
            self.children_defs.clear();
        }
        self.children_defs.push(ButtonChild::Icon(icon_handle));
        self
    }

    // Text hinzufügen
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        if self.size != ButtonSize::Icon {
            // Text nicht hinzufügen bei Icon-Size
            self.children_defs.push(ButtonChild::Text(text.into()));
        }
        self
    }

    // Spawn-Methode
    pub fn spawn<'a>(self, parent: &'a mut ChildBuilder, font: Handle<Font>) -> EntityCommands<'a> {
        // 1) Standard-Style holen
        let style_def = get_button_style_def(self.variant);

        // 2) Node mit Basiswerten erstellen
        let mut node = base_style();
        let mut font_size = 14.0;
        apply_size_style(&mut node, self.size, &mut font_size);

        // 3) Explizit gesetzte Breite/Höhe anwenden
        if let Some(w) = self.width {
            node.width = w;
        }
        if let Some(h) = self.height {
            node.height = h;
            node.min_height = h;
        }

        // 4) Entität spawnen – ohne deprecated Bundles!
        let mut cmd = parent.spawn((
            // Behaviour-Komponenten
            Button,
            node, // Layout + Style
            BackgroundColor(style_def.background(Interaction::None, self.disabled).0),
            BorderColor(style_def.border(Interaction::None, self.disabled).0),
            // Fokus ggf. durchlassen, wenn disabled
            if self.disabled {
                FocusPolicy::Pass
            } else {
                FocusPolicy::Block
            },
            // Eigene Marker …
            ButtonMarker,
            ButtonState {
                variant: self.variant,
                size: self.size,
                disabled: self.disabled,
                style_def: style_def.clone(),
            },
        ));

        // 5) Optionales Callback
        if let Some(cb) = self.on_click {
            cmd.insert(OnClick::new(cb));
        }

        // 6) Zusatz-Marker aus dem Builder
        for add_marker in self.markers {
            add_marker(&mut cmd);
        }

        // 7) Kinder (Text/Icon)
        cmd.with_children(|cb| {
            for child_def in self.children_defs {
                match child_def {
                    ButtonChild::Text(text) => {
                        // Alle Zweige enden mit ;, damit der match () zurückgibt
                        let _ = cb.spawn((
                            Text::new(text),
                            TextFont {
                                font: font.clone(),
                                font_size,
                                ..default()
                            },
                            TextColor(style_def.text_color),
                        ));
                    }
                    ButtonChild::Icon(handle) => {
                        let icon_px = match self.size {
                            ButtonSize::Small => 14.0,
                            ButtonSize::Icon => 20.0,
                            _ => 16.0,
                        };

                        // `handle.into()` nutzt From<Handle<Image>> für ImageNode
                        let bundle = (
                            Node {
                                // Layout für das Icon
                                width: Val::Px(icon_px),
                                height: Val::Px(icon_px),
                                ..default()
                            },
                            ImageNode::from(handle),
                            // Farbe/Tint:
                            TextColor(style_def.text_color),
                        );
                        let _ = cb.spawn(bundle);
                    }
                };
            }
        });

        cmd
    }
}
// ======== Systeme ========

pub fn update_button_visuals(
    // --- Query für die Buttons ---
    mut buttons: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &ButtonState,
            Option<&Children>, // <<< Zugriff auf Kinder hinzugefügt
        ),
        (Changed<Interaction>, With<ButtonMarker>),
    >,
    // --- Separate Queries für die Kind-Elemente ---
    mut text_query: Query<&mut TextColor, With<Text>>, // <<< Query für UI Text,
    mut image_query: Query<&mut ImageNode>,            // <<< Query für UI Images (Icons)
) {
    // Iteriere über alle Buttons, deren Interaktionsstatus sich geändert hat
    // Verwendung des for-Loops für bessere Lesbarkeit
    for (interaction, mut bg_color, mut border_color, state, children_opt) in buttons.iter_mut() {
        // 1. Aktualisiere Hintergrundfarbe des Buttons
        *bg_color = state.style_def.background(*interaction, state.disabled);

        // 2. Aktualisiere Randfarbe des Buttons
        *border_color = state.style_def.border(*interaction, state.disabled);

        // --- Kinderfarben anpassen ---
        if let Some(children) = children_opt {
            // Prüfe, ob der Button Kinder hat
            for &child_entity in children.iter() {
                // Iteriere über die Kinder-Entities

                // VERSUCH 1: Kind als Text zu stylen
                if let Ok(mut text) = text_query.get_mut(child_entity) {
                    // Bestimme die Zielfarbe für den Text
                    let target_text_color = if state.disabled {
                        // Wenn Button deaktiviert, Text ausgrauen (transparent machen)
                        state.style_def.text_color.with_alpha(0.6)
                    } else {
                        // Hier KÖNNTE man später auf Hover/Press reagieren,
                        // z.B. text_color heller/dunkler machen basierend auf `interaction`
                        // match *interaction {
                        //    Interaction::Hovered => state.style_def.text_color.lighter(0.1),
                        //    _ => state.style_def.text_color
                        // }
                        state.style_def.text_color // Standardmäßig normale Textfarbe
                    };

                    // Wende die Farbe auf alle Sections des Textes an (oft nur eine)
                    text.0 = target_text_color;
                }

                // VERSUCH 2: Kind als Icon (UiImage) zu stylen
                if let Ok(mut image) = image_query.get_mut(child_entity) {
                    // Bestimme die Zielfarbe für das Icon (als Tint)
                    let target_icon_color = if state.disabled {
                        // Wenn Button deaktiviert, Icon ausgrauen (transparent machen)
                        state.style_def.text_color.with_alpha(0.6)
                    } else {
                        // Hier KÖNNTE man später auf Hover/Press reagieren
                        state.style_def.text_color // Standardmäßig normale Tint-Farbe (oft = Textfarbe)
                    };

                    // Wende die Farbe als Tint auf das UiImage an
                    image.color = target_icon_color;
                }
            }
        }
        // --- Ende Kinderfarben ---
    }
}

/// Verarbeitet Klick-Events **NUR** für Buttons mit einem `fn()`-Callback.
/// **ACHTUNG:** Für jeden anderen Closure-Typ `F` bräuchte man ein separates System!
/// -> Deshalb ist der Event-Ansatz (siehe unten) oft besser.
pub fn handle_button_clicks_fn(
    buttons: Query<
        (&Interaction, &OnClick<fn()>, &ButtonState),
        (Changed<Interaction>, With<ButtonMarker>),
    >,
) {
    for (interaction, on_click, state) in buttons.iter() {
        // Führe Callback nur bei *Abschluss* des Klicks aus (Pressed -> Hovered oder Pressed -> None)
        // Oder einfach bei `Interaction::Pressed`, je nach gewünschtem Verhalten. Hier nehmen wir Pressed.
        if *interaction == Interaction::Pressed && !state.disabled {
            debug!("Calling fn() callback");
            on_click.call();
        }
    }
}

/// Alternatives System: Sendet ein Event beim Klick, anstatt einen Callback direkt aufzurufen.
/// Dieses System kann für *alle* Buttons funktionieren, unabhängig von einem Callback.
pub fn handle_button_clicks_event(
    mut interactions: Query<
        (Entity, &Interaction, &ButtonState),
        (Changed<Interaction>, With<ButtonMarker>),
    >,
    mut button_clicked_events: EventWriter<ButtonClickedEvent>,
) {
    for (entity, interaction, state) in interactions.iter_mut() {
        if *interaction == Interaction::Pressed && !state.disabled {
            debug!("Sending ButtonClickedEvent for entity {:?}", entity);
            button_clicked_events.send(ButtonClickedEvent {
                button_entity: entity,
            });
        }
    }
}
