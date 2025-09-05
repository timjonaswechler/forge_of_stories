use anyhow::Result;

use server::{ServerConfigMinimal, start_server};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = color_eyre::install();

    // Temporary quick-start config; later this will be read from Settings.
    let cfg = ServerConfigMinimal::default();

    println!("🚀 Forge of Stories Server booting...");
    let _handle = start_server(Some(cfg.clone())).await?;

    println!("✅ Live: listening on {}:{}", cfg.bind_address, cfg.port);
    println!("⏹️  Press Ctrl+C to stop.");

    tokio::signal::ctrl_c().await?;
    println!("🛑 Shutting down.");

    Ok(())
}
