use serde::{Deserialize, Serialize};
use strum::Display;

use crate::messages::AetherToWizard;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    /// Navigate to a page by id/name. The string should match a registered page id.
    Navigate(String),
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    Reload,
    Save,
    ClearScreen,
    Error(String),
    IdleTimeout,
    Help,
    AetherEvent(AetherToWizard),
    // Server control (triggered by UI; handled centrally in App)
    StartServer,
    StopServer,
    RestartServer,

    // Settings gating (set by Settings/Setup flow)
    SettingsReady,
    SettingsInvalid(String),

    // Supervisor events (forwarded from AetherSupervisor into the App action loop)
    ApplyRuntimeSetting {
        key: String,
        value: String,
    },
}
