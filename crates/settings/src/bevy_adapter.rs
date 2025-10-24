#![cfg(feature = "bevy")]

use bevy::{
    app::App,
    ecs::resource::Resource,
    prelude::{Deref, DerefMut, World},
};
use std::sync::Arc;

#[derive(Resource, Clone)]
pub struct SettingsStoreRef(pub Arc<crate::SettingsStore>);

#[derive(Resource, Deref, DerefMut, Clone)]
pub struct SettingsArc<T: Send + Sync + 'static>(pub Arc<T>);

#[derive(Resource, Default)]
pub struct SettingsRegistry {
    pub updaters: Vec<fn(&Arc<crate::SettingsStore>, &mut World)>,
}

pub trait AppSettingsExt {
    fn insert_settings_store(self, store: crate::SettingsStore) -> Self;
    fn register_settings_section<S>(self) -> Self
    where
        S: crate::Settings + Send + Sync + 'static + Clone;
}

impl AppSettingsExt for App {
    fn insert_settings_store(mut self, store: crate::SettingsStore) -> Self {
        self.world_mut()
            .insert_resource(SettingsStoreRef(Arc::new(store)));
        self.world_mut()
            .insert_resource(SettingsRegistry::default());

        self
    }

    fn register_settings_section<S>(mut self) -> Self
    where
        S: crate::Settings + Send + Sync + 'static + Clone,
    {
        let store = self.world().resource::<SettingsStoreRef>().0.clone();

        // If the resource already exists we are done (idempotent call)
        if self.world().contains_resource::<SettingsArc<S>>() {
            return self;
        }

        // Ensure the section is registered in the store (it may already be,
        // e.g. because the store was pre-built with the section).
        if !store.is_registered::<S>() {
            store.register::<S>().expect("register settings section");
        }

        // Fetch current value and insert as Bevy resource (always insert even if pre‑registered).
        // If deserialization fails (e.g. corrupt / mismatching delta), fall back to defaults so the
        // app can continue running instead of panicking.
        let arc = match store.get::<S>() {
            Ok(arc) => arc,
            Err(e) => {
                eprintln!(
                    "[settings] failed to load section '{}': {e}. Falling back to defaults.",
                    S::name()
                );
                std::sync::Arc::new(S::default())
            }
        };
        self.world_mut().insert_resource(SettingsArc::<S>(arc));

        // Register the updater
        fn update_one<S>(store: &Arc<crate::SettingsStore>, world: &mut World)
        where
            S: crate::Settings + Send + Sync + 'static + Clone,
        {
            if let Ok(new_arc) = store.get::<S>() {
                let mut res = world.resource_mut::<SettingsArc<S>>();
                if !Arc::ptr_eq(&res.0, &new_arc) {
                    res.0 = new_arc;
                }
            }
        }

        let mut reg = self.world_mut().resource_mut::<SettingsRegistry>();
        reg.updaters.push(update_one::<S>);

        self
    }
}

/// Bevy system: reload settings file (if changed externally) and re-run
/// all registered section updaters. Call this in a schedule (e.g. Update)
/// when you want polling-based hot reload without the file watcher feature.
pub fn settings_reload_system(world: &mut World) {
    // Clone Arc so we can release world borrow while reloading.
    let store_arc = world.resource::<SettingsStoreRef>().0.clone();
    if store_arc.reload().is_ok() {
        // Re-run all updaters (clone list to avoid borrow conflicts).
        let reg = world.resource::<SettingsRegistry>();
        let updaters: Vec<_> = reg.updaters.iter().copied().collect();
        let _ = reg;
        for updater in updaters {
            updater(&store_arc, world);
        }
    }
}
