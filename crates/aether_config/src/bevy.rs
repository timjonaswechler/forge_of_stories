#![cfg(feature = "bevy")]
use crate::{General, Network, Security};
use app::AppBase;
use bevy::prelude::*;
pub use bevy::{ecs, prelude::Resource};
use settings::{AppSettingsExt, SettingsArc};

pub trait AppAetherSettingsExt {
    fn use_aether_server_settings(
        self,
        base: &AppBase,
        store: Option<settings::SettingsStore>,
    ) -> Self;
}

impl AppAetherSettingsExt for App {
    fn use_aether_server_settings(
        mut self,
        base: &AppBase,
        store: Option<settings::SettingsStore>,
    ) -> Self {
        let store = match store {
            Some(s) => s,
            None => crate::build_server_settings_store(
                &base
                    .path_context
                    .settings_file(Some(base.app_id().to_string())),
                &base.version(),
            )
            .expect("build server settings store"),
        };
        self = self.insert_settings_store(store);

        // Only register each section if it hasn't been registered already
        if !self.world().contains_resource::<SettingsArc<General>>() {
            self = self.register_settings_section::<General>();
        }
        if !self.world().contains_resource::<SettingsArc<Network>>() {
            self = self.register_settings_section::<Network>();
        }
        if !self.world().contains_resource::<SettingsArc<Security>>() {
            self = self.register_settings_section::<Security>();
        }

        self
    }
}
