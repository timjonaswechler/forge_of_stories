//! Integration example demonstrating PathContext usage with configuration management.
//!
//! This example shows how PathContext can be used in a real-world scenario
//! with settings, logging, and resource management.

use paths::{PathContext, RuntimeEnvironment};
use std::fs;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PathContext Integration Example ===\n");

    // Initialize path context for the application
    let ctx = PathContext::new(
        "forge_studios", // Your studio name
        "epic_rpg",      // Your game/project
        "game_client",   // The specific app (client, server, editor, etc.)
    );

    // Display environment information
    println!("🔧 Environment: {:?}", ctx.environment());
    println!("📁 Base Path: {:?}\n", ctx.base_path());

    // Create all necessary directories
    println!("📂 Creating directory structure...");
    ctx.ensure_directories()?;
    println!("   ✓ Directories created\n");

    // === Settings Management ===
    println!("⚙️  Settings Management:");
    let settings_path = ctx.settings_file(None);
    println!("   Settings file: {:?}", settings_path);

    // Create a sample settings file
    let settings_content = r#"{
  "version": "1.0.0",
  "graphics": {
    "resolution": "1920x1080",
    "vsync": true,
    "quality": "high"
  },
  "audio": {
    "master_volume": 0.8,
    "music_volume": 0.6,
    "sfx_volume": 0.7
  },
  "network": {
    "auto_connect": true,
    "preferred_server": "eu-west-1"
  }
}"#;

    fs::write(&settings_path, settings_content)?;
    println!("   ✓ Settings file created\n");

    // === Keybinding Configuration ===
    println!("🎮 Keybinding Configuration:");
    let keybinding_path = ctx.keybinding_file();
    println!("   Keybinding file: {:?}", keybinding_path);

    let keybinding_content = r#"{
  "movement": {
    "forward": "W",
    "backward": "S",
    "left": "A",
    "right": "D"
  },
  "actions": {
    "jump": "Space",
    "interact": "E",
    "inventory": "I"
  }
}"#;

    fs::write(&keybinding_path, keybinding_content)?;
    println!("   ✓ Keybinding file created\n");

    // === Server Configuration ===
    println!("🌐 Server Configuration:");
    let servers_path = ctx.servers_file();
    println!("   Servers file: {:?}", servers_path);

    let servers_content = r#"{
  "servers": [
    {
      "id": "eu-west-1",
      "name": "Europe West",
      "address": "eu-west.epicrpg.com:7777"
    },
    {
      "id": "us-east-1",
      "name": "US East",
      "address": "us-east.epicrpg.com:7777"
    }
  ]
}"#;

    fs::write(&servers_path, servers_content)?;
    println!("   ✓ Servers file created\n");

    // === Version Management ===
    println!("📦 Version Management:");
    let version = "1.2.3";
    let version_path = ctx.version_file(version);
    println!("   Version file: {:?}", version_path);

    let version_content = format!(
        r#"{{
  "version": "{}",
  "release_date": "2024-03-15",
  "changelog": [
    "Added new quest system",
    "Fixed inventory bug",
    "Performance improvements"
  ]
}}"#,
        version
    );

    fs::write(&version_path, version_content)?;
    println!("   ✓ Version file created\n");

    // === Save Game Management ===
    println!("💾 Save Game Management:");
    let save_name = "autosave_001";
    let save_path = ctx.save_dir(save_name);
    println!("   Save directory: {:?}", save_path);

    fs::create_dir_all(&save_path)?;

    // Create sample save files
    let save_data_path = save_path.join("game_state.json");
    let save_data = r#"{
  "player": {
    "name": "Hero",
    "level": 42,
    "position": {"x": 123.4, "y": 56.7, "z": 89.0}
  },
  "progress": {
    "main_quest": 15,
    "side_quests": 23
  }
}"#;

    fs::write(&save_data_path, save_data)?;
    println!("   ✓ Save files created\n");

    // === Mod/DLC Management ===
    println!("🎨 Mod/DLC Management:");
    let mods_dir = ctx.mods_dir();
    println!("   Mods directory: {:?}", mods_dir);

    // Create sample mod structure
    let sample_mod_dir = mods_dir.join("awesome_mod");
    fs::create_dir_all(&sample_mod_dir)?;

    let mod_info_path = sample_mod_dir.join("mod.json");
    let mod_info = r#"{
  "name": "Awesome Mod",
  "version": "2.0.0",
  "author": "ModMaker",
  "description": "Adds awesome features to the game"
}"#;

    fs::write(&mod_info_path, mod_info)?;
    println!("   ✓ Mod structure created\n");

    // === Asset Management ===
    println!("🖼️  Asset Management:");
    let assets_dir = ctx.assets_dir();
    println!("   Assets directory: {:?}", assets_dir);

    fs::create_dir_all(&assets_dir)?;

    let asset_manifest_path = assets_dir.join("manifest.json");
    let asset_manifest = r#"{
  "assets": [
    {"id": "texture_001", "path": "textures/hero.png"},
    {"id": "sound_001", "path": "sounds/background.ogg"},
    {"id": "model_001", "path": "models/sword.glb"}
  ]
}"#;

    fs::write(&asset_manifest_path, asset_manifest)?;
    println!("   ✓ Asset manifest created\n");

    // === Logging ===
    println!("📝 Logging:");
    let log_path = ctx.log_file_now();
    println!("   Log file: {:?}", log_path);

    let mut log_file = fs::File::create(&log_path)?;
    writeln!(log_file, "[INFO] Application started")?;
    writeln!(log_file, "[INFO] Environment: {:?}", ctx.environment())?;
    writeln!(log_file, "[INFO] Settings loaded from: {:?}", settings_path)?;
    writeln!(log_file, "[INFO] All systems initialized")?;
    println!("   ✓ Log file created\n");

    // === Summary ===
    println!("📊 Summary:");
    println!("   Studio: {}", ctx.studio());
    println!("   Project: {}", ctx.project_id());
    println!("   App ID: {}", ctx.app_id());
    println!("   Project root: {:?}", ctx.project_root());
    println!();

    // List all created files
    println!("📄 Created files:");
    let created_files = vec![
        ("Settings", settings_path),
        ("Keybindings", keybinding_path),
        ("Servers", servers_path),
        ("Version", version_path),
        ("Save", save_data_path),
        ("Mod Info", mod_info_path),
        ("Asset Manifest", asset_manifest_path),
        ("Log", log_path),
    ];

    for (name, path) in created_files {
        if path.exists() {
            let metadata = fs::metadata(&path)?;
            println!("   ✓ {} ({} bytes): {:?}", name, metadata.len(), path);
        }
    }

    println!("\n✨ Integration example completed successfully!");

    // Environment-specific advice
    println!("\n💡 Tips:");
    match ctx.environment() {
        RuntimeEnvironment::Development => {
            println!("   • Running in DEVELOPMENT mode");
            println!("   • Files are stored in project directory");
            println!("   • Perfect for testing and debugging");
        }
        RuntimeEnvironment::Production => {
            println!("   • Running in PRODUCTION mode");
            println!("   • Files are stored in platform app data directory");
            println!("   • Safe for end-user installations");
        }
    }

    Ok(())
}
