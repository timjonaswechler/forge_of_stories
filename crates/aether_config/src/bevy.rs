#![cfg(feature = "bevy")]

use bevy::prelude::*;
use settings::{AppSettingsExt, SettingsArc};

pub trait AppAetherSettingsExt {
    fn use_aether_server_settings(self, store: Option<settings::SettingsStore>) -> Self;
}

impl AppAetherSettingsExt for App {
    fn use_aether_server_settings(mut self, store: Option<settings::SettingsStore>) -> Self {
        let store = match store {
            Some(s) => s,
            None => crate::build_server_settings_store().expect("build server settings store"),
        };
        self = self.insert_settings_store(store);
        self = self
            .register_settings_section::<crate::General>()
            .register_settings_section::<crate::Network>()
            .register_settings_section::<crate::Security>();
        self
    }
}

// Bequeme Type-Aliase für Resources in Bevy:
pub type GeneralRes = SettingsArc<crate::General>;
pub type NetworkRes = SettingsArc<crate::Network>;
pub type SecurityRes = SettingsArc<crate::Security>;
