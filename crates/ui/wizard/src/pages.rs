use color_eyre::Result;
use ratatui::{
    Frame,
    layout::{Rect, Size},
};
use std::{any::Any, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, config::Config, tui::Event};

mod home;
mod login;
mod setup;

pub use home::HomePage;
pub use login::LoginPage;
pub use setup::SetupPage;

/// A `Page` composes multiple `Component`s and exposes a lifecycle similar to the
/// existing `Component` trait but at the page level.
pub trait Page {
    #[allow(dead_code)]
    fn name(&self) -> &str;

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        let _ = tx;
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        let _ = config;
        Ok(())
    }

    fn register_shared_state(&mut self, _state: Arc<dyn Any + Send + Sync>) -> Result<()> {
        Ok(())
    }
    fn register_shortcuts(
        &self,
    ) -> Option<(&'static str, Box<[crate::services::shortcuts::Shortcut]>)> {
        None
    }
    fn init(&mut self, area: Size) -> Result<()> {
        let _ = area;
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let _ = event;
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let _ = action;
        Ok(None)
    }

    /// Draw the page using the provided `Frame` and `area`.
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;

    /// Called when the page becomes active.
    fn on_enter(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when the page is leaving / being replaced.
    fn on_exit(&mut self) -> Result<()> {
        Ok(())
    }
}
