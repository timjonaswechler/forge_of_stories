// crates/forge_ui/src/layout.rs

use super::theme::UiTheme;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*; // Optional, falls Theme Styling beeinflusst

// --- Marker Komponenten ---

/// Marker für einen vertikalen Stack-Container.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct VerticalStack;

/// Marker für einen horizontalen Stack-Container.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct HorizontalStack;

/// Marker für eine Top-Level UI-Wurzel (optional, aber gute Konvention).
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct UiRoot;

// --- Builder für Stacks ---

/// Gemeinsame Basis für Stack-Builder
pub struct StackBuilderBase {
    justify_content: JustifyContent, // Hauptachse
    align_items: AlignItems,         // Querachse
    gap: Val,                        // Abstand zwischen Elementen
    padding: UiRect,                 // Innerer Abstand
    margin: UiRect,                  // Äußerer Abstand
    width: Option<Val>,              // Explizite Breite
    height: Option<Val>,             // Explizite Höhe
    background_color: Option<Color>,
}

impl Default for StackBuilderBase {
    fn default() -> Self {
        Self {
            justify_content: JustifyContent::FlexStart, // Standard: Oben/Links starten
            align_items: AlignItems::Stretch,           // Standard: Strecken auf Querachse
            gap: Val::Px(0.0),                          // Standard: Kein Abstand
            padding: UiRect::default(),
            margin: UiRect::default(),
            width: None,            // Standard: Auto
            height: None,           // Standard: Auto
            background_color: None, // Standard: Transparent
        }
    }
}

// --- Vertical Stack Builder ---

pub struct VerticalStackBuilder {
    base: StackBuilderBase,
    children: Vec<Box<dyn FnOnce(&mut ChildBuilder) + Send + Sync>>,
}

impl Default for VerticalStackBuilder {
    fn default() -> Self {
        let mut base = StackBuilderBase::default();
        // Sinnvolle Defaults für Vertical Stack überschreiben
        base.align_items = AlignItems::Center; // Zentriert horizontal, oft gewünscht
        Self {
            base,
            children: Vec::new(),
        }
    }
}

impl VerticalStackBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Setzt die Ausrichtung der Elemente entlang der Hauptachse (vertikal).
    pub fn justify(mut self, justify: JustifyContent) -> Self {
        self.base.justify_content = justify;
        self
    }
    /// Setzt die Ausrichtung der Elemente entlang der Querachse (horizontal).
    pub fn align(mut self, align: AlignItems) -> Self {
        self.base.align_items = align;
        self
    }
    /// Setzt den vertikalen Abstand zwischen den Elementen.
    pub fn gap(mut self, gap: Val) -> Self {
        self.base.gap = gap;
        self
    }
    /// Setzt das Padding für alle Seiten.
    pub fn padding(mut self, padding: Val) -> Self {
        self.base.padding = UiRect::all(padding);
        self
    }
    /// Setzt das Padding spezifisch.
    pub fn padding_rect(mut self, padding: UiRect) -> Self {
        self.base.padding = padding;
        self
    }
    /// Setzt das Margin für alle Seiten.
    pub fn margin(mut self, margin: Val) -> Self {
        self.base.margin = UiRect::all(margin);
        self
    }
    /// Setzt das Margin spezifisch.
    pub fn margin_rect(mut self, margin: UiRect) -> Self {
        self.base.margin = margin;
        self
    }
    /// Setzt eine feste Breite.
    pub fn width(mut self, width: Val) -> Self {
        self.base.width = Some(width);
        self
    }
    /// Setzt eine feste Höhe.
    pub fn height(mut self, height: Val) -> Self {
        self.base.height = Some(height);
        self
    }
    /// Setzt eine Hintergrundfarbe.
    pub fn background(mut self, color: Color) -> Self {
        self.base.background_color = Some(color);
        self
    }

    /// Fügt ein Kind-Element hinzu, das durch eine Closure gespawnt wird.
    /// Die Closure erhält einen `&mut ChildBuilder`.
    pub fn add(
        mut self,
        child_builder: impl FnOnce(&mut ChildBuilder) + Send + Sync + 'static,
    ) -> Self {
        self.children.push(Box::new(child_builder));
        self
    }

    /// Spawnt den vertikalen Stack und seine Kinder.
    #[must_use]
    pub fn spawn<'w, 'a>(self, parent: &'a mut ChildBuilder<'w>) -> EntityCommands<'a> {
        let mut style = Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column, // <<< Vertikal
            justify_content: self.base.justify_content,
            align_items: self.base.align_items,
            row_gap: self.base.gap, // <<< row_gap für Vertikal
            padding: self.base.padding,
            margin: self.base.margin,
            ..default()
        };
        if let Some(w) = self.base.width {
            style.width = w;
        }
        if let Some(h) = self.base.height {
            style.height = h;
        }

        let mut cmd = parent.spawn((
            VerticalStack, // Marker
            Node { ..style },
            BackgroundColor(self.base.background_color.map_or(Color::NONE, |c| c)),
        ));

        // Kinder hinzufügen
        cmd.with_children(|builder| {
            for child_fn in self.children {
                (child_fn)(builder);
            }
        });

        cmd
    }
}

// --- Horizontal Stack Builder ---

pub struct HorizontalStackBuilder {
    base: StackBuilderBase,
    children: Vec<Box<dyn FnOnce(&mut ChildBuilder) + Send + Sync>>,
}

impl Default for HorizontalStackBuilder {
    fn default() -> Self {
        let mut base = StackBuilderBase::default();
        // Sinnvolle Defaults für Horizontal Stack überschreiben
        base.align_items = AlignItems::Center; // Zentriert vertikal, oft gewünscht
        base.justify_content = JustifyContent::FlexStart; // Standard: linksbündig
        Self {
            base,
            children: Vec::new(),
        }
    }
}

impl HorizontalStackBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Setzt die Ausrichtung der Elemente entlang der Hauptachse (horizontal).
    pub fn justify(mut self, justify: JustifyContent) -> Self {
        self.base.justify_content = justify;
        self
    }
    /// Setzt die Ausrichtung der Elemente entlang der Querachse (vertikal).
    pub fn align(mut self, align: AlignItems) -> Self {
        self.base.align_items = align;
        self
    }
    /// Setzt den horizontalen Abstand zwischen den Elementen.
    pub fn gap(mut self, gap: Val) -> Self {
        self.base.gap = gap;
        self
    }
    /// Setzt das Padding für alle Seiten.
    pub fn padding(mut self, padding: Val) -> Self {
        self.base.padding = UiRect::all(padding);
        self
    }
    /// Setzt das Padding spezifisch.
    pub fn padding_rect(mut self, padding: UiRect) -> Self {
        self.base.padding = padding;
        self
    }
    /// Setzt das Margin für alle Seiten.
    pub fn margin(mut self, margin: Val) -> Self {
        self.base.margin = UiRect::all(margin);
        self
    }
    /// Setzt das Margin spezifisch.
    pub fn margin_rect(mut self, margin: UiRect) -> Self {
        self.base.margin = margin;
        self
    }
    /// Setzt eine feste Breite.
    pub fn width(mut self, width: Val) -> Self {
        self.base.width = Some(width);
        self
    }
    /// Setzt eine feste Höhe.
    pub fn height(mut self, height: Val) -> Self {
        self.base.height = Some(height);
        self
    }
    /// Setzt eine Hintergrundfarbe.
    pub fn background(mut self, color: Color) -> Self {
        self.base.background_color = Some(color);
        self
    }

    /// Fügt ein Kind-Element hinzu, das durch eine Closure gespawnt wird.
    pub fn add(
        mut self,
        child_builder: impl FnOnce(&mut ChildBuilder) + Send + Sync + 'static,
    ) -> Self {
        self.children.push(Box::new(child_builder));
        self
    }

    /// Spawnt den horizontalen Stack und seine Kinder.
    #[must_use]
    pub fn spawn<'w, 'a>(self, parent: &'a mut ChildBuilder<'w>) -> EntityCommands<'a> {
        let mut style = Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row, // <<< Horizontal
            justify_content: self.base.justify_content,
            align_items: self.base.align_items,
            column_gap: self.base.gap, // <<< column_gap für Horizontal
            padding: self.base.padding,
            margin: self.base.margin,
            ..default()
        };
        if let Some(w) = self.base.width {
            style.width = w;
        }
        if let Some(h) = self.base.height {
            style.height = h;
        }

        let mut cmd = parent.spawn((
            HorizontalStack, // Marker
            Node { ..style },
            BackgroundColor(self.base.background_color.map_or(Color::NONE, |c| c)),
        ));

        // Kinder hinzufügen
        cmd.with_children(|builder| {
            for child_fn in self.children {
                (child_fn)(builder);
            }
        });

        cmd
    }
}

// --- Funktion zum Spawnen einer Standard-UI-Wurzel ---

// --- Funktion zum Spawnen einer Standard-UI-Wurzel ---
impl UiRoot {
    // <<<< Impelentation Block für UiRoot
    /// Spawnt eine Top-Level UI-Node, die den gesamten Bildschirm ausfüllt
    /// und als Container für die restliche UI dient. Enthält den `UiRoot` Marker.
    /// Nützlich, um z.B. einen globalen Hintergrund oder Padding zu setzen
    /// und einen definierten Parent für Overlay-Elemente wie Dialoge zu haben.
    pub fn spawn(commands: &mut Commands, theme: &UiTheme) -> Entity {
        // <<<< Jetzt assoziierte Funktion
        commands
            .spawn((
                UiRoot, // Marker hinzufügen
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    // Standardmäßig zentriert und vertikal? Oder flex-start?
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(theme.layout.padding.base)),
                    ..default()
                },
                BackgroundColor(theme.color.tomato.solid_primary), // Beispielhintergrundfarbe
            ))
            .id() // Gibt die Entity ID der Wurzel zurück
    }
}
