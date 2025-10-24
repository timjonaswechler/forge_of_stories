use serde::{Deserialize, Serialize};
use settings::Settings;

#[derive(Clone, Serialize, Deserialize)]
pub struct Network {
    port: u16,
    label: String,
}
impl Default for Network {
    fn default() -> Self {
        Self {
            port: 5000,
            label: "main".into(),
        }
    }
}
impl Settings for Network {
    const SECTION: &'static str = "network";
}
