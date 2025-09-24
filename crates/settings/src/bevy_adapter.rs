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
struct SettingsRegistry {
    updaters: Vec<fn(&Arc<crate::SettingsStore>, &mut World)>,
}

pub trait AppSettingsExt {
    fn insert_settings_store(self, store: crate::SettingsStore) -> Self;
    fn register_settings_section<S>(self) -> Self
    where
        S: crate::Settings + Send + Sync + 'static + Clone;
    // fn settings_poll_interval(self, dur: Duration) -> Self;
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
        // 1) typed section im Store registrieren
        store.register::<S>().expect("register settings section");
        // 2) initial seed als Bevy-Resource
        let arc = store.get::<S>().expect("get after register");
        self.world_mut().insert_resource(SettingsArc::<S>(arc));
        // 3) Updater hinterlegen
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
/// Errors are logged via the store logger (or eprintln! fallback).
pub fn settings_reload_system(world: &mut World) {
    // Clone Arc so we can release world borrow while reloading.
    let store_arc = world.resource::<SettingsStoreRef>().0.clone();
    if store_arc.reload().is_ok() {
        // Re-run all updaters (clone list to avoid borrow conflicts).
        let reg = world.resource::<SettingsRegistry>();
        let updaters: Vec<_> = reg.updaters.iter().copied().collect();
        drop(reg);
        for updater in updaters {
            updater(&store_arc, world);
        }
    }
}
