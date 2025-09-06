mod aether;
mod bevy;

use crate::aether::AetherApp;
use color_eyre::Result;

#[tokio::main]
pub async fn main() -> Result<()> {
    let base = app::init::<AetherApp>().expect("Inizialisation went wrong");
    let mut app = AetherApp::init(base)?;
    app.run().await?;
    Ok(())
}
