// crates/forge_ui/src/dialog.rs

use bevy::audio::*;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use uuid::Uuid;

use std::hash::Hash;

use super::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use super::theme::UiTheme; // Für Schließen-Button

// --- Ressourcen (ggf. global definieren) ---
#[derive(Resource, Default, Debug)]
pub struct ActiveDialogs {
    pub current_modal: Option<Entity>,
}
struct OpenDialogSound(Handle<AudioSource>);

#[derive(Bundle, Clone, Default)]
pub struct DialogContentStyle {
    pub node: Node,
    pub border_radius: BorderRadius,
    pub transform: Transform,
    pub background_color: BackgroundColor,
    // Weitere Style-Komponenten nach Bedarf
}
// --- Komponenten ---

/// Eindeutige ID für eine Dialog-Instanz.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DialogId(pub Uuid);

impl DialogId {
    pub fn new_unique() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Marker für den **Wurzelknoten** eines Dialogs. Dieser Knoten wird sichtbar/unsichtbar geschaltet.
/// Enthält die eindeutige ID.
#[derive(Component, Debug, Clone)]
pub struct DialogRoot {
    pub id: DialogId,
    pub initially_open: bool, // Startet der Dialog offen?
    pub modal: bool,          // Blockiert er die UI dahinter?
}

/// Marker für das Overlay (der halbtransparente Hintergrund).
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct DialogOverlay;

/// Marker für den sichtbaren Content-Bereich des Dialogs.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct DialogContent;

/// Marker für einen Button, der den aktuellen Dialog schließen soll.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct DialogCloseTrigger;

// Optional: Titel und Beschreibung könnten auch eigene Marker bekommen,
// aber oft reicht es, sie als Text im Content zu platzieren.
// #[derive(Component, Default)] pub struct DialogTitle;
// #[derive(Component, Default)] pub struct DialogDescription;

// --- Events ---

/// Event zum Öffnen eines Dialogs über seine ID.
#[derive(Event, Debug, Clone)]
pub struct OpenDialogEvent(pub DialogId);

/// Event zum Schließen des aktuell offenen modalen Dialogs (oder eines spezifischen?).
#[derive(Event, Debug, Clone)]
pub struct CloseDialogEvent {
    // Optional: ID zum gezielten Schließen (falls nicht-modal erlaubt wird)
    // pub id_to_close: Option<DialogId>
}

// --- Builder ---

// Typ für die Funktion, die den Dialog-Inhalt baut
type DialogContentBuilderFn =
    Box<dyn FnOnce(&mut ChildBuilder, &UiTheme, &Handle<Font>) + Send + Sync>;

pub struct DialogBuilder {
    id: DialogId,
    modal: bool,
    initially_open: bool,
    content_builder: Option<DialogContentBuilderFn>,
    // Styling-Optionen?
    width: Option<Val>,
    height: Option<Val>,
    overlay_color: Option<Color>,
    // Titel/Beschreibung optional als direkte Strings im Builder?
    title: Option<String>,
    description: Option<String>,
    // Standardmäßig einen Schließen-Button hinzufügen?
    show_default_close_button: bool,
}

impl DialogBuilder {
    pub fn new(id: DialogId) -> Self {
        Self {
            id,
            modal: true, // Standardmäßig modal
            initially_open: false,
            content_builder: None,
            width: Some(Val::Px(400.0)), // Standardbreite
            height: None,                // Auto-Höhe
            overlay_color: None,         // Wird vom Theme geholt
            title: None,
            description: None,
            show_default_close_button: true, // Standard: X oben rechts
        }
    }

    pub fn non_modal(mut self) -> Self {
        self.modal = false;
        self
    }

    pub fn initially_open(mut self) -> Self {
        self.initially_open = true;
        self
    }

    /// Definiert den Inhalt des Dialogs über eine Closure.
    pub fn with_content(
        mut self,
        builder: impl FnOnce(&mut ChildBuilder, &UiTheme, &Handle<Font>) + Send + Sync + 'static,
    ) -> Self {
        self.content_builder = Some(Box::new(builder));
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn hide_default_close_button(mut self) -> Self {
        self.show_default_close_button = false;
        self
    }

    pub fn width(mut self, width: Val) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: Val) -> Self {
        self.height = Some(height);
        self
    }

    /// Spawnt den kompletten Dialog (Overlay + Content).
    /// Der Dialog wird normalerweise **nicht** direkt als Kind eines anderen UI-Elements
    /// gespawnt, sondern auf oberster Ebene (z.B. als Kind der Root-UI-Node oder
    /// einer dedizierten Overlay-Node), um das Z-Index-Problem zu umgehen.
    #[must_use]
    pub fn spawn<'w, 'a>(
        self,
        commands: &'a mut Commands, // Braucht Commands, da oft Top-Level
        theme: &UiTheme,
        font_handle: &Handle<Font>,
        // Optional: Icons für Default Close Button übergeben
        close_icon: Option<&Handle<Image>>,
        // Optional: Eine feste Parent-Entity für alle Dialoge
        dialog_container: Option<Entity>,
    ) -> EntityCommands<'a> {
        let overlay_bg = self.overlay_color.unwrap_or(Color::BLACK.with_alpha(0.6));

        // Styling für den Content-Bereich
        let mut content_style = DialogContentStyle {
            node: Node {
                position_type: PositionType::Absolute, // Wird im Overlay zentriert
                // Positionierung in der Mitte des Overlays
                left: Val::Percent(50.),
                top: Val::Percent(50.),

                // Innere Gestaltung
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(theme.layout.padding.base)), // Padding für den Content
                width: self.width.unwrap_or(Val::Px(400.0)),              // Direkt setzen
                height: self.height.unwrap_or_default(),                  // Direkt setzen
                row_gap: Val::Px(12.0), // Abstand zwischen Elementen im Dialog

                ..default()
            },
            border_radius: BorderRadius::all(Val::Px(theme.layout.radius.base)),
            background_color: BackgroundColor(theme.color.gray.background_secondary), // Popover-Farbe für Content
            // Transform zum Zentrieren verwenden
            // Verschiebe um -50% der *eigenen* Breite/Höhe nach links/oben
            transform: Transform::from_translation(Vec3::new(-50., -50., 0.)),
        };
        if let Some(w) = self.width {
            content_style.node.width = w;
        }
        if let Some(h) = self.height {
            content_style.node.height = h;
        }

        let mut root_node_visibility = Visibility::Hidden; // Standardmäßig versteckt
        if self.initially_open {
            root_node_visibility = Visibility::Visible;
            // TODO: Muss beim Start in ActiveDialogs registriert werden (in einem System?)
        }

        // --- Spawnen des Dialog-Wurzelknotens ---
        // Dieser Knoten kontrolliert die Sichtbarkeit und enthält Overlay + Content
        let mut root_entity_commands = commands.spawn((
            DialogRoot {
                id: self.id.clone(),
                initially_open: self.initially_open,
                modal: self.modal,
            },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.),
                top: Val::Px(0.),
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                // Zentriert Content per Flexbox (im Overlay)
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            // Wird nur sichtbar geschaltet, wenn der Dialog offen ist
            root_node_visibility,
            // Z-Index SEHR WICHTIG: Muss über anderer UI liegen
            GlobalZIndex(10), // Beispielwert, ggf. anpassen
        ));

        let root_entity_id = root_entity_commands.id();

        // Optional: Dialog als Kind eines speziellen Containers spawnen
        let mut pending_parent: Option<Entity> = None;
        if let Some(container) = dialog_container {
            pending_parent = Some(container);
        }

        // --- Kinder des Wurzelknotens: Overlay und Content ---
        root_entity_commands.with_children(|builder| {
            // 1. Overlay spawnen (wenn modal)
            if self.modal {
                builder.spawn((
                    DialogOverlay,
                    Node {
                        position_type: PositionType::Absolute, // Über Content
                        left: Val::Px(0.),
                        top: Val::Px(0.),
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        ..default()
                    },
                    BackgroundColor(overlay_bg),
                    // Overlay bekommt niedrigeren Z-Index als Content
                    GlobalZIndex(9),
                    FocusPolicy::Block,
                ));
            }

            // 2. Content-Node spawnen
            let mut content_cmd = builder.spawn((DialogContent, content_style));

            // Internen Content des Dialogs bauen
            content_cmd.with_children(|content_builder| {
                // A. Optionaler Standard-Titel und Beschreibung
                if let Some(title_text) = self.title {
                    content_builder.spawn((
                        Text::new(title_text),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 18.0, // Titelgröße
                            ..default()
                        },
                        TextColor(theme.color.white.text_primary), // Popover-Farbe
                    ));
                    // .insert(DialogTitle); // Optional Marker hinzufügen
                }
                if let Some(desc_text) = self.description {
                    content_builder
                        .spawn((
                            Text::new(desc_text),
                            TextFont {
                                font: font_handle.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(theme.color.white.solid_primary), // Gedämpft
                        ))
                        .insert(Node {
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        });
                    // .insert(DialogDescription); // Optional Marker hinzufügen
                }

                // B. Benutzerdefinierter Inhalt
                if let Some(builder_fn) = self.content_builder {
                    (builder_fn)(content_builder, theme, font_handle);
                }

                // C. Optionaler Standard-Schließen-Button
                if self.show_default_close_button {
                    // Button absolut oben rechts positionieren
                    content_builder
                        .spawn(Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(theme.layout.spacing), // Kleiner Abstand zum Rand
                            right: Val::Px(theme.layout.spacing),

                            ..default()
                        })
                        .with_children(|btn_parent| {
                            let mut close_button_builder = ButtonBuilder::new()
                                .size(ButtonSize::Icon)
                                .variant(ButtonVariant::Ghost) // Unauffällig// `maybe_with_icon` (hypothetisch) oder if-Abfrage
                                .add_marker(|cmd| {
                                    cmd.insert(DialogCloseTrigger);
                                }); // Wichtig!

                            // Wenn ein Icon übergeben wurde, verwenden Sie es, sonst Text "X"
                            if let Some(icon_handle) = close_icon {
                                close_button_builder =
                                    close_button_builder.with_icon(icon_handle.clone());
                            } else {
                                // Fallback, wenn kein Icon übergeben wurde
                                close_button_builder = close_button_builder.with_text("X");
                            }

                            // Button final spawnen
                            let _ =
                                close_button_builder.spawn(btn_parent, font_handle.clone(), theme);
                        });
                }
            });
        });
        // Nach dem Spawnen: Parent-Child-Beziehung setzen, um Borrow-Konflikte zu vermeiden
        let root_entity_commands_id = root_entity_commands.id();
        let result = root_entity_commands;

        // Gib den Borrow auf commands frei, bevor wir ihn erneut verwenden
        drop(result);

        if let Some(container) = pending_parent {
            commands
                .entity(container)
                .add_child(root_entity_commands_id);
        }

        commands.entity(root_entity_commands_id)
    }
}

// --- Systeme ---

/// System zum Öffnen eines Dialogs über Event.
pub fn open_dialog_system(
    mut commands: Commands,
    mut ev_open: EventReader<OpenDialogEvent>,
    mut q_dialogs: Query<(Entity, &DialogRoot, &mut Visibility)>,
    mut active_dialogs: ResMut<ActiveDialogs>,
    // audio: Option<Res<Audio>>, // Optional: Sound abspielen
    // Optional: Geladene Sound-Assets
) {
    for event in ev_open.read() {
        info!("Received OpenDialogEvent for ID: {:?}", event.0);
        let mut found_dialog = false;
        for (entity, root, mut visibility) in q_dialogs.iter_mut() {
            if root.id == event.0 {
                info!("  Found matching DialogRoot Entity: {:?}", entity);
                found_dialog = true;
                if *visibility == Visibility::Hidden {
                    info!("    Dialog was hidden, setting to Inherited.");
                    *visibility = Visibility::Inherited;
                    info!("Opened dialog: {:?}", root.id);

                    // Alten modalen schließen, falls ein neuer geöffnet wird
                    if root.modal {
                        if let Some(old_modal_entity) = active_dialogs.current_modal {
                            if old_modal_entity != entity {
                                // Finde den alten Dialog und verstecke ihn
                                if let Ok((_, _, mut old_vis)) = q_dialogs.get_mut(old_modal_entity)
                                {
                                    *old_vis = Visibility::Hidden;
                                    info!("Closed previous modal dialog implicitly.");
                                }
                            }
                        }
                        active_dialogs.current_modal = Some(entity);
                    }

                    // Optional: Open Sound
                    // if let Some(audio_res) = audio { audio_res.play(...); }

                    // Fokus setzen? (Schwierig in Bevy UI)
                    // commands.entity(entity).insert(RequestFocus); // Hypothetische Komponente
                } else {
                    info!("    Dialog was already visible."); // <<< Log 4
                }
                break; // ID gefunden, Schleife beenden
            }
        }
        if !found_dialog {
            error!("  DialogRoot with ID {:?} not found in query!", event.0); // <<< Log 5
        }
    }
}

/// System zum Schließen des aktuell offenen (modalen) Dialogs.
pub fn close_dialog_system(
    // Trigger für dieses System: Event ODER Klick auf CloseButton ODER ESC
    mut ev_close: EventReader<CloseDialogEvent>,
    // Query für Close-Buttons, die geklickt wurden
    q_close_triggers: Query<
        &Interaction,
        (Changed<Interaction>, With<DialogCloseTrigger>, With<Button>),
    >,
    // Tastatureingabe (ESC)
    keyboard: Res<ButtonInput<KeyCode>>, // Neu: ButtonInput statt Input
    mut active_dialogs: ResMut<ActiveDialogs>,
    mut q_dialogs: Query<(Entity, &mut Visibility, &DialogRoot)>,
) {
    let mut close_requested = !ev_close.is_empty(); // Schließen durch Event?
    ev_close.clear(); // Events verarbeiten

    // Schließen durch Klick auf einen Close-Button?
    for interaction in q_close_triggers.iter() {
        if *interaction == Interaction::Pressed {
            // War vorher Clicked? Jetzt Pressed!
            close_requested = true;
            break;
        }
    }

    // Schließen durch ESC?
    if keyboard.just_pressed(KeyCode::Escape) {
        close_requested = true;
    }

    if close_requested {
        if let Some(modal_entity) = active_dialogs.current_modal.take() {
            // Nimmt ID und setzt auf None
            if let Ok((_, mut visibility, _)) = q_dialogs.get_mut(modal_entity) {
                if *visibility != Visibility::Hidden {
                    *visibility = Visibility::Hidden;
                    info!("Closed modal dialog.");
                    // TODO: Fokus zurück zum Trigger setzen (schwierig)
                }
            }
        } else if close_requested {
            warn!("Close requested, but no modal dialog was active.");
        }
    }
}

/// System, das Klicks auf das Overlay abfängt (nur wenn modal).
/// Wenn ein modaler Dialog offen ist, schließt ein Klick auf das Overlay den Dialog.
pub fn handle_overlay_click_system(
    q_overlays: Query<(Entity, &Interaction, &Parent), (Changed<Interaction>, With<DialogOverlay>)>,
    q_dialog_root: Query<&DialogRoot>, // Um Modalität zu prüfen
    mut ev_close: EventWriter<CloseDialogEvent>,
) {
    for (_overlay_entity, interaction, parent) in q_overlays.iter() {
        if *interaction == Interaction::Pressed {
            // Oder Clicked, je nach Bevy Version/Verhalten
            // Finde den DialogRoot des Overlays
            if let Ok(dialog_root) = q_dialog_root.get(parent.get()) {
                // Nur schließen, wenn modal und geklickt
                if dialog_root.modal {
                    info!("Overlay clicked, sending CloseDialogEvent.");
                    ev_close.send(CloseDialogEvent {});
                }
            }
        }
    }
}

// Optional: System, das verhindert, dass hinter einem modalen Dialog geklickt wird
// (kann komplex sein, einfacher ist oft, Klicks auf Overlay zum Schließen zu nutzen)

// --- System zum initialen Registrieren von `initially_open` Dialogen ---
pub fn register_initially_open_dialogs(
    mut active_dialogs: ResMut<ActiveDialogs>,
    query: Query<(Entity, &DialogRoot), Added<DialogRoot>>, // Nur neue Dialoge
) {
    for (entity, root) in query.iter() {
        if root.initially_open && root.modal {
            if let Some(old_modal) = active_dialogs.current_modal {
                warn!("Multiple initially open modal dialogs detected! Only the last one registered ({:?}) will be active. Previous: {:?}", entity, old_modal);
            }
            active_dialogs.current_modal = Some(entity);
            info!("Registered initially open modal dialog: {:?}", root.id);
        }
        // TODO: Handle initially open non-modal dialogs if needed
    }
}
