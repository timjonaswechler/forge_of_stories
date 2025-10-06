use app::Application;

/// Forge of Stories client application.
///
/// This is a marker type that defines the application's identity.
/// The actual app logic is built using AppBuilder in main.rs.
pub struct FOSApp;

impl Application for FOSApp {
    const APP_ID: &'static str = "forge_of_stories";
}
