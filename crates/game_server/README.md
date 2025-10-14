Ja, ich erstelle jetzt einen detaillierten Plan für die Integration von bevy_replicon in das game_server Crate mit der Anforderung, dass du zur Runtime Endpoints aktivieren und
  deaktivieren kannst.

  🎯 bevy_replicon Integration Plan für game_server

  Architektur-Übersicht

  Was ist bevy_replicon?

  bevy_replicon ist ein Server-Authoritative Replication Framework für Bevy, das:
  - ✅ Automatische Entity & Component Replication
  - ✅ Change Detection (nur Änderungen werden gesendet)
  - ✅ Dual-Channel System (Reliable Updates + Unreliable Mutations)
  - ✅ Client/Server Message System
  - ✅ Transport-agnostisch (BYO Transport)

  Wichtig: bevy_replicon liefert KEIN I/O!

  Du musst deine eigenen Transports (QUIC, Steam, Loopback) mit bevy_replicon verbinden.

  ---
  📋 Was wir brauchen

  1. Core Dependencies

  [dependencies]
  bevy_replicon = "0.36.1"
  bevy = { version = "0.17.2", features = ["serialize"] }

  2. Plugin-Struktur

  game_server/
  ├── lib.rs                      # GameServerPlugin + Replicon Setup
  ├── components.rs               # Replicated Components (Player, Position, Velocity)
  ├── messages.rs                 # Client Messages (PlayerInput)
  ├── replication/
  │   ├── mod.rs
  │   ├── transport_bridge.rs    # Bridge: networking → bevy_replicon
  │   └── runtime_endpoints.rs   # Runtime Endpoint Management
  ├── systems/
  │   ├── movement.rs             # Input processing + movement
  │   ├── spawning.rs             # Player spawn/despawn
  │   └── heartbeat.rs            # Heartbeat (unchanged)
  ├── world_setup.rs              # Terrain (unchanged)
  └── savegame.rs                 # Savegame (unchanged)

  ---
  🔧 Komponenten-Design

  1. Replicated Components

  // components.rs
  use bevy::prelude::*;
  use serde::{Deserialize, Serialize};

  #[derive(Component, Serialize, Deserialize, Clone)]
  pub struct Player {
      pub client_id: ClientId,
  }

  #[derive(Component, Serialize, Deserialize, Clone, Copy)]
  pub struct Position(pub Vec3);

  #[derive(Component, Serialize, Deserialize, Clone, Copy)]
  pub struct Velocity(pub Vec3);

  #[derive(Component, Serialize, Deserialize, Clone, Copy)]
  pub struct PlayerColor(pub Color);

  2. Client Messages

  // messages.rs
  use bevy::prelude::*;
  use serde::{Deserialize, Serialize};

  #[derive(Serialize, Deserialize, Clone)]
  pub struct PlayerInput {
      pub direction: Vec2,
      pub jump: bool,
  }

  ---
  🌉 Transport Bridge Design

  Das ist der Kern der Integration: Wir brauchen einen Bridge-Layer, der deine bestehenden Transports (QUIC, Steam, Loopback) mit bevy_replicon verbindet.

  Konzept: TransportBridge

  // replication/transport_bridge.rs

  use bevy::prelude::*;
  use bevy_replicon::prelude::*;
  use networking::shared::{TransportEvent, ClientId};
  use std::collections::HashMap;

  /// Bridge zwischen unserem Transport-System und bevy_replicon
  #[derive(Resource)]
  pub struct TransportBridge {
      /// Mapping: ServerMode → Channel-System
      active_transports: HashMap<ClientId, TransportHandle>,

      /// Queue für outgoing replicon messages
      outgoing_queue: Vec<(ClientId, ChannelId, Vec<u8>)>,

      /// Queue für incoming transport events
      incoming_queue: Vec<TransportEvent>,
  }

  struct TransportHandle {
      client_id: ClientId,
      // Optional: Transport-spezifische Daten
  }

  impl TransportBridge {
      /// Poll events von deinen Transports und füttere sie zu Replicon
      pub fn poll_transports(&mut self,
          loopback: Option<&mut LoopbackServerTransport>,
          external: Option<&mut ExternalTransport>,
      ) {
          self.incoming_queue.clear();

          if let Some(lb) = loopback {
              lb.poll_events(&mut self.incoming_queue);
          }

          if let Some(ext) = external {
              ext.poll_events(&mut self.incoming_queue);
          }
      }

      /// Registriere einen neuen Client in Replicon
      pub fn register_client(&mut self, client_id: ClientId) {
          self.active_transports.insert(client_id, TransportHandle {
              client_id,
          });
      }

      /// Entferne einen Client aus Replicon
      pub fn unregister_client(&mut self, client_id: ClientId) {
          self.active_transports.remove(&client_id);
      }

      /// Sende Replicon-Messages über deine Transports
      pub fn send_replicon_messages(&mut self,
          loopback: Option<&mut LoopbackServerTransport>,
          external: Option<&mut ExternalTransport>,
      ) {
          for (client_id, channel_id, data) in self.outgoing_queue.drain(..) {
              let payload = TransportPayload::message(channel_id, data.into());

              if client_id == HOST_CLIENT_ID {
                  if let Some(lb) = loopback.as_mut() {
                      let _ = lb.send(client_id, payload);
                  }
              } else {
                  if let Some(ext) = external.as_mut() {
                      let _ = ext.send(client_id, payload);
                  }
              }
          }
      }
  }

  Integration mit bevy_replicon Channels

  // Systems die Transport → Replicon bridgen

  /// System: Transport Events → Replicon
  fn bridge_transport_to_replicon(
      mut bridge: ResMut<TransportBridge>,
      mut server_messages: ResMut<ServerMessages>,
  ) {
      for event in &bridge.incoming_queue {
          match event {
              TransportEvent::Connected(client_id) => {
                  bridge.register_client(*client_id);
                  // Trigger Replicon ConnectEvent
                  server_messages.insert_received(*client_id, /* ... */);
              }
              TransportEvent::Disconnected(client_id) => {
                  bridge.unregister_client(*client_id);
                  // Trigger Replicon DisconnectEvent
              }
              TransportEvent::Message { client_id, channel, data } => {
                  // Füttere Message zu Replicon
                  server_messages.insert_received(*client_id, *channel, data.clone());
              }
              _ => {}
          }
      }
  }

  /// System: Replicon → Transport
  fn bridge_replicon_to_transport(
      mut bridge: ResMut<TransportBridge>,
      mut server_messages: ResMut<ServerMessages>,
  ) {
      // Drain messages aus Replicon und queue sie für Transport
      for (client_id, channel_id, data) in server_messages.drain_sent() {
          bridge.outgoing_queue.push((client_id, channel_id, data));
      }
  }

  ---
  🎮 Runtime Endpoint Management

  Anforderung: Zur Runtime Endpoints aktivieren/deaktivieren

  // replication/runtime_endpoints.rs

  use bevy::prelude::*;
  use networking::server::transport::{ServerTransport, ExternalTransport};

  /// Resource für Runtime Transport Management
  #[derive(Resource)]
  pub struct RuntimeTransports {
      /// Loopback ist immer aktiv (Embedded mode)
      loopback: Option<LoopbackServerTransport>,

      /// External kann zur Runtime hinzugefügt/entfernt werden
      external: Option<ExternalTransport>,

      /// Status-Tracking
      external_active: bool,
  }

  impl RuntimeTransports {
      /// Aktiviere External Transport (QUIC oder Steam)
      pub fn activate_external(&mut self, transport: ExternalTransport) -> Result<(), String> {
          if self.external.is_some() {
              return Err("External transport already active".to_string());
          }

          info!("Activating external transport");
          self.external = Some(transport);
          self.external_active = true;
          Ok(())
      }

      /// Deaktiviere External Transport
      pub fn deactivate_external(&mut self) -> Result<(), String> {
          match self.external.take() {
              Some(mut transport) => {
                  info!("Deactivating external transport");
                  transport.shutdown();
                  self.external_active = false;
                  Ok(())
              }
              None => Err("No external transport active".to_string()),
          }
      }

      /// Check ob External aktiv ist
      pub fn is_external_active(&self) -> bool {
          self.external_active
      }
  }

  /// System: Verarbeite Runtime Transport Changes
  fn handle_transport_changes(
      mut transports: ResMut<RuntimeTransports>,
      mut bridge: ResMut<TransportBridge>,
      mut commands: Commands,
  ) {
      // Wenn External Transport aktiviert wurde, registriere alle bestehenden Clients
      // Wenn External Transport deaktiviert wurde, disconnecte alle External Clients

      // Dies wird von ServerCommand::AddExternal/RemoveExternal getriggert
  }

  ServerCommand Integration

  // lib.rs - Erweitere ServerCommand

  pub enum ServerCommand {
      Shutdown,
      Pause,
      Resume,
      /// Aktiviere External Transport (QUIC/Steam)
      AddExternal(ExternalTransport),
      /// Deaktiviere External Transport
      RemoveExternal,
  }

  // In GameServer::run()
  match command {
      ServerCommand::AddExternal(transport) => {
          // 1. Füge Transport zu RuntimeTransports hinzu
          runtime_transports.activate_external(transport)?;

          // 2. Warm-up Transport
          // 3. Registriere in TransportBridge
      }
      ServerCommand::RemoveExternal => {
          // 1. Disconnecte alle External Clients
          // 2. Entferne aus TransportBridge
          // 3. Shutdown Transport
          runtime_transports.deactivate_external()?;
      }
  }

  ---
  🔄 Replication Setup

  GameServerPlugin mit Replicon

  // lib.rs

  use bevy::prelude::*;
  use bevy_replicon::prelude::*;

  pub struct GameServerPlugin;

  impl Plugin for GameServerPlugin {
      fn build(&self, app: &mut App) {
          app
              // === Replicon Core ===
              .add_plugins(RepliconPlugins.build()
                  .set(ServerPlugin {
                      tick_schedule: FixedUpdate.intern(),
                      ..Default::default()
                  })
              )

              // === Register Replicated Components ===
              .replicate::<Player>()
              .replicate::<Position>()
              .replicate::<Velocity>()
              .replicate::<PlayerColor>()

              // === Register Client Messages ===
              .add_client_message::<PlayerInput>(ChannelKind::Ordered)

              // === Resources ===
              .init_resource::<TransportBridge>()
              .init_resource::<RuntimeTransports>()
              .init_resource::<PlayerColorAssigner>()

              // === Systems ===
              // Transport Bridge (Early)
              .add_systems(PreUpdate, (
                  bridge_transport_to_replicon,
                  handle_transport_changes,
              ))

              // Game Logic (Update)
              .add_systems(Update, (
                  spawn_player_on_connect,
                  despawn_player_on_disconnect,
                  receive_player_inputs,
              ))

              // Movement (FixedUpdate)
              .add_systems(FixedUpdate, (
                  apply_velocity,
                  heartbeat_system,
              ))

              // Transport Bridge (PostUpdate - nach Replicon)
              .add_systems(PostUpdate, bridge_replicon_to_transport);
      }
  }

  ---
  🎯 System Implementations

  1. Player Spawning

  // systems/spawning.rs

  use bevy::prelude::*;
  use bevy_replicon::prelude::*;

  /// Spawn player wenn Client connected
  pub fn spawn_player_on_connect(
      mut commands: Commands,
      mut connect_events: EventReader<ConnectEvent>,
      mut color_assigner: ResMut<PlayerColorAssigner>,
  ) {
      for event in connect_events.read() {
          let client_id = event.client_id;
          let color = color_assigner.assign(client_id);

          // Spawn mit Replicated marker - bevy_replicon synct automatisch!
          commands.spawn((
              Replicated,  // ← WICHTIG: Markiert Entity für Replication
              Player { client_id },
              Position(Vec3::new(0.0, 2.0, 0.0)),
              Velocity(Vec3::ZERO),
              PlayerColor(color),
          ));

          info!("Spawned replicated player for client {}", client_id);
      }
  }

  /// Despawn player wenn Client disconnected
  pub fn despawn_player_on_disconnect(
      mut commands: Commands,
      mut disconnect_events: EventReader<DisconnectEvent>,
      players: Query<(Entity, &Player)>,
  ) {
      for event in disconnect_events.read() {
          for (entity, player) in &players {
              if player.client_id == event.client_id {
                  commands.entity(entity).despawn();
                  info!("Despawned player for client {}", event.client_id);
              }
          }
      }
  }

  2. Movement System

  // systems/movement.rs

  use bevy::prelude::*;
  use bevy_replicon::prelude::*;

  const MOVE_SPEED: f32 = 5.0;

  /// Receive player inputs from clients via bevy_replicon
  pub fn receive_player_inputs(
      mut server_messages: ResMut<ServerMessages>,
      mut players: Query<(&Player, &mut Velocity)>,
  ) {
      // Replicon gibt uns Iterator über (ClientId, Message)
      for (client_id, input) in server_messages.receive::<PlayerInput>() {
          // Finde Player mit dieser ClientId
          for (player, mut velocity) in &mut players {
              if player.client_id == client_id {
                  let move_vec = Vec3::new(input.direction.x, 0.0, input.direction.y);
                  velocity.0 = if move_vec.length_squared() > 0.01 {
                      move_vec.normalize() * MOVE_SPEED
                  } else {
                      Vec3::ZERO
                  };
              }
          }
      }
  }

  /// Apply velocity to position (unchanged)
  pub fn apply_velocity(
      mut query: Query<(&mut Position, &Velocity)>,
      time: Res<Time>,
  ) {
      for (mut pos, vel) in &mut query {
          pos.0 += vel.0 * time.delta_secs();
      }
  }

  ---
  📊 Runtime Endpoint Control Flow

  Szenario: Singleplayer → LAN öffnen

  1. Server läuft mit nur Loopback
     ├── RuntimeTransports { loopback: Some(...), external: None }
     └── TransportBridge registriert nur HOST_CLIENT_ID

  2. User drückt "Open to LAN"
     ├── ServerHandle::add_external(QuicServerTransport::new(...))
     └── ServerCommand::AddExternal gesendet

  3. GameServer::run() empfängt Command
     ├── RuntimeTransports::activate_external(transport)
     ├── Warm-up Transport
     └── TransportBridge bereitet sich auf neue Connections vor

  4. Remote Client verbindet sich
     ├── TransportEvent::Connected(remote_client_id)
     ├── bridge_transport_to_replicon registriert Client
     ├── Replicon triggert ConnectEvent
     └── spawn_player_on_connect spawnt Player

  5. Replication läuft
     ├── Replicon sendet Updates über ServerMessages
     ├── bridge_replicon_to_transport queued Messages
     └── TransportBridge::send_replicon_messages sendet via QUIC

  Szenario: LAN schließen

  1. User drückt "Close to LAN"
     ├── ServerHandle::remove_external()
     └── ServerCommand::RemoveExternal gesendet

  2. GameServer::run() empfängt Command
     ├── RuntimeTransports::deactivate_external()
     ├── Für jeden External Client:
     │   ├── TransportBridge::unregister_client()
     │   ├── Replicon triggert DisconnectEvent
     │   └── despawn_player_on_disconnect despawnt Player
     └── Transport shutdown

  3. Server läuft wieder nur mit Loopback
     └── RuntimeTransports { loopback: Some(...), external: None }

  ---
  🗂️ Was kann WEG?

  ❌ Zu löschen

  - network.rs - komplett (Replicon macht das)
  - protocol.rs - komplett (Component-basierte Replication)
  - PlayerInputQueue aus movement.rs (Replicon ServerMessages)

  ✅ Bleibt unverändert

  - world_setup.rs - Terrain Generation
  - savegame.rs - Savegame System
  - systems/heartbeat.rs - Heartbeat Logic
  - apply_velocity System

  ---
  📝 Migrations-Checklist

  Phase 1: Setup (2-3h)

  - Add bevy_replicon dependency
  - Create components.rs mit Serialize/Deserialize
  - Create messages.rs für PlayerInput
  - Setup GameServerPlugin mit RepliconPlugins

  Phase 2: Transport Bridge (3-4h)

  - Implement TransportBridge Resource
  - Implement bridge_transport_to_replicon System
  - Implement bridge_replicon_to_transport System
  - Test: Messages flow korrekt

  Phase 3: Runtime Endpoints (2-3h)

  - Implement RuntimeTransports Resource
  - Add activate_external() / deactivate_external()
  - Extend ServerCommand mit AddExternal/RemoveExternal
  - Test: Endpoint activation/deactivation

  Phase 4: Game Systems (2-3h)

  - Implement spawn_player_on_connect
  - Implement despawn_player_on_disconnect
  - Implement receive_player_inputs
  - Keep apply_velocity unverändert

  Phase 5: Testing (2-3h)

  - Singleplayer funktioniert
  - Open to LAN funktioniert
  - Close LAN funktioniert
  - Multiple Clients können connecten
  - Movement synct korrekt
  - No Desync

  Phase 6: Cleanup (1h)

  - Delete network.rs
  - Delete protocol.rs
  - Update Cargo.toml
  - Update docs

  ---
  🎯 Erwartete Verbesserungen

  | Metrik              | Vorher               | Nachher               | Gewinn |
  |---------------------|----------------------|-----------------------|--------|
  | LOC                 | ~800                 | ~400                  | -50%   |
  | Bandwidth           | Hoch (alle Entities) | Niedrig (nur Changes) | -60%   |
  | Complexity          | Hoch (manuell)       | Mittel (Bridge-Layer) | -40%   |
  | Runtime Flexibility | ❌ Nein               | ✅ Ja                  | +100%  |

  ---
  ⚠️ Wichtige Hinweise

  1. bevy_replicon ist Transport-agnostisch

  Du musst die Bridge selbst implementieren. bevy_replicon liefert nur das Replication-Framework, kein I/O.

  2. Channel Mapping

  // Deine Channels (networking)
  pub enum TransportChannel {
      Reliable = 0,
      Unreliable = 1,
  }

  // Replicon Channels
  ServerChannel::Updates    // Reliable (0)
  ServerChannel::Mutations  // Unreliable (1)
  ClientChannel::MutationAcks  // Reliable (0)

  3. Entity Ownership

  - Server spawnt Entity mit Replicated Component
  - Client bekommt DASSELBE Entity repliziert (nicht separates Spawn)
  - Client fügt nur Visuals hinzu (Mesh, Transform, etc.)

  4. Performance

  - Replicon sendet NUR geänderte Components
  - Nutzt Change Detection (Bevy's Changed<T>)
  - Dual-Channel: Reliable Updates + Unreliable Mutations

  ---
  Das ist der vollständige Plan! Die Hauptarbeit liegt in der Transport Bridge - alles andere ist relativ straightforward. Möchtest du mit der Implementierung anfangen? 🚀
