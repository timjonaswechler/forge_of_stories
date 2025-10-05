#![cfg(feature = "bevy")]
use crate::{General, Network, Security};
use app::AppContext;
use bevy::prelude::*;
pub use bevy::{ecs, prelude::Resource};

pub trait AppAetherSettingsExt {
    fn use_aether_server_settings(
        self,
        context: &AppContext,
        store: Option<settings::SettingsStore>,
    ) -> Self;
}

impl AppAetherSettingsExt for App {
    fn use_aether_server_settings(
        self,
        context: &AppContext,
        store: Option<settings::SettingsStore>,
    ) -> Self {
        use settings::AppSettingsExt;

        let store = match store {
            Some(s) => s,
            None => crate::build_server_settings_store(
                &context
                    .path_context
                    .settings_file(Some(context.app_id().to_string())),
                &context.version(),
            )
            .expect("build server settings store"),
        };

        self.insert_settings_store(store)
            .register_settings_section::<General>()
            .register_settings_section::<Network>()
            .register_settings_section::<Security>()
    }
}
