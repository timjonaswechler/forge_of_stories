use super::enums::{ButtonSize, ButtonVariant};
use super::style::ButtonStyleDef; // Braucht ButtonStyleDef
use bevy::prelude::*;
use std::marker::PhantomData;

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
}

/// Optional: Component to hold a direct callback closure.
#[derive(Component, Clone)]
pub struct OnClick<F: Fn() + Send + Sync + 'static> {
    pub(super) callback: F, // Machen wir 'pub(super)' damit nur call() public ist
    pub(super) _marker: PhantomData<F>,
}

impl<F: Fn() + Send + Sync + 'static> OnClick<F> {
    pub fn new(callback: F) -> Self {
        Self {
            callback,
            _marker: PhantomData,
        }
    }
    // Behalten wir die call Methode public
    pub fn call(&self) {
        (self.callback)();
    }
}
