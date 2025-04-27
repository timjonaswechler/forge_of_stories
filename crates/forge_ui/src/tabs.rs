// crates/forge_ui/src/tabs.rs
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use serde::de;
use std::collections::HashMap; // Für Content-Bereiche
use std::hash::Hash; // Für generischen Wert-Typ

use super::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use super::theme::UiTheme; // Nutzen Button als Basis für Trigger

// --- Komponenten ---

/// Eindeutiger Bezeichner für einen Tab.
/// Kann beliebig erweitert werden (z.B. mit String, u32, etc.).
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TabId(pub String);

// Optional: Hilfsfunktionen für bessere Lesbarkeit
impl TabId {
    pub fn new(label: String) -> Self {
        TabId(label)
    }
}

/// Marker für den Haupt-Tabs-Container.
#[derive(Component, Default, Debug, Clone)]
pub struct TabsContainer;

/// Zustand des Tabs-Containers, speichert den aktuell aktiven Wert.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct TabsState<T: Component + PartialEq + Eq + Hash + Clone + Send + Sync + 'static> {
    pub active_value: T,
}

/// Marker für die Liste der Tab-Triggers.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct TabsList;

/// Marker und Zustand für einen einzelnen Tab-Trigger (Button).
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct TabsTrigger<T: Component + PartialEq + Eq + Hash + Clone + Send + Sync + 'static> {
    pub value: T,
    pub parent_tabs: Entity, // Verweis auf den TabsContainer
    pub is_active: bool,     // Wird vom System gesetzt
    pub disabled: bool,
}

/// Marker und Zustand für einen Content-Bereich, der zu einem Trigger-Wert gehört.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct TabsContent<T: Component + PartialEq + Eq + Hash + Clone + Send + Sync + 'static> {
    pub value: T,
}

// --- Events ---

/// Event, das ausgelöst wird, wenn ein anderer Tab aktiviert wird.
#[derive(Event, Debug, Clone)]
pub struct TabChangedEvent<T: Component + PartialEq + Eq + Hash + Clone + Send + Sync + 'static> {
    pub tabs_entity: Entity, // Der Haupt-Tabs-Container
    pub active_value: T,     // Der neue aktive Wert
}

// --- Builder ---

// Helferstruktur für einen einzelnen Tab (Trigger + zugehöriger Content)
pub struct TabItem<T: Component + PartialEq + Eq + Hash + Clone + Send + Sync + 'static> {
    value: T,
    trigger_label: String, // Oder NodeElement? Einfachheit halber erstmal String.
    content_builder:
        Option<Box<dyn FnOnce(&mut ChildBuilder, &UiTheme, &Handle<Font>) + Send + Sync>>, // Closure zum Bauen des Contents
    disabled: bool,
}

#[derive(Bundle, Clone, Default)]
pub struct TabsListStyle {
    pub node: Node,
    pub border_radius: BorderRadius,
    // Weitere Style-Komponenten nach Bedarf
}
// Builder für die gesamte Tabs-Komponente
pub struct TabsBuilder<T: Component + PartialEq + Eq + Hash + Clone + Send + Sync + 'static> {
    default_value: Option<T>,
    tabs: Vec<TabItem<T>>,
    // Optional: Styling für TabsList
    list_style: Option<TabsListStyle>,
    // Optional: Styling für TabsContent
    content_style: Option<Node>,
}

impl<T: Component + PartialEq + Eq + Hash + Clone + Send + Sync + 'static> TabsBuilder<T> {
    /// Erstellt einen neuen TabsBuilder.
    pub fn new(default_value: T) -> Self {
        Self {
            default_value: Some(default_value),
            tabs: Vec::new(),
            list_style: None,
            content_style: None,
        }
    }

    /// Fügt einen Tab hinzu.
    /// `content_builder`: Eine Closure, die `&mut ChildBuilder`, `&UiTheme`, `&Handle<Font>` nimmt
    ///                   und den Inhalt für diesen Tab spawnt.
    pub fn add_tab(
        mut self,
        value: T,
        trigger_label: impl Into<String>,
        content_builder: impl FnOnce(&mut ChildBuilder, &UiTheme, &Handle<Font>) + Send + Sync + 'static,
    ) -> Self {
        self.tabs.push(TabItem {
            value,
            trigger_label: trigger_label.into(),
            content_builder: Some(Box::new(content_builder)),
            disabled: false,
        });
        self
    }

    /// Fügt einen deaktivierten Tab hinzu (ohne Content).
    pub fn add_disabled_tab(mut self, value: T, trigger_label: impl Into<String>) -> Self {
        self.tabs.push(TabItem {
            value,
            trigger_label: trigger_label.into(),
            content_builder: None,
            disabled: true,
        });
        self
    }

    // TODO: Methoden zum Anpassen des list_style / content_style

    /// Spawnt die Tabs-Komponente.
    #[must_use]
    pub fn spawn<'w, 'a>(
        self,
        parent: &'a mut ChildBuilder<'w>,
        theme: &UiTheme,
        font_handle: &Handle<Font>,
    ) -> EntityCommands<'a> {
        let default_value = self
            .default_value
            .clone()
            .expect("TabsBuilder requires a default_value"); // Oder einen Fallback definieren

        let mut content_builders: HashMap<
            T,
            Box<dyn FnOnce(&mut ChildBuilder, &UiTheme, &Handle<Font>) + Send + Sync>,
        > = HashMap::new();

        // Spawnt den Haupt-Container
        let mut tabs_cmd = parent.spawn((
            TabsContainer,
            TabsState {
                active_value: default_value.clone(),
            },
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column, // Liste oben, Content unten

                ..default()
            },
        ));

        let tabs_entity = tabs_cmd.id();

        // Inhalte vorbereiten und Container/Trigger spawnen
        tabs_cmd.with_children(|builder| {
            // 1. TabsList Container spawnen
            let list_node_style = self.list_style.unwrap_or_else(|| TabsListStyle {
                node: Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row, // Trigger nebeneinander
                    height: Val::Px(40.),               // Feste Höhe wie in Shadcn 'h-10'
                    padding: UiRect::all(Val::Px(4.)),  // Entspricht p-1
                    align_items: AlignItems::Center,    // Zentriert Trigger vertikal

                    ..default()
                },
                border_radius: BorderRadius::all(theme.radius * 0.8), // 'rounded-md'
            });

            builder
                .spawn((
                    TabsList,
                    list_node_style,
                    BackgroundColor(theme.muted), // Hintergrund für Liste
                ))
                .with_children(|list_builder| {
                    // 2. TabsTrigger Buttons spawnen
                    for item in &self.tabs {
                        // Bestimmen, ob dieser Trigger initial aktiv ist
                        let is_initially_active = item.value == default_value;

                        // Wir verwenden ButtonBuilder für den Trigger
                        let mut trigger_builder = ButtonBuilder::new()
                            .variant(ButtonVariant::Ghost) // Basis: Ghost (kein BG/Border)
                            .size(ButtonSize::Small) // Passt gut zur h-10 Liste
                            .with_text(&item.trigger_label)
                            .disabled(item.disabled);

                        // Style für den aktiven Zustand (überschreibt Ghost-Style teilweise)
                        // Wir steuern das Aussehen über die Kombo `TabsTrigger.is_active` und ButtonVariant
                        // Das Styling selbst passiert im `update_tabs_visuals`-System.
                        // Wir fügen nur den spezifischen Marker hinzu.

                        // Button spawnen und Trigger-Komponente hinzufügen
                        let _ = trigger_builder
                            .spawn(list_builder, font_handle.clone(), theme)
                            .insert(TabsTrigger {
                                value: item.value.clone(),
                                parent_tabs: tabs_entity,
                                is_active: is_initially_active, // Startzustand
                                disabled: item.disabled,
                            });
                    }
                });

            // 3. Content-Builder sammeln und Content-Nodes vorbereiten
            // for item in self.tabs {
            // if let Some(cb) = item.content_builder {
            // content_builders.insert(item.value.clone(), cb);

            // Spawn Content Container (initial versteckt, wenn nicht aktiv)
            let content_area_style = self.content_style.clone().unwrap_or_else(|| Node {
                margin: UiRect::top(Val::Px(8.)), // 'mt-2' (Abstand zur Liste)
                // WICHTIG: Dieser Container braucht KEIN spezielles Layouting
                //          für seine Kinder, da die Kinder überlappen sollen.
                //          Größe sollte sich anpassen oder vom Parent kommen.
                //          Man könnte width/height auf 100% setzen, falls der TabsContainer
                //          eine feste Größe hat und der Content den Platz füllen soll.
                // width: Val::Percent(100.0),
                // height: Val::Percent(100.0), // Vorsicht hiermit, nur wenn Container Größe hat
                ..default()
            });

            builder
                .spawn(Node {
                    ..content_area_style
                })
                .with_children(|content_area_parent| {
                    // 2. ALLE TabsContent-Nodes ALS KINDER DIESES EINEN Containers spawnen
                    for item in self.tabs.into_iter() {
                        if let Some(cb) = item.content_builder {
                            // Nur wenn Content da ist
                            content_builders.insert(item.value.clone(), cb); // Ownership transfer, no clone
                            let is_visible = item.value == default_value;

                            content_area_parent.spawn((
                                TabsContent {
                                    value: item.value.clone(),
                                },
                                Node {
                                    // WICHTIG: Position auf Absolute setzen!
                                    position_type: PositionType::Absolute,
                                    // Optional: Auf 0,0 in der oberen linken Ecke des Parents festnageln
                                    // top: Val::Px(0.0),
                                    // left: Val::Px(0.0),
                                    // Optional: Volle Breite/Höhe des Parent einnehmen
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                if is_visible {
                                    Visibility::Inherited
                                } else {
                                    Visibility::Hidden
                                },
                            ));
                        }
                    }
                }); // Ende des Content-Area-Containers
                    // }
                    // }
        });

        // 4. Nach dem ersten Bauen die Inhalte der Content-Bereiche füllen
        //    (geht nicht direkt im with_children oben, da wir Query brauchen)
        // Dieser Teil ist tricky ohne apply_deferred. Wir spawnen Content leer und füllen in einem System.
        tabs_cmd.insert(InitialContentBuilders {
            builders: content_builders,
            font_handle: font_handle.clone(),
        });

        tabs_cmd
    }
}

// Temporäre Komponente, um die Content-Builder nach dem Spawnen zu übergeben
#[derive(Component)]
pub struct InitialContentBuilders<
    T: Component + PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
> {
    builders: HashMap<
        T,
        Box<dyn FnOnce(&mut ChildBuilder, &UiTheme, &Handle<Font>) + Send + Sync + 'static>,
    >,
    font_handle: Handle<Font>,
}

// --- Systeme ---

/// System zum initialen Füllen der TabsContent-Bereiche nach dem Spawnen.
/// Läuft einmalig dank `RemovedComponents`.

pub fn populate_initial_tab_content<
    T: Component + PartialEq + Eq + Hash + Clone + Send + Sync + 'static + std::fmt::Debug,
>(
    mut commands: Commands,
    mut content_query: Query<(Entity, &TabsContent<T>, Option<&Children>)>,
    // Query für die Builder-Komponente, die entfernt wird
    mut tabs_container_query: Query<
        (Entity, &mut InitialContentBuilders<T>),
        Added<InitialContentBuilders<T>>,
    >,
    theme_opt: Option<Res<UiTheme>>,
) {
    // --- HIER PRÜFEN ---
    let Some(theme) = theme_opt else {
        // Wenn das Theme noch nicht verfügbar ist, überspringe die Ausführung dieses Frames.
        warn!(
            "UiTheme resource not yet available in populate_initial_tab_content. Skipping frame."
        );
        return;
    };
    for (tabs_entity, mut initial_content) in tabs_container_query.iter_mut() {
        info!(
            "Populating initial content for TabsContainer {:?}",
            tabs_entity
        );
        // Iteriere über alle gespawnten Content-Bereiche
        for (content_entity, content_marker, children_opt) in content_query.iter_mut() {
            // Prüfen, ob Inhalt bereits hinzugefügt wurde (sollte nicht der Fall sein)
            if children_opt.map_or(false, |c| !c.is_empty()) {
                continue;
            }

            // Finde den passenden Builder und entferne ihn aus der Map
            if let Some(builder_fn) = initial_content.builders.remove(&content_marker.value) {
                info!(
                    "  -> Populating content for value: {:?}",
                    content_marker.value
                );
                commands.entity(content_entity).with_children(|parent| {
                    (builder_fn)(parent, &theme, &initial_content.font_handle);
                });
            }
        }

        // Entferne die temporäre Builder-Komponente vom TabsContainer
        commands
            .entity(tabs_entity)
            .remove::<InitialContentBuilders<T>>();
    }
}

/// System, das auf Klicks auf TabsTrigger reagiert.
pub fn handle_tab_triggers<
    T: Component + PartialEq + Eq + Hash + Clone + Send + Sync + 'static + std::fmt::Debug,
>(
    mut trigger_interactions: Query<
        (&Interaction, &TabsTrigger<T>),
        (Changed<Interaction>, With<Button>), // Reagiert auf Klicks auf den Button
    >,
    mut tabs_state_query: Query<&mut TabsState<T>>, // Zustand des Haupt-Containers
    mut ev_tab_changed: EventWriter<TabChangedEvent<T>>,
) {
    for (interaction, trigger) in trigger_interactions.iter_mut() {
        if *interaction == Interaction::Pressed && !trigger.disabled {
            if let Ok(mut tabs_state) = tabs_state_query.get_mut(trigger.parent_tabs) {
                // Nur ändern, wenn es nicht schon der aktive Tab ist
                if tabs_state.active_value != trigger.value {
                    info!(
                        "Tab Trigger clicked: New active value = {:?}",
                        trigger.value
                    );
                    tabs_state.active_value = trigger.value.clone();

                    // Event senden
                    ev_tab_changed.send(TabChangedEvent {
                        tabs_entity: trigger.parent_tabs,
                        active_value: tabs_state.active_value.clone(),
                    });
                }
            } else {
                error!(
                    "Parent TabsContainer {:?} not found for trigger {:?}",
                    trigger.parent_tabs, trigger.value
                );
            }
        }
    }
}

/// System, das das Aussehen der Trigger und die Sichtbarkeit des Contents aktualisiert,
/// wenn sich der aktive Tab im `TabsState` ändert.
pub fn update_tabs_visuals<
    T: Component + PartialEq + Eq + Hash + Clone + Send + Sync + 'static + std::fmt::Debug,
>(
    mut tabs_query: Query<(Entity, &TabsState<T>), Changed<TabsState<T>>>, // Nur wenn State sich ändert
    // Query für alle Trigger und deren zugehörige Button-Komponenten
    mut triggers_query: Query<
        (
            &mut TabsTrigger<T>,
            &mut BackgroundColor, // Style direkt ändern
            &mut BorderColor,     // Style direkt ändern
            // Zugriff auf TextColor für den Trigger-Text? Ja!
            Option<&Children>,
        ),
        With<Button>,
    >, // Stellen sicher, dass es Buttons sind
    // Query für alle Content-Bereiche
    mut contents_query: Query<(&TabsContent<T>, &mut Visibility)>,
    // Text-Komponente der Trigger finden
    mut text_query: Query<&mut TextColor>, // Zugriff auf Text für Farbänderung

    theme_opt: Option<Res<UiTheme>>,
) {
    // --- HIER PRÜFEN ---
    let Some(theme) = theme_opt else {
        // Wenn das Theme noch nicht verfügbar ist, überspringe die Ausführung dieses Frames.
        warn!("UiTheme resource not yet available in update_tabs_visuals. Skipping frame.");
        return;
    };
    for (tabs_entity, tabs_state) in tabs_query.iter_mut() {
        info!(
            "Updating visuals for TabsContainer {:?}, new active value: {:?}",
            tabs_entity, tabs_state.active_value
        );
        // 1. Aktualisiere alle Trigger, die zu diesem TabsContainer gehören
        for (mut trigger, mut bg_color, mut border_color, children) in triggers_query.iter_mut() {
            if trigger.parent_tabs == tabs_entity {
                trigger.is_active = trigger.value == tabs_state.active_value;

                let target_text_color; // Farbe für den Text im Trigger

                // Setze das Aussehen des Triggers
                if trigger.is_active && !trigger.disabled {
                    // Aktiver Stil: Heller Hintergrund, "echte" Textfarbe
                    *bg_color = BackgroundColor(theme.background); // Hintergrund aus dem *Haupt*-Theme
                    *border_color = BorderColor(Color::NONE); // Kein expliziter Rand im aktiven Zustand
                    target_text_color = theme.foreground; // Helle/normale Textfarbe

                // Optional: Shadow-Effekt (schwierig in Bevy UI nativ)
                } else {
                    // Inaktiver Stil (wie Ghost Button)
                    *bg_color = BackgroundColor(Color::NONE); // Kein Hintergrund
                    *border_color = BorderColor(Color::NONE); // Kein Rand
                    target_text_color = theme.muted_foreground; // Gedämpfte Textfarbe
                }

                // Farbe des Trigger-Texts aktualisieren
                if let Some(children) = children {
                    for &child in children.iter() {
                        if let Ok(mut text_color) = text_query.get_mut(child) {
                            // Update color for all sections in the text
                            *text_color = TextColor(target_text_color);
                        }
                    }
                }

                // Deaktivierte Trigger immer ausgrauen
                if trigger.disabled {
                    *bg_color = BackgroundColor(Color::NONE);
                    *border_color = BorderColor(Color::NONE);
                    // Textfarbe weiter ausgrauen (durch Alfa)
                    if let Some(children) = children {
                        for &child in children.iter() {
                            if let Ok(mut text_color) = text_query.get_mut(child) {
                                // Update color for all sections in the text
                                *text_color = TextColor(target_text_color.with_alpha(0.5));
                            }
                        }
                    }
                }
            }
        }

        // 2. Aktualisiere die Sichtbarkeit aller Content-Bereiche
        for (content, mut visibility) in contents_query.iter_mut() {
            // Zugehörigkeit prüfen? Geht implizit, da wir nur bei Zustandsänderung des Containers hier sind.
            // Ist der Wert dieses Contents der aktive Wert?
            if content.value == tabs_state.active_value {
                *visibility = Visibility::Inherited;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
