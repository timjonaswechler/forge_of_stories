use clap::Parser;
use client::ClientConfig;
use color_eyre::Result;

#[derive(Parser)]
#[command(name = "fos_client_dev")]
#[command(about = "Forge of Stories Client - Development/Testing Entry Point")]
struct Args {
    /// Server address to connect to
    #[arg(short, long, default_value = "127.0.0.1:5000")]
    server: String,

    /// Client configuration file
    #[arg(short, long, default_value = "client.toml")]
    config: String,

    /// Enable local server for testing
    #[arg(long)]
    with_local_server: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    println!("🎮 Starting Forge of Stories Client (Dev Mode)");
    println!("📡 Server: {}", args.server);
    println!("⚙️  Config: {}", args.config);

    if args.with_local_server {
        println!("🖥️  Starting with integrated local server...");
        // Hier würde der integrierte local server gestartet
        // fos_server::start_local_server()?;
    }

    let client_config = ClientConfig::from_file(&args.config)?;
    client::start_client(client_config, &args.server)?;

    Ok(())
}
