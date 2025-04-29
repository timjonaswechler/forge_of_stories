use super::components::{ButtonMarker, ButtonState, OnClick};
use super::enums::{ButtonChild, ButtonSize, ButtonVariant};

use super::style::{apply_size_style, base_style, get_button_style_def};
use crate::theme::UiTheme;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

// ======== Builder ========
pub struct ButtonBuilder<F: Fn() + Send + Sync + 'static = fn()> {
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    children_defs: Vec<ButtonChild>,
    on_click: Option<OnClick<F>>,
    width: Option<Val>,
    height: Option<Val>,
    border_radius: Option<Val>,
    markers: Vec<Box<dyn FnOnce(&mut EntityCommands) + Send + Sync>>,
}

// --- impl Clone, Default, new ---
impl<F: Fn() + Send + Sync + 'static + Clone> Clone for ButtonBuilder<F> {
    fn clone(&self) -> Self {
        Self {
            variant: self.variant.clone(),
            size: self.size.clone(),
            disabled: self.disabled.clone(),
            children_defs: self.children_defs.clone(),
            on_click: self.on_click.clone(),
            width: self.width.clone(),
            height: self.height.clone(),
            border_radius: self.border_radius.clone(),
            markers: Vec::new(), // cannot clone FnOnce closures, so start fresh
        }
    }
}

impl Default for ButtonBuilder<fn()> {
    fn default() -> Self {
        Self {
            variant: ButtonVariant::Default,
            size: ButtonSize::Default,
            disabled: false,
            children_defs: Vec::new(),
            on_click: None,
            width: None,
            height: None,
            border_radius: None,
            markers: Vec::new(),
        }
    }
}

impl ButtonBuilder<fn()> {
    pub fn new() -> Self {
        Self::default()
    }
}

// --- Methoden ---
impl<F: Fn() + Send + Sync + 'static> ButtonBuilder<F> {
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

    pub fn width(mut self, width: Val) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: Val) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_icon(mut self, icon_handle: Handle<Image>) -> Self {
        if self.size == ButtonSize::Icon {
            self.children_defs.clear();
        }
        self.children_defs.push(ButtonChild::Icon(icon_handle));
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        if self.size != ButtonSize::Icon {
            self.children_defs.push(ButtonChild::Text(text.into()));
        }
        self
    }

    pub fn on_click<NewF: Fn() + Send + Sync + 'static>(
        self,
        callback: NewF,
    ) -> ButtonBuilder<NewF> {
        ButtonBuilder {
            variant: self.variant,
            size: self.size,
            disabled: self.disabled,
            children_defs: self.children_defs,
            on_click: Some(OnClick::new(callback)),
            width: self.width,
            height: self.height,
            border_radius: self.border_radius,
            markers: self.markers,
        }
    }
    pub fn border_radius(mut self, radius_px: f32) -> Self {
        self.border_radius = Some(Val::Px(radius_px)); // <<< Konvertiere f32 zu Val::Px
        self
    }

    /// Sets the border radius for all corners using a `Val` (e.g., `Val::Px(5.0)`, `Val::Percent(50.0)`).
    pub fn border_radius_val(mut self, radius: Val) -> Self {
        self.border_radius = Some(radius);
        self
    }

    pub fn mark<M: Bundle + Default>(self) -> Self
    where
        M: Send + Sync + 'static,
    {
        self.add_marker(|ec| {
            ec.insert(M::default());
        })
    }

    pub fn add_marker(
        mut self,
        func: impl FnOnce(&mut EntityCommands) + Send + Sync + 'static,
    ) -> Self {
        self.markers.push(Box::new(func));
        self
    }

    #[must_use]
    pub fn spawn<'w, 'a>(
        self,
        parent: &'a mut ChildBuilder<'w>,
        theme: &UiTheme,            // <<< Reihenfolge angepasst
        font_handle: &Handle<Font>, // <<< Name und Reihenfolge angepasst
    ) -> EntityCommands<'a> {
        // 1. Style Def holen
        let style_def = get_button_style_def(self.variant, theme);

        // 2. Basis-Style vorbereiten und Grössen-Style anwenden
        let mut node_style = base_style();
        let font_size = apply_size_style(&mut node_style, self.size, theme);

        // Setze BorderRadius *nach* apply_size_style, falls Icon es überschreiben soll
        // node_style.border_radius = BorderRadius::all(Val::Px(theme.layout.radius.base));

        // 3. Width/Height Overrides
        if let Some(w) = self.width {
            node_style.width = w;
        }
        if let Some(h) = self.height {
            node_style.height = h;
            if let (Val::Px(current), Val::Px(h_px)) = (node_style.min_height, h) {
                if current < h_px {
                    node_style.min_height = h;
                }
            }
        }
        let final_radius = self
            .border_radius
            .unwrap_or_else(|| Val::Px(theme.layout.radius.base)); // <<< Radius Logik

        // 4. Button Entity spawnen
        let mut cmd = parent.spawn((
            Button,                                                 // Bevy Komponente
            Node { ..node_style },                                  // Angewendeter Style
            style_def.background(Interaction::None, self.disabled), // Initialer Hintergrund
            style_def.border(Interaction::None, self.disabled),     // Initialer Rand
            if self.disabled {
                FocusPolicy::Pass
            } else {
                FocusPolicy::Block
            },
            Interaction::default(),
            // Custom Components
            ButtonMarker,
            ButtonState {
                variant: self.variant,
                size: self.size,
                disabled: self.disabled,
                style_def: style_def.clone(),
            },
            BorderRadius::all(final_radius), // <<< Setze den BorderRadius
        ));

        // 5. OnClick Komponente
        if let Some(on_click_component) = self.on_click {
            cmd.insert(on_click_component);
        }

        // 6. Marker anwenden
        for marker_fn in self.markers {
            marker_fn(&mut cmd);
        }

        // 7. Kinder spawnen
        cmd.with_children(|cb| {
            for child_def in self.children_defs {
                match child_def {
                    ButtonChild::Text(text) => {
                        // Hole die korrekte initiale Textfarbe (ggf. Disabled)
                        let initial_text_color = style_def.text_color(self.disabled); // <<< Neu: text_color Methode nutzen
                        cb.spawn((
                            Text::new(text), // Besser als TextBundle hier, wenn TextColor separat gesetzt wird
                            TextFont {
                                font: font_handle.clone(),
                                font_size,
                                ..default()
                            },
                            TextColor(initial_text_color), // <<< Setze angepasste Farbe
                        ));
                    }
                    ButtonChild::Icon(handle) => {
                        let icon_size_val = match self.size {
                            ButtonSize::Small => theme.font.font_size.sm,
                            ButtonSize::Icon => theme.font.font_size.lg,
                            _ => theme.font.font_size.base,
                        };
                        let icon_size = Val::Px(icon_size_val);
                        // Hole die korrekte initiale Icon-Tint-Farbe (ggf. Disabled)
                        let initial_icon_tint = style_def.text_color(self.disabled);

                        cb.spawn((
                            Node {
                                width: icon_size,
                                height: icon_size,
                                margin: UiRect::axes(Val::Px(0.0), Val::Px(2.0)),
                                ..default()
                            },
                            ImageNode {
                                image: handle,
                                ..default()
                            },
                            BackgroundColor(initial_icon_tint),
                            FocusPolicy::Pass,
                            Visibility::Inherited,
                        ))
                        .insert(BackgroundColor(
                            style_def
                                .text_color
                                .with_alpha(if self.disabled { 0.6 } else { 1.0 }),
                        )); // Start tinted like text color
                    }
                }
            }
        });

        cmd
    }
}
