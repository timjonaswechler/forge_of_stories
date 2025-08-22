use crate::messages::AetherStatsSnapshot;
use serde::{Deserialize, Serialize};
use strum::Display;

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

    // Server control (triggered by UI; handled centrally in App)
    StartServer,
    StopServer,
    RestartServer,

    // Settings gating (set by Settings/Setup flow)
    SettingsReady,
    SettingsInvalid(String),

    // Supervisor events (forwarded from AetherSupervisor into the App action loop)
    ServerStarted,
    ServerStopped,
    ServerStats(AetherStatsSnapshot),
    ApplyRuntimeSetting {
        key: String,
        value: String,
    },
}
