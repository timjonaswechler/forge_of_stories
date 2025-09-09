use std::sync::Arc;

use aether_config::{ServerSettingField, apply_server_setting, build_server_settings_store};
use color_eyre::Result;
use settings::SettingsStore;

/// Domain Facade: SettingsGateway
///
/// Phase 8.2 – This gateway encapsulates direct interaction with
/// `aether_config` + `SettingsStore` so UI code (e.g. `SettingsPage`)
/// depends only on this abstraction and not on the lower-level crates.
///
/// Design goals:
/// - Narrow surface: only expose what the UI currently needs
/// - Hide construction (`build_server_settings_store`)
/// - Provide semantic helpers for frequently used mutations
/// - Allow future extension (validation, caching, diffing, async IO)
///
/// Non-goals (for now):
/// - Category modeling (left to UI for presentation concerns)
/// - Generic reflection over all settings (can be added later)
///
/// Migration Plan:
/// 1. Replace direct calls in `SettingsPage` to:
///    - `build_server_settings_store`  -> `SettingsGateway::new_server`
///    - `apply_server_setting`         -> `SettingsGateway::apply` / helpers
///    - Direct `Arc<SettingsStore>` use -> `gateway.store()`
/// 2. Later: move per‑category export logic here if duplication emerges.
///
/// Thread-safety:
/// - Internally holds an `Arc<SettingsStore>` so cheap to clone.
/// - If mutation semantics of `SettingsStore` change, wrap in RwLock/Mutex.
///
/// Error handling:
/// - Uses `color_eyre::Result` for consistency with the rest of the crate.
#[derive(Clone)]
pub struct SettingsGateway {
    store: Arc<SettingsStore>,
}

impl SettingsGateway {
    /// Construct a gateway for the server settings domain.
    pub fn new_server() -> Result<Self> {
        let store = build_server_settings_store()?;
        Ok(Self {
            store: Arc::new(store),
        })
    }

    /// Raw access for legacy components that still expect an `Arc<SettingsStore>`.
    /// Prefer adding focused methods instead of leaking more of the store API.
    pub fn store(&self) -> Arc<SettingsStore> {
        self.store.clone()
    }

    /// Generic apply wrapper. Accepts any `ServerSettingField` + raw string value.
    pub fn apply(&self, field: ServerSettingField, value: &str) -> Result<()> {
        apply_server_setting(&self.store, field, value)
    }

    /// Semantic helper: toggle the Autostart flag.
    pub fn set_autostart(&self, enabled: bool) -> Result<()> {
        self.apply(
            ServerSettingField::GeneralAutostart,
            if enabled { "true" } else { "false" },
        )
    }

    /// Export a snapshot of key/value pairs for a given list of fields.
    /// Useful for constructing UI representations without exposing the store.
    ///
    /// NOTE: For now this is a very small helper. It can be expanded to:
    /// - derive from a category model
    /// - include metadata (validation rules, types, defaults)
    /// - diff vs. persisted values
    pub fn export_fields(
        &self,
        fields: &[ServerSettingField],
    ) -> Vec<(ServerSettingField, String)> {
        fields
            .iter()
            .filter_map(|f| {
                // `SettingsStore` currently lacks a strongly typed accessor in this context;
                // if/when one is added, replace this with the higher-level API. For now,
                // we rely on `serde` export (if available) or implement a manual mapping.
                //
                // Placeholder: returning empty list until richer introspection is available.
                //
                // Alternative approach (future):
                //   let raw = self.store.get_raw(f);
                //   Some((*f, raw.to_string()))
                //
                // Returning None keeps the iterator compact.
                let _ = f;
                None
            })
            .collect()
    }

    /// Execute a closure with an immutable reference to the underlying store.
    /// This can help transition legacy code while keeping page-level code
    /// restricted to the gateway surface.
    pub fn with_store<R>(&self, f: impl FnOnce(&SettingsStore) -> R) -> R {
        f(&self.store)
    }
}

/// Builder-style convenience if future variants (e.g., client vs server) appear.
impl Default for SettingsGateway {
    fn default() -> Self {
        Self::new_server().expect("failed to build server settings store")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gateway_constructs() {
        let gw = SettingsGateway::new_server().expect("gateway new_server failed");
        assert!(Arc::strong_count(&gw.store()) >= 1);
    }

    #[test]
    fn apply_autostart_bool() {
        let gw = SettingsGateway::new_server().expect("gateway new_server failed");
        gw.set_autostart(true).expect("apply autostart true failed");
        gw.set_autostart(false)
            .expect("apply autostart false failed");
    }
}
