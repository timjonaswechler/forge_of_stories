use app::Application;

/// Aether server application.
///
/// This is a marker type that defines the application's identity.
/// The actual app logic is built using AppBuilder in main.rs.
pub struct AetherApp;

impl Application for AetherApp {
    const APP_ID: &'static str = "aether";
}
