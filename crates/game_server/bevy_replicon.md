  Bevy_replicon Datenaustausch - Übersicht

  Bevy_replicon bietet drei Hauptmechanismen für den Datenaustausch:

  1. Component-Replikation (Automatische Synchronisation)

  Die Basis für die automatische Replikation von Komponenten zwischen Server und Client:

  Setup:
  // In deiner App-Initialisierung
  app.replicate::<Position>()          // Automatische Replikation bei Änderung
     .replicate::<Health>()
     .replicate_once::<PlayerName>();  // Nur einmal senden

  Verwendung:
  // Server: Entity mit Replicated-Marker erstellen
  commands.spawn((
      Replicated,  // Markiert Entity für Replikation
      Position(Vec2::ZERO),
      Health(100),
      PlayerName("Alice".to_string()),
  ));

  // Client: Komponenten werden automatisch synchronisiert!

  Wichtige Konzepte:
  - Replicated Component markiert Entities für Replikation
  - ReplicationMode::OnChange - Sendet bei jeder Änderung (Standard)
  - ReplicationMode::Once - Sendet nur einmal beim Spawnen

  2. Messages (Explizite Nachrichten mit Antwort-Semantik)

  Für Request-Response Muster oder wenn du Daten explizit senden möchtest:

  Server → Client Messages:

  // Registrierung:
  app.add_server_message::<ChatMessage>(Channel::Ordered);

  // Server sendet:
  fn send_chat(mut messages: Commands) {
      messages.trigger(ToClients {
          mode: SendMode::Broadcast,  // An alle
          event: ChatMessage("Hello!".to_string()),
      });
  }

  // Client empfängt:
  fn receive_chat(trigger: Trigger<ChatMessage>) {
      println!("Received: {}", trigger.event().0);
  }

  Client → Server Messages:

  // Registrierung:
  app.add_client_message::<PlayerInput>(Channel::Unreliable);

  // Client sendet:
  fn send_input(mut messages: Commands) {
      messages.trigger(PlayerInput {
          direction: Vec2::new(1.0, 0.0)
      });
  }

  // Server empfängt:
  fn receive_input(trigger: Trigger<FromClient<PlayerInput>>) {
      let client_id = trigger.event().client_id;
      let input = &trigger.event().event;
      println!("Client {:?} moved {:?}", client_id, input.direction);
  }

  3. Events (Fire-and-Forget Ereignisse)

  Für einmalige Ereignisse ohne Antwort-Erwartung:

  Server → Client Events:

  // Registrierung:
  app.add_server_event::<ExplosionEvent>(Channel::Ordered);

  // Server triggert:
  fn trigger_explosion(mut commands: Commands) {
      commands.trigger(ToClients {
          mode: SendMode::Broadcast,
          event: ExplosionEvent { position: Vec3::ZERO },
      });
  }

  // Client reagiert:
  fn on_explosion(trigger: Trigger<ExplosionEvent>) {
      // Spiele Explosion-Effekt ab
  }

  Client → Server Events:

  // Registrierung:
  app.add_client_event::<PlayerJumpEvent>(Channel::Ordered);

  // Client triggert:
  fn jump(mut commands: Commands) {
      commands.trigger(PlayerJumpEvent);
  }

  // Server reagiert:
  fn on_player_jump(trigger: Trigger<FromClient<PlayerJumpEvent>>) {
      let client_id = trigger.event().client_id;
      // Validiere und führe Jump aus
  }

  Channel-Typen:

  pub enum Channel {
      Unreliable,  // Schnell, aber keine Garantie
      Ordered,     // Garantierte Reihenfolge
      Unordered,   // Garantierte Zustellung, aber nicht Reihenfolge
  }

  Entity-Mapping:

  Wenn deine Messages/Components Entities enthalten:

  #[derive(Component, Serialize, Deserialize)]
  struct TargetComponent {
      #[entities]  // Annotiere Entities!
      target: Entity,
  }

  // Registrierung mit Mapping:
  app.add_mapped_server_message::<TargetComponent>(Channel::Ordered);
  app.replicate::<TargetComponent>();  // Automatisches Mapping für Components

  Fortgeschrittene Serialisierung:

  Für custom Serialisierung (z.B. Quantisierung):

  app.replicate_as::<Position, QuantizedPosition>();

  // Oder mit komplett eigenen Funktionen:
  app.replicate_with(RuleFns::new(
      serialize_custom,
      deserialize_custom,
  ));

  Wichtige Unterschiede:

  | Feature   | Messages                | Events          | Component-Replikation    |
  |-----------|-------------------------|-----------------|--------------------------|
  | Semantik  | Request/Response        | Fire-and-Forget | Automatische Sync        |
  | Timing    | Explizit                | Explizit        | Automatisch bei Änderung |
  | Best für  | Spieler-Input, Commands | Sounds, VFX     | Positions, Health, State |
  | Buffering | Ja (an Tick gebunden)   | Ja              | Ja                       |

  Möchtest du, dass ich mir deine aktuelle Implementierung in
  crates/game_server/src/messages.rs anschaue und Verbesserungsvorschläge mache?
