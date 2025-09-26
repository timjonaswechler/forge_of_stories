mod aether;
pub mod bevy;

use crate::aether::AetherApp;
use color_eyre::Result;

#[tokio::main]
pub async fn main() -> Result<()> {
    let base =
        app::init::<AetherApp>(env!("CARGO_PKG_VERSION")).expect("Inizialisation went wrong");
    let mut app = AetherApp::init(base)?;
    app.run().await?;
    Ok(())
}
