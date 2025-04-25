// ############ crates/forge_ui/src/button.rs ############
// (Replaces your existing forge_ui/src/button.rs)
// Contains ButtonBuilder, ButtonVariant, ButtonSize, ButtonStyleDef, ButtonState,
// OnClick (optional), ButtonClickedEvent, ButtonMarker, ButtonChild,
// get_button_style_def, base_style, apply_size_style, spawn method,
// update_button_visuals system, handle_button_clicks_event system.

use crate::theme::UiTheme;
use bevy::{color::*, ecs::system::EntityCommands, prelude::*, ui::FocusPolicy};
use std::marker::PhantomData;
// ======== Varianten-System ========

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Component)]
pub enum ButtonVariant {
    #[default]
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Component)]
pub enum ButtonSize {
    #[default]
    Default,
    Small,
    Large,
    Icon, // Represents a square button intended primarily for an icon
}

// A structure to encapsulate theme values for a specific button variant
#[derive(Debug, Clone)]
pub struct ButtonStyleDef {
    pub background_color: Color,
    pub text_color: Color,
    pub border_color: Color,
    pub hovered_background_factor: f32, // e.g., 0.15 for lighter, -0.15 for darker
    pub pressed_background_factor: f32, // e.g., 0.15 for lighter, -0.15 for darker
    pub hovered_border_factor: f32,     // Optional: Border color adjustment on hover
    pub pressed_border_factor: f32,     // Optional: Border color adjustment on press
}

impl ButtonStyleDef {
    // Calculates BACKGROUND color based on interaction and disabled state
    pub fn background(&self, interaction: Interaction, disabled: bool) -> BackgroundColor {
        if disabled {
            return BackgroundColor(self.background_color.darker(0.4).with_alpha(0.6));
        }
        match interaction {
            Interaction::Hovered => {
                if self.hovered_background_factor >= 0.0 {
                    BackgroundColor(
                        self.background_color
                            .lighter(self.hovered_background_factor),
                    )
                } else {
                    BackgroundColor(
                        self.background_color
                            .darker(-self.hovered_background_factor),
                    )
                }
            }
            Interaction::Pressed => {
                if self.pressed_background_factor >= 0.0 {
                    BackgroundColor(
                        self.background_color
                            .lighter(self.pressed_background_factor),
                    )
                } else {
                    BackgroundColor(
                        self.background_color
                            .darker(-self.pressed_background_factor),
                    )
                }
            }
            Interaction::None => BackgroundColor(self.background_color),
        }
    }

    // Calculates BORDER color based on interaction and disabled state
    pub fn border(&self, interaction: Interaction, disabled: bool) -> BorderColor {
        let mut final_border_color = self.border_color;

        if disabled {
            final_border_color = self.border_color.with_alpha(0.6);
        } else if final_border_color != Color::NONE {
            // Only apply factors if there's a base border
            match interaction {
                Interaction::Hovered => {
                    if self.hovered_border_factor > 0.0 {
                        final_border_color = final_border_color.lighter(self.hovered_border_factor);
                    } else if self.hovered_border_factor < 0.0 {
                        final_border_color = final_border_color.darker(-self.hovered_border_factor);
                    }
                }
                Interaction::Pressed => {
                    if self.pressed_border_factor > 0.0 {
                        final_border_color = final_border_color.lighter(self.pressed_border_factor);
                    } else if self.pressed_border_factor < 0.0 {
                        final_border_color = final_border_color.darker(-self.pressed_border_factor);
                    }
                }
                Interaction::None => {}
            }
        }
        BorderColor(final_border_color)
    }
}

// TODO: In the future, this function should take `theme: &UiTheme`
fn get_button_style_def(variant: ButtonVariant, theme: &UiTheme) -> ButtonStyleDef {
    match variant {
        ButtonVariant::Default => ButtonStyleDef {
            // Example using direct colors - replace with theme.primary, theme.primary_foreground etc.
            background_color: theme.primary, // Replace with theme.primary
            text_color: theme.primary_foreground, // Replace with theme.primary_foreground
            border_color: theme.border,      // Replace with theme.border (or NONE)
            hovered_background_factor: 0.15, // Lighter
            pressed_background_factor: -0.05, // Slightly darker
            hovered_border_factor: 0.0,
            pressed_border_factor: 0.0,
        },
        ButtonVariant::Destructive => ButtonStyleDef {
            background_color: theme.destructive, // Replace with theme.destructive
            text_color: theme.destructive_foreground, // Replace with theme.destructive_foreground
            border_color: theme.border,          // Replace with theme.border (or NONE)
            hovered_background_factor: 0.15,
            pressed_background_factor: -0.05,
            hovered_border_factor: 0.0,
            pressed_border_factor: 0.0,
        },
        ButtonVariant::Outline => ButtonStyleDef {
            background_color: Color::NONE, // Replace with theme.background or theme.card
            text_color: theme.primary_foreground, // Replace with theme.foreground or theme.card_foreground
            border_color: theme.border,           // Replace with theme.border
            hovered_background_factor: -0.8,      // Makes a transparent BG slightly opaque gray
            pressed_background_factor: -0.7,      // Slightly more opaque gray
            hovered_border_factor: 0.3,           // Lighter border
            pressed_border_factor: -0.1,          // Darker border
        },
        ButtonVariant::Secondary => ButtonStyleDef {
            background_color: theme.secondary, // Replace with theme.secondary
            text_color: theme.secondary_foreground, // Replace with theme.secondary_foreground
            border_color: Color::NONE,
            hovered_background_factor: 0.15,
            pressed_background_factor: -0.05,
            hovered_border_factor: 0.0,
            pressed_border_factor: 0.0,
        },
        ButtonVariant::Ghost => ButtonStyleDef {
            background_color: Color::NONE, // Replace with theme.background / card
            text_color: theme.primary_foreground,
            border_color: Color::NONE,
            hovered_background_factor: -0.9, // Very subtle background hover
            pressed_background_factor: -0.85, // Slightly more background on press
            hovered_border_factor: 0.0,
            pressed_border_factor: 0.0,
        },
        ButtonVariant::Link => ButtonStyleDef {
            background_color: Color::NONE,
            text_color: theme.primary_foreground,
            border_color: Color::NONE,
            hovered_background_factor: 0.0, // Link hover is usually text decoration
            pressed_background_factor: 0.0,
            hovered_border_factor: 0.0,
            pressed_border_factor: 0.0,
            // TODO: Add text decoration handling for links in the update system?
        },
    }
}

// --- Base styling function for the Node ---
fn base_style() -> Node {
    Node {
        display: Display::Flex,                  // Use Flexbox for layout
        justify_content: JustifyContent::Center, // Center items horizontally
        align_items: AlignItems::Center,         // Center items vertically
        column_gap: Val::Px(8.0),                // Space between icon and text
        ..default()
    }
}

// --- Function to apply size-specific styling ---
// Modifies Style and returns the font size
fn apply_size_style(style: &mut Node, size: ButtonSize) -> f32 {
    // Apply default radius (could later come from theme)
    // style.border = UiRect::all(Val::Px(6.0)); // This sets *width*, need BorderRadius for rounding

    match size {
        ButtonSize::Default => {
            style.min_height = Val::Px(36.);
            style.padding = UiRect::axes(Val::Px(16.), Val::Px(8.)); // Horizontal, Vertical
            14.0 // font size
        }
        ButtonSize::Small => {
            style.min_height = Val::Px(32.);
            style.padding = UiRect::axes(Val::Px(12.), Val::Px(4.));
            12.0 // font size
        }
        ButtonSize::Large => {
            style.min_height = Val::Px(40.);
            style.padding = UiRect::axes(Val::Px(32.), Val::Px(8.));
            16.0 // font size
        }
        ButtonSize::Icon => {
            // Square button
            style.width = Val::Px(36.);
            style.height = Val::Px(36.);
            style.padding = UiRect::all(Val::Px(0.0)); // Let icon fill the space
                                                       // Ensure icon is centered if it's smaller than the button
            style.justify_content = JustifyContent::Center;
            style.align_items = AlignItems::Center;
            // Font size doesn't strictly apply to icon-only, but good default
            16.0
        }
    }
}

// ======== Komponenten ========

/// Marker component for query filtering
#[derive(Component, Default, Debug)]
pub struct ButtonMarker;

/// Stores the configured state and resolved style definition of the button
#[derive(Component, Debug, Clone)]
pub struct ButtonState {
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub disabled: bool,
    pub style_def: ButtonStyleDef,
    // Potential future additions: font_size, icon_size ?
}

/// Optional: Component to hold a direct callback closure.
/// Generally prefer events, but can be useful for simple cases.
#[derive(Component, Clone)]
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
        (self.callback)();
    }
}

/// Event sent when a non-disabled button is clicked (Pressed state).
#[derive(Event, Debug, Clone)]
pub struct ButtonClickedEvent {
    /// The entity ID of the button that was clicked.
    pub button_entity: Entity,
}

// ======== Kinder-Definitionen ========

#[derive(Clone, Debug)]
pub enum ButtonChild {
    Text(String),
    Icon(Handle<Image>),
    // Could add Spacer, etc. later
}

// ======== Builder ========
pub struct ButtonBuilder<F: Fn() + Send + Sync + 'static = fn()> {
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    children_defs: Vec<ButtonChild>,
    on_click: Option<OnClick<F>>, // Store the component directly
    width: Option<Val>,
    height: Option<Val>,
    // Allow adding arbitrary components/markers via closures
    markers: Vec<Box<dyn FnOnce(&mut EntityCommands) + Send + Sync>>,
}

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
            markers: Vec::new(), // cannot clone FnOnce closures, so start fresh
        }
    }
}

// Default constructor uses a null function type `fn()` for the callback
impl Default for ButtonBuilder<fn()> {
    fn default() -> Self {
        Self {
            variant: ButtonVariant::Default,
            size: ButtonSize::Default,
            disabled: false,
            children_defs: Vec::new(),
            on_click: None, // No callback by default
            width: None,
            height: None,
            markers: Vec::new(),
        }
    }
}

impl ButtonBuilder<fn()> {
    pub fn new() -> Self {
        Self::default()
    }
}

// Methods available for any ButtonBuilder type
impl<F: Fn() + Send + Sync + 'static> ButtonBuilder<F> {
    /// Sets the visual style variant of the button.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the size preset of the button.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Sets whether the button is disabled (visually grayed out, blocks interaction).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets an explicit width for the button node. Overrides default size behavior.
    pub fn width(mut self, width: Val) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets an explicit height for the button node. Overrides default size behavior.
    pub fn height(mut self, height: Val) -> Self {
        self.height = Some(height);
        self
    }

    /// Adds an icon to the button. If size is `ButtonSize::Icon`, this replaces any text.
    pub fn with_icon(mut self, icon_handle: Handle<Image>) -> Self {
        if self.size == ButtonSize::Icon {
            self.children_defs.clear(); // Icon size implies only icon
        }
        self.children_defs.push(ButtonChild::Icon(icon_handle));
        self
    }

    /// Adds text to the button. Ignored if size is `ButtonSize::Icon`.
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        if self.size != ButtonSize::Icon {
            self.children_defs.push(ButtonChild::Text(text.into()));
        }
        self
    }

    /// Replaces the current onClick callback with a new one.
    /// This CHANGES the type of the builder if the new callback is not `fn()`.
    pub fn on_click<NewF: Fn() + Send + Sync + 'static>(
        self,
        callback: NewF,
    ) -> ButtonBuilder<NewF> {
        // Create a new builder with the different callback type
        ButtonBuilder {
            variant: self.variant,
            size: self.size,
            disabled: self.disabled,
            children_defs: self.children_defs,
            on_click: Some(OnClick::new(callback)), // Store the component
            width: self.width,
            height: self.height,
            markers: self.markers,
        }
    }

    /// Adds a closure that can insert additional components or bundles onto the button entity.
    pub fn mark<M: Bundle + Default>(self) -> Self
    where
        M: Send + Sync + 'static,
    {
        // This is a simplified way to add a *marker* component type.
        // You could generalize `add_component` if needed.
        self.add_marker(|ec| {
            ec.insert(M::default());
        })
    }

    /// Adds a generic closure to modify the EntityCommands after spawning the core button.
    pub fn add_marker(
        mut self,
        func: impl FnOnce(&mut EntityCommands) + Send + Sync + 'static,
    ) -> Self {
        self.markers.push(Box::new(func));
        self
    }

    /// Spawns the button entity as a child of the given parent UI node.
    /// Requires the font handle for text rendering.
    ///
    /// Returns the `EntityCommands` for the newly spawned button, allowing further customization.
    #[must_use] // Indicate that the EntityCommands should likely be used (e.g., to get the Entity id)
    pub fn spawn<'w, 'a>(
        self,
        parent: &'a mut ChildBuilder<'w>,
        font: Handle<Font>, // Pass the font handle explicitly
        theme: &UiTheme,    // TODO: Add theme when ready
    ) -> EntityCommands<'a> {
        // 1. Get the style definition for the variant (replace with theme access later)
        let style_def = get_button_style_def(self.variant, theme);

        // 2. Prepare the base Style and determine font size
        let mut style = base_style();
        let font_size = apply_size_style(&mut style, self.size);
        // let border_radius = theme.radius; // TODO: Use theme radius

        // 3. Apply width/height overrides
        if let Some(w) = self.width {
            style.width = w;
        }
        if let Some(h) = self.height {
            style.height = h;
            // Ensure min_height respects explicit height if set
            if let (Val::Px(current), Val::Px(h_px)) = (style.min_height, h) {
                if current < h_px {
                    style.min_height = h;
                }
            }
        }

        // 4. Spawn the button entity
        let mut cmd = parent.spawn((
            // Bevy UI necessary components
            Button, // Makes it interactive
            Node { ..style },
            style_def.background(Interaction::None, self.disabled),
            style_def.border(Interaction::None, self.disabled),
            // TODO: Set border_radius based on theme.radius
            // BorderRadius::all(border_radius),
            if self.disabled {
                FocusPolicy::Pass
            } else {
                FocusPolicy::Block
            },
            Interaction::default(), // Initialize interaction state
            // Custom components
            ButtonMarker,
            ButtonState {
                variant: self.variant,
                size: self.size,
                disabled: self.disabled,
                style_def: style_def.clone(), // Clone the style def for this button instance
            },
        ));

        // 5. Add the OnClick component if it was configured
        if let Some(on_click_component) = self.on_click {
            cmd.insert(on_click_component);
        }

        // 6. Apply any additional marker components/closures
        for marker_fn in self.markers {
            marker_fn(&mut cmd);
        }

        // 7. Spawn children (Text and/or Icon)
        cmd.with_children(|cb| {
            for child_def in self.children_defs {
                match child_def {
                    ButtonChild::Text(text) => {
                        cb.spawn((
                            Text::new(text),
                            TextFont {
                                font: font.clone(),
                                font_size,
                                ..Default::default() // Initial text color
                            },
                            TextColor(style_def.text_color), // Initial text color
                        ));
                        // Note: For links, text decoration would need custom handling here or in a system
                    }
                    ButtonChild::Icon(handle) => {
                        // Determine icon size based on button size (adjust as needed)
                        let icon_size = match self.size {
                            ButtonSize::Small => Val::Px(14.0),
                            ButtonSize::Icon => Val::Percent(75.0), // Fill most of the icon button
                            _ => Val::Px(16.0),
                        };

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
                            BackgroundColor(Color::NONE),
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

        // Return the commands for further modification if needed
        cmd
    }
}

// ======== Systeme ========

/// System to update the button's background, border, and children's colors
/// based on interaction state and disabled status.
pub fn update_button_visuals(
    mut buttons: Query<
        (
            Entity, // Get the entity ID for context if needed
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &ButtonState,
            Option<&Children>, // Access children if they exist
        ),
        (Changed<Interaction>, With<ButtonMarker>), // Run only when interaction changes
    >,
    // Query to access Text children specifically
    mut text_query: Query<&mut TextColor, With<Text>>, // Need mut Text to change section color
    // Query to access Image children specifically (for tinting)
    // Updated for Bevy 0.13+ - Accessing the image color tint requires the Node Color component if used before.
    // Let's assume we are using BackgroundColor on the ImageBundle to TINT it. This is a common workaround.
    mut image_bg_color_query: Query<&mut BackgroundColor, (With<ImageNode>, Without<ButtonMarker>)>, // Target the ImageBundle's BG for tint
                                                                                                     // Alternatively, if using Bevy 0.12 or before, or a different tinting method:
                                                                                                     // mut image_tint_query: Query<&mut UiImage>,
) {
    for (_entity, interaction, mut bg_color, mut border_color, state, children_opt) in
        buttons.iter_mut()
    {
        // 1. Update Button Background and Border
        *bg_color = state.style_def.background(*interaction, state.disabled);
        *border_color = state.style_def.border(*interaction, state.disabled);

        // --- 2. Update Children Colors ---
        if let Some(children) = children_opt {
            let child_target_color = if state.disabled {
                state.style_def.text_color.with_alpha(0.6) // Disabled = Grayed out
            } else {
                // Here you *could* slightly change child color on hover/press too, if desired
                // match *interaction {
                //    Interaction::Hovered => state.style_def.text_color.lighter(0.1),
                //    _ => state.style_def.text_color
                // }
                state.style_def.text_color // Default: Use base text color
            };

            for &child_entity in children.iter() {
                // Attempt to update as Text
                if let Ok(mut text_color) = text_query.get_mut(child_entity) {
                    // Update color for all sections in the text
                    *text_color = TextColor(child_target_color);
                }

                // Attempt to update as Image (Tint via BackgroundColor)
                if let Ok(mut image_bg) = image_bg_color_query.get_mut(child_entity) {
                    // Treat the background color of the ImageBundle node AS the tint
                    *image_bg = BackgroundColor(child_target_color);
                }

                // Alternative: If using UiImage.tint directly (less common now)
                // if let Ok(mut ui_image) = image_tint_query.get_mut(child_entity) {
                //     ui_image.tint = child_target_color;
                // }
            }
        }
    }
}

/// System that detects button presses (Interaction::Pressed) on non-disabled buttons
/// and sends a `ButtonClickedEvent`.
pub fn handle_button_clicks_event(
    mut interactions: Query<
        (Entity, &Interaction, &ButtonState),
        // Trigger only when Interaction changes to avoid sending events every frame while pressed
        (Changed<Interaction>, With<ButtonMarker>),
    >,
    mut button_clicked_events: EventWriter<ButtonClickedEvent>,
) {
    for (entity, interaction, state) in interactions.iter_mut() {
        // Send event specifically when the button *becomes* pressed
        if *interaction == Interaction::Pressed && !state.disabled {
            debug!("Button {:?} pressed, sending event.", entity);
            button_clicked_events.send(ButtonClickedEvent {
                button_entity: entity,
            });
        }
    }
}

/// Optional: System to handle direct `fn()` callbacks stored in `OnClick`.
/// Note: This system only works for `OnClick<fn()>`, not other closure types.
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
