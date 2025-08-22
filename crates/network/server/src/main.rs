// mod certificate;
// mod runtime;
// mod server;
// // mod settings;

// // fn main() {
// //     // wizard::run();
// //     let mut server = server::ServerApp::new("localhost".to_string(), 8080, 1.0);
// //     server.init();
// // }
// use bevy::prelude::*;
// use std::sync::mpsc::{Receiver, Sender, channel};
// use std::sync::{Arc, Mutex};
// use std::thread;

// // use crate::settings::ServerConfig;

// // Beispiel-Commands für Settings
// enum SettingCommand {
//     UpdateKeyPath(String, String), // Key, Value
// }

// struct StatsSnapshot {
//     num_players: usize,
//     server_uptime: f32,
//     difficulty: String,
//     debug: bool,
//     // weitere Felder...
// }

// // Sender/Receiver-Wrapper für Commands ins Bevy-System
// #[derive(Resource)]
// struct CommandSender(Sender<SettingCommand>);
// #[derive(Resource)]
// struct CommandReceiver(Arc<Mutex<Receiver<SettingCommand>>>);

// // Sender/Receiver-Wrapper für Stats
// #[derive(Resource)]
// struct StatsSender(Sender<StatsSnapshot>);
// #[derive(Resource)]
// struct StatsReceiver(Arc<Mutex<Receiver<StatsSnapshot>>>);

// fn main() {
//     // 1. Channels für Settings-Kommandos einrichten
//     let (cmd_tx, cmd_rx) = channel::<SettingCommand>();
//     let (stats_tx, stats_rx) = channel::<StatsSnapshot>();

//     // Receiver in Arc<Mutex<>> wrappen für Thread-Sicherheit
//     let stats_rx = Arc::new(Mutex::new(stats_rx));
//     let cmd_rx = Arc::new(Mutex::new(cmd_rx));

//     // search and load Server Settings
//     // let server_config = settings::ServerConfig::load();
//     //search for login credentials
//     // let login_credentials = settings::LoginCredentials::load();

//     // 2. start TUI und Webserver
//     let tui_stats_rx = stats_rx.clone();
//     let tui_cmd_tx = cmd_tx.clone();
//     // wizard::run(server_config, login_credentials, tui_cmd_tx, tui_stats_rx);
//     // wizard::run().expect("Failed to run wizard");

//     // match server_config.server_managment_mode {
//     //     Ok(settings::ServerManagmentMode::WEBANDTUI) => {
//     //         // TUI und Webserver starten
//     //         println!("Starting TUI and web server...");
//     //         webserver::run(
//     //             server_config.clone(),
//     //             login_credentials.clone(),
//     //             stats_rx.clone(),
//     //             cmd_tx.clone(),
//     //         );
//     //     }
//     //     _ => {
//     //         // Nur TUI starten
//     //         println!("Starting TUI only...");
//     //     }
//     // }
//     // let web_stats_rx = stats_rx.clone();
//     // let web_cmd_tx = cmd_tx.clone();
//     // thread::spawn(move || {
//     //     webserver::run(server_config, login_credentials, web_stats_rx, web_cmd_tx);
//     // });

//     // Server start wenn TUI initialisiert ist und alle Settings gesetzt sind
//     thread::spawn(move || {
//         App::new()
//             .insert_resource(StatsSender(stats_tx))
//             .insert_resource(StatsReceiver(stats_rx))
//             .insert_resource(CommandReceiver(cmd_rx)) // Receiver in Ressource packen
//             // Weitere Spiel-Logik-Systeme ...
//             .run();
//     });
// }
fn main() {}
