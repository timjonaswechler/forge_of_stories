// crates/forge_ui/src/card.rs

use bevy::{ecs::system::EntityCommands, prelude::*};

use super::theme::UiTheme;
// Wichtig: ButtonBuilder importieren, falls er im Footer verwendet wird
use super::button::ButtonBuilder;

// --- Marker Komponenten (bleiben gleich) ---
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Card;

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct CardHeader;

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct CardContent;

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct CardFooter;

// --- Datenstruktur für Builder ---
// Enum, um die verschiedenen Teile der Karte im Builder zu speichern
#[derive(Clone)]
pub enum CardSection {
    Header(Vec<NodeElement>), // Kann Titel, Beschreibung etc. enthalten
    Content(Vec<NodeElement>),
    Footer(Vec<NodeElement>),
}

// Generischer Typ, um verschiedene UI-Elemente als Kinder zu erlauben
// (Hier sehr vereinfacht, könnte komplexer sein für Input, Select etc.)
#[derive(Clone)]
pub enum NodeElement {
    Text {
        content: String,
        style: ElementStyle, // z.B. Title, Description, Muted, Normal
        font_size: Option<f32>,
    },
    Image(Handle<Image>),
    Button(ButtonBuilder), // << Erlaubt direktes Einfügen eines konfigurierten Buttons
                           // Hier könnten später weitere Elemente hinzukommen:
                           // Input(...), Select(...), HorizontalLayout(Vec<NodeElement>), ...
}

// Beispiel-Enum für Textstile innerhalb der Karte
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum ElementStyle {
    #[default]
    Normal,
    Title,
    Description,
    Muted,
}

// --- Der Card Builder ---

pub struct CardBuilder {
    width: Option<Val>,
    sections: Vec<CardSection>,
    // Zukünftig: variant, custom_padding, etc.
}

impl Default for CardBuilder {
    fn default() -> Self {
        Self {
            width: None, // Default: passt sich Inhalt oder Parent an
            sections: Vec::new(),
        }
    }
}

impl CardBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn width(mut self, width: Val) -> Self {
        self.width = Some(width);
        self
    }

    // Methoden zum Hinzufügen von Sektionen

    pub fn with_header(mut self, elements: Vec<NodeElement>) -> Self {
        self.sections.push(CardSection::Header(elements));
        self
    }

    pub fn with_content(mut self, elements: Vec<NodeElement>) -> Self {
        self.sections.push(CardSection::Content(elements));
        self
    }

    pub fn with_footer(mut self, elements: Vec<NodeElement>) -> Self {
        self.sections.push(CardSection::Footer(elements));
        self
    }

    // --- Spawnen ---

    #[must_use]
    pub fn spawn<'w, 'a>(
        self,
        parent: &'a mut ChildBuilder<'w>,
        theme: &UiTheme,
        // Font wird für Text-Elemente benötigt, daher hier übergeben
        font_handle: &Handle<Font>,
    ) -> EntityCommands<'a> {
        let mut card_style = Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            border: UiRect::all(Val::Px(1.0)),

            ..default()
        };
        if let Some(w) = self.width {
            card_style.width = w;
        }

        let mut card_entity_commands = parent.spawn((
            Card, // Marker
            Node { ..card_style },
            BackgroundColor(theme.card),
            BorderColor(theme.border),
            BorderRadius::all(theme.radius), // Bevy 0.15+
        ));

        card_entity_commands.with_children(|card_body| {
            for section in self.sections {
                match section {
                    CardSection::Header(elements) => {
                        spawn_section(
                            card_body,
                            theme,
                            font_handle,
                            CardHeader,          // Marker
                            header_style(theme), // Spezifisches Style für Header
                            elements,
                        );
                    }
                    CardSection::Content(elements) => {
                        spawn_section(
                            card_body,
                            theme,
                            font_handle,
                            CardContent,          // Marker
                            content_style(theme), // Spezifisches Style für Content
                            elements,
                        );
                    }
                    CardSection::Footer(elements) => {
                        spawn_section(
                            card_body,
                            theme,
                            font_handle,
                            CardFooter,          // Marker
                            footer_style(theme), // Spezifisches Style für Footer
                            elements,
                        );
                    }
                }
            }
        });

        card_entity_commands
    }
}

// --- Interne Helper zum Spawnen der Sektionen und Elemente ---

/// Generische Funktion zum Spawnen einer Karten-Sektion (Header, Content, Footer)
fn spawn_section<'w, 'a, M: Component + Default>(
    parent: &'a mut ChildBuilder<'w>,
    theme: &UiTheme,
    font_handle: &Handle<Font>,
    marker: M,    // Der spezifische Marker (CardHeader, CardContent, etc.)
    style: Style, // Das spezifische Layout der Sektion
    elements: Vec<NodeElement>,
) -> EntityCommands<'a> {
    let mut section_entity_cmd = parent.spawn((marker, Node { style, ..default() }));

    section_entity_cmd.with_children(|section_body| {
        for element in elements {
            spawn_element(section_body, theme, font_handle, element);
        }
    });

    section_entity_cmd
}

/// Spawnt ein einzelnes Element innerhalb einer Sektion
fn spawn_element<'w, 'a>(
    parent: &'a mut ChildBuilder<'w>,
    theme: &UiTheme,
    font_handle: &Handle<Font>,
    element: NodeElement,
) {
    match element {
        NodeElement::Text {
            content,
            style,
            font_size,
        } => {
            let (text_color, default_size) = match style {
                ElementStyle::Normal => (theme.card_foreground, 14.0),
                ElementStyle::Title => (theme.card_foreground, 18.0), // Größer für Titel
                ElementStyle::Description => (theme.muted_foreground, 14.0), // Gedämpft für Beschreibung
                ElementStyle::Muted => (theme.muted_foreground, 14.0),
            };

            parent.spawn((
                Text::new(content),
                TextFont {
                    font: font_handle.clone(),
                    font_size: font_size.unwrap_or(default_size),
                    ..default()
                },
                TextColor(text_color),
            ));
        }
        NodeElement::Image(handle) => {
            parent.spawn((
                Node {
                    width: Val::Px(14.0),
                    height: Val::Px(14.0),
                    margin: UiRect::axes(Val::Px(0.0), Val::Px(2.0)),
                    ..default()
                },
                ImageNode {
                    image: handle,
                    ..default()
                },
                BackgroundColor(theme.card_foreground),
                Visibility::Inherited,
            ));
        }

        NodeElement::Button(builder) => {
            // Spawnen des Buttons direkt mit dem übergebenen Builder
            let _ = builder.spawn(parent, font_handle.clone(), theme);
        } // Hier würden weitere Element-Typen (Input, Select, Layouts...) behandelt werden
    }
}

// --- Styling-Funktionen für die Sektionen ---

fn header_style(theme: &UiTheme) -> Node {
    let padding_val = match theme.radius {
        Val::Px(radius) => Val::Px(radius + 6.0),
        other => other,
    };
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(padding_val), // Etwas Padding basierend auf Radius
        row_gap: Val::Px(6.0),             // Platz zwischen Titel/Beschreibung
        ..default()
    }
}
fn content_style(theme: &UiTheme) -> Node {
    let padding_val = match theme.radius {
        Val::Px(radius) => Val::Px(radius + 6.0),
        other => other,
    };
    Node {
        padding: UiRect {
            left: padding_val,
            right: padding_val,
            top: Val::Px(0.0), // Kein Top-Padding, wie bei Shadcn oft üblich
            bottom: padding_val,
        },
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(12.0), // Größerer Abstand zwischen Content-Elementen
        ..default()
    }
}
fn footer_style(theme: &UiTheme) -> Node {
    let padding_val = match theme.radius {
        Val::Px(radius) => Val::Px(radius + 6.0),
        other => other,
    };
    Node {
        padding: UiRect {
            left: padding_val,
            right: padding_val,
            top: Val::Px(0.0), // Kein Top-Padding
            bottom: padding_val,
        },
        display: Display::Flex,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::FlexEnd, // Default: rechtsbündig
        column_gap: Val::Px(8.0),                 // Platz zwischen Footer-Elementen (z.B. Buttons)
        ..default()
    }
}
