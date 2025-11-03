# Bevy_replicon Wissensdatenbank - Vollständige Referenz

## 📚 Inhaltsverzeichnis
1. [Drei Hauptmechanismen für Datenaustausch](#drei-hauptmechanismen)
2. [Server-Client Lifecycle](#server-client-lifecycle)
3. [Observer Pattern & Trigger System](#observer-pattern)
4. [Channel-Typen & Netzwerk-Architektur](#channel-typen)
5. [Praktische Patterns & Best Practices](#best-practices)
6. [Fehlerbehandlung & Debugging](#fehlerbehandlung)

---

## 🔄 Drei Hauptmechanismen für Datenaustausch {#drei-hauptmechanismen}

### 1. Component-Replikation (Automatische Synchronisation)

Die Basis für die automatische Replikation von Komponenten zwischen Server und Client.

#### Setup & Registrierung:
```rust
// In deiner App-Initialisierung (Server UND Client)
app.add_plugins(RepliconPlugins)
   .replicate::<Position>()           // Automatische Replikation bei Änderung
   .replicate::<Health>()
   .replicate_once::<PlayerName>();   // Nur einmal beim Spawn senden
```

#### Verwendung auf dem Server:
```rust
fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Replicated,  // ⚠️ WICHTIG: Markiert Entity für Replikation
        Position { translation: Vec3::ZERO },
        Health(100),
        PlayerName("Alice".to_string()),
    ));
}
```

#### Client-seitige Verarbeitung:
```rust
// Komponenten werden automatisch synchronisiert!
// Du kannst replizierte Entities normal abfragen:
fn render_players(
    players: Query<(&Position, &Health), With<Player>>,
) {
    for (pos, health) in &players {
        // Render Spieler an pos.translation
    }
}
```

#### Wichtige Konzepte:
- **`Replicated` Component** markiert Entities für Replikation
- **`ReplicationMode::OnChange`** - Sendet bei jeder Änderung (Standard)
- **`ReplicationMode::Once`** - Sendet nur einmal beim Spawnen
- **Server-Authoritative** - Nur Server-Änderungen werden repliziert

---

### 2. Messages (Explizite Nachrichten mit Antwort-Semantik)

Für hochfrequente oder strukturierte Kommunikation (z.B. Player Input).

#### Server → Client Messages:

```rust
// 1. Message-Typ definieren (in shared Crate)
#[derive(Message, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub text: String,
    pub sender: String,
}

// 2. Registrierung (Server UND Client)
app.add_server_message::<ChatMessage>(Channel::Ordered);

// 3. Server sendet Message mit Observer-Trigger
fn broadcast_chat(mut commands: Commands) {
    commands.trigger(ToClients {
        mode: SendMode::Broadcast,        // An alle Clients
        message: ChatMessage {
            text: "Hello World!".to_string(),
            sender: "Server".to_string(),
        },
    });
}

// 4. Client empfängt mit Observer
fn receive_chat(trigger: Trigger<ChatMessage>) {
    let msg = trigger.event();
    println!("{}: {}", msg.sender, msg.text);
}

// Observer registrieren:
app.observe(receive_chat);
```

#### Client → Server Messages:

```rust
// 1. Message-Typ definieren
#[derive(Message, Serialize, Deserialize, Clone)]
pub struct PlayerInput {
    pub direction: Vec2,
    pub jump: bool,
}

// 2. Registrierung (Server UND Client)
app.add_client_message::<PlayerInput>(Channel::Unreliable);

// 3. Client sendet Input
fn send_player_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let direction = Vec2::new(
        keyboard.pressed(KeyCode::KeyD) as i32 as f32 -
        keyboard.pressed(KeyCode::KeyA) as i32 as f32,
        keyboard.pressed(KeyCode::KeyW) as i32 as f32 -
        keyboard.pressed(KeyCode::KeyS) as i32 as f32,
    ).normalize_or_zero();

    commands.trigger(PlayerInput {
        direction,
        jump: keyboard.pressed(KeyCode::Space),
    });
}

// 4. Server empfängt mit FromClient<T> Observer
fn process_input(trigger: Trigger<FromClient<PlayerInput>>) {
    let FromClient { client_id, message } = trigger.event();

    // client_id identifiziert den sendenden Client
    println!("Client {:?} input: {:?}", client_id, message.direction);

    // Finde Spieler-Entity für diesen Client
    // ... (siehe Server-Pattern unten)
}

// Observer registrieren:
app.observe(process_input);
```

---

### 3. Events (Fire-and-Forget Ereignisse)

Für einmalige Ereignisse ohne Antwort-Erwartung (z.B. Sound-Effekte, Explosionen).

#### Server → Client Events:

```rust
// 1. Event-Typ definieren
#[derive(Event, Serialize, Deserialize, Clone)]
pub struct ExplosionEvent {
    pub position: Vec3,
    pub radius: f32,
}

// 2. Registrierung
app.add_server_event::<ExplosionEvent>(Channel::Ordered);

// 3. Server triggert Event
fn trigger_explosion(mut commands: Commands) {
    commands.trigger(ToClients {
        mode: SendMode::Broadcast,
        event: ExplosionEvent {
            position: Vec3::new(10.0, 0.0, 5.0),
            radius: 5.0,
        },
    });
}

// 4. Client reagiert
fn on_explosion(trigger: Trigger<ExplosionEvent>) {
    let explosion = trigger.event();
    // Spiele Partikel-Effekt ab
    // Spiele Sound ab
}

app.observe(on_explosion);
```

#### Client → Server Events:

```rust
// 1. Event definieren
#[derive(Event, Serialize, Deserialize, Clone)]
pub struct PlayerJumpEvent;

// 2. Registrierung
app.add_client_event::<PlayerJumpEvent>(Channel::Ordered);

// 3. Client triggert
fn handle_jump_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        commands.trigger(PlayerJumpEvent);
    }
}

// 4. Server validiert und führt aus
fn on_player_jump(trigger: Trigger<FromClient<PlayerJumpEvent>>) {
    let client_id = trigger.event().client_id;

    // Validiere: Ist Spieler auf dem Boden?
    // Wenn ja, apply Jump-Force
}

app.observe(on_player_jump);
```

---

## 🚀 Server-Client Lifecycle {#server-client-lifecycle}

### Server States

```rust
pub enum ServerState {
    Stopped,   // Server ist inaktiv
    Running,   // Server akzeptiert Connections
}
```

#### Server-Startup Pattern:

```rust
fn setup_server(mut commands: Commands) {
    // 1. RepliconPlugins hinzufügen (macht dies automatisch)
    // 2. Backend-Ressourcen hinzufügen (z.B. RenetServer)
    // 3. Server-State wird automatisch auf Running gesetzt

    info!("Server started on port 5000");
}

// Systems nur im Running-State ausführen:
app.add_systems(
    Update,
    game_logic.run_if(in_state(ServerState::Running))
);
```

#### Server-Shutdown Pattern:

```rust
fn shutdown_server(
    mut commands: Commands,
    mut server_state: ResMut<NextState<ServerState>>,
) {
    // Setze State auf Stopped
    server_state.set(ServerState::Stopped);

    // Backend-Ressourcen werden automatisch entfernt
    // Alle Clients werden disconnected
}
```

### Client States

```rust
pub enum ClientState {
    Disconnected,  // Nicht verbunden
    Connecting,    // Verbindungsaufbau
    Connected,     // Verbunden mit Server
}
```

#### Client-Connection Pattern:

```rust
fn connect_to_server(mut commands: Commands) {
    // 1. Backend-Transport einrichten (z.B. NetcodeClientTransport)
    // 2. State wechselt automatisch: Disconnected -> Connecting -> Connected
}

// Systems nur bei Verbindung ausführen:
app.add_systems(
    Update,
    send_inputs.run_if(in_state(ClientState::Connected))
);
```

#### Client-Disconnection:

```rust
fn disconnect(
    mut commands: Commands,
    mut client_state: ResMut<NextState<ClientState>>,
) {
    client_state.set(ClientState::Disconnected);
    // Transport wird automatisch aufgeräumt
}
```

### ConnectedClient Component

Jeder verbundene Client wird als Entity mit `ConnectedClient` Component repräsentiert:

```rust
// Server-seitig:
fn handle_new_clients(
    new_clients: Query<Entity, Added<ConnectedClient>>,
    client_ids: Query<&NetworkId>,
) {
    for client_entity in &new_clients {
        let network_id = client_ids.get(client_entity).unwrap();

        info!("New client connected: {:?}", network_id.get());

        // Spawn Spieler für diesen Client
        commands.spawn((
            Player,
            PlayerOwner { client_entity }, // Link zu ConnectedClient
            Replicated,
        ));
    }
}
```

### DisconnectRequest

Server kann Clients disconnecten:

```rust
fn kick_player(
    mut commands: Commands,
    players: Query<&PlayerOwner>,
) {
    // Finde Client-Entity für Spieler
    for owner in &players {
        commands.trigger_targets(
            DisconnectRequest {
                client: owner.client_entity,
            },
            owner.client_entity,
        );
    }
}
```

---

## 🎯 Observer Pattern & Trigger System {#observer-pattern}

Bevy_replicon nutzt Bevys **Observer System** für Message/Event-Handling.

### Was sind Observer?

Observer sind Callback-Funktionen, die automatisch ausgeführt werden, wenn ein Event getriggert wird.

```rust
// Observer registrieren:
app.observe(my_observer_system);

// System-Signatur für Observer:
fn my_observer_system(trigger: Trigger<MyEvent>) {
    // trigger.event() gibt Zugriff auf Event-Daten
    let event_data = trigger.event();
}
```

### Trigger Extensions

#### ClientTriggerExt - Client sendet an Server

```rust
use bevy_replicon::prelude::*;

fn send_to_server(mut commands: Commands) {
    // Sendet PlayerInput an Server
    commands.trigger(PlayerInput {
        direction: Vec2::ZERO,
        jump: false,
    });
}
```

#### ServerTriggerExt - Server sendet an Clients

```rust
fn send_to_clients(mut commands: Commands) {
    // Broadcast an alle Clients
    commands.trigger(ToClients {
        mode: SendMode::Broadcast,
        message: ChatMessage { text: "Hello!".into() },
    });

    // Nur an einen spezifischen Client
    commands.trigger(ToClients {
        mode: SendMode::Direct(client_entity),
        message: PrivateMessage { .. },
    });

    // An alle außer einem
    commands.trigger(ToClients {
        mode: SendMode::BroadcastExcept(excluded_client),
        message: PlayerLeftMessage { .. },
    });
}
```

### FromClient Wrapper

Server empfängt Client-Messages immer als `FromClient<T>`:

```rust
fn handle_client_message(trigger: Trigger<FromClient<PlayerInput>>) {
    // Extrahiere client_id und message
    let FromClient { client_id, message } = trigger.event();

    // client_id ist ClientId enum:
    match client_id {
        ClientId::Server => {
            // Message von lokaler Server-Instanz (Listen-Server)
        }
        _ => {
            // Normaler Client
            let client_entity = client_id.entity().unwrap();
        }
    }
}
```

### ToClients SendMode

```rust
pub enum SendMode {
    /// An alle Clients
    Broadcast,

    /// An alle außer einem
    BroadcastExcept(Entity),  // Entity ist ConnectedClient

    /// Nur an einen spezifischen Client
    Direct(Entity),
}
```

---

## 📡 Channel-Typen & Netzwerk-Architektur {#channel-typen}

### Channel-Delivery-Garantien

```rust
pub enum Channel {
    /// Unreliable und unordered - Schnellste Option
    /// ⚠️ Packets können verloren gehen oder in falscher Reihenfolge ankommen
    Unreliable,

    /// Reliable aber unordered
    /// ✅ Garantiert dass alle Packets ankommen
    /// ⚠️ Reihenfolge nicht garantiert
    Unordered,

    /// Reliable UND ordered - Langsamste Option
    /// ✅ Garantiert Zustellung UND Reihenfolge
    Ordered,
}
```

### Channel-Auswahl nach Use-Case

| Use Case | Empfohlener Channel | Begründung |
|----------|---------------------|------------|
| Player Input (WASD) | `Unreliable` | Hochfrequent, alte Daten werden überschrieben |
| Jump Event | `Ordered` | Muss ankommen und in korrekter Reihenfolge |
| Chat Message | `Ordered` | Muss ankommen und in korrekter Reihenfolge |
| Sound Effect Event | `Unordered` | Muss ankommen, Reihenfolge egal |
| Component Updates | `Automatic` | Replicon managed dies intern |

### Interne Replicon Channels

#### ServerChannel (Server → Client):

```rust
pub enum ServerChannel {
    /// Für Entity Mappings, Inserts, Removals, Despawns
    /// - Reliable & Ordered
    Updates,

    /// Für Component Mutations (Wert-Updates)
    /// - Unreliable (neuester Wert überschreibt alten)
    Mutations,
}
```

**Warum zwei Channels?**

Replicon nutzt ein **Dual-Channel System** für optimale Performance:

1. **Updates Channel (Reliable)**:
   - Entity-Creation, Component-Insertion, Removals
   - Muss ankommen, sonst brechen Referenzen
   - Atomic updates pro Tick

2. **Mutations Channel (Unreliable)**:
   - Component-Wert-Updates (z.B. Position-Änderungen)
   - Kann verloren gehen → nächstes Update überschreibt
   - Split über MTU-Größe für partielle Anwendung

**Beispiel:**
```rust
// Tick 1: Spawn Player (Updates Channel)
commands.spawn((Replicated, Player, Position(Vec3::ZERO)));

// Tick 2-100: Position Updates (Mutations Channel)
position.translation.x += 0.1; // Jeder Tick

// Wenn Tick 50 verloren geht → egal, Tick 51 überschreibt
```

#### ClientChannel (Client → Server):

```rust
pub enum ClientChannel {
    /// Für Acks von empfangenen Mutations
    /// - Reliable & Ordered
    MutationAcks,
}
```

---

## 💡 Best Practices & Patterns {#best-practices}

### 1. Message vs Event - Wann was verwenden?

**Messages verwenden für:**
- Spieler-Input (hochfrequent)
- Commands (z.B. "Use Item")
- Daten die der Server verarbeiten muss

**Events verwenden für:**
- Fire-and-forget Notifications
- Sound/VFX Triggers
- UI-Updates

**Component-Replikation verwenden für:**
- Game State (Position, Health, etc.)
- Automatische Synchronisation
- State der sich regelmäßig ändert

### 2. Client-Owned Entities Pattern

```rust
// Server: Link Player zu ConnectedClient
#[derive(Component)]
pub struct PlayerOwner {
    pub client_entity: Entity,
}

// Beim Spawn
commands.spawn((
    Player,
    PlayerOwner { client_entity },  // Nicht repliziert!
    Replicated,
));

// Bei Input-Handling
fn process_input(
    trigger: Trigger<FromClient<PlayerInput>>,
    mut players: Query<(&PlayerOwner, &mut Velocity)>,
) {
    let client_id = trigger.event().client_id;
    let client_entity = client_id.entity().unwrap();

    // Finde Spieler für diesen Client
    for (owner, mut velocity) in &mut players {
        if owner.client_entity == client_entity {
            // Update velocity basierend auf Input
        }
    }
}
```

### 3. Local Player Marker Pattern

```rust
// Client: Markiere eigenen Spieler
#[derive(Component)]
pub struct LocalPlayer;

fn mark_local_player(
    mut commands: Commands,
    local_client_id: Res<LocalClientId>,
    players: Query<(Entity, &PlayerIdentity), Without<LocalPlayer>>,
) {
    for (entity, identity) in &players {
        if identity.client_id == local_client_id.0 {
            commands.entity(entity).insert(LocalPlayer);
        }
    }
}

// Nutze LocalPlayer für Client-only Rendering
fn render_local_ui(
    local_player: Query<&Health, With<LocalPlayer>>,
) {
    if let Ok(health) = local_player.get_single() {
        // Zeige Health Bar nur für eigenen Spieler
    }
}
```

### 4. Entity Mapping Pattern

Für Messages/Components mit Entity-Referenzen:

```rust
use bevy::ecs::entity::MapEntities;

#[derive(Message, Serialize, Deserialize)]
pub struct AttackCommand {
    #[entities]  // ⚠️ WICHTIG: Annotiere Entities!
    pub target: Entity,
}

// Registrierung mit Mapping:
app.add_mapped_client_message::<AttackCommand>(Channel::Ordered);
```

**Ohne `#[entities]`**: Entity IDs stimmen nicht zwischen Client und Server überein!

### 5. Independent Messages Pattern

Standardmäßig sind Server-Messages an Replications-Ticks gebunden:

```rust
// Message wartet auf Entity-Updates bevor sie ausgeführt wird
app.add_server_message::<ChatMessage>(Channel::Ordered);

// ✅ Wenn Message KEINE Entities referenziert, markiere als independent:
app.add_server_message::<ChatMessage>(Channel::Ordered)
   .make_message_independent::<ChatMessage>();
```

**Vorteil**: Message wird sofort ausgeführt, keine Wartezeit auf Entity-Sync.

### 6. Error Handling Pattern

```rust
fn process_input(
    trigger: Trigger<FromClient<PlayerInput>>,
    players: Query<(&PlayerOwner, &mut Velocity)>,
) {
    let FromClient { client_id, message } = trigger.event();

    // ⚠️ Validierung: Client-Entity existiert?
    let Some(client_entity) = client_id.entity() else {
        warn!("Received input from invalid client: {:?}", client_id);
        return;
    };

    // ⚠️ Validierung: Spieler existiert?
    let mut found = false;
    for (owner, mut velocity) in &mut players {
        if owner.client_entity == client_entity {
            // Process input
            found = true;
            break;
        }
    }

    if !found {
        warn!("No player found for client {:?}", client_id);
    }
}
```

### 7. Graceful Disconnect Pattern

```rust
// Server: Cleanup bei Disconnect
fn handle_disconnects(
    mut commands: Commands,
    mut removed: RemovedComponents<ConnectedClient>,
    players: Query<(Entity, &PlayerOwner)>,
) {
    for disconnected_client in removed.read() {
        // Finde alle Entities die diesem Client gehören
        for (player_entity, owner) in &players {
            if owner.client_entity == disconnected_client {
                // Despawn Player
                commands.entity(player_entity).despawn();
            }
        }

        info!("Client {:?} disconnected, cleaned up", disconnected_client);
    }
}
```

---

## 🐛 Fehlerbehandlung & Debugging {#fehlerbehandlung}

### Häufige Fehler

#### 1. "Message not registered"
```rust
// ❌ Falsch: Nur auf einer Seite registriert
// Server:
app.add_client_message::<PlayerInput>(Channel::Unreliable);

// ✅ Richtig: Auf BEIDEN Seiten registrieren
// Server:
app.add_client_message::<PlayerInput>(Channel::Unreliable);
// Client:
app.add_client_message::<PlayerInput>(Channel::Unreliable);
```

#### 2. "Entity not found" bei Mapping
```rust
// ❌ Falsch: #[entities] Annotation fehlt
#[derive(Message, Serialize, Deserialize)]
pub struct AttackCommand {
    pub target: Entity,  // ❌ Entity wird nicht gemapped!
}

// ✅ Richtig:
#[derive(Message, Serialize, Deserialize)]
pub struct AttackCommand {
    #[entities]  // ✅ Entity wird automatisch gemapped
    pub target: Entity,
}

// UND:
app.add_mapped_client_message::<AttackCommand>(Channel::Ordered);
```

#### 3. Observer wird nicht aufgerufen
```rust
// ❌ Falsch: Observer nicht registriert
fn my_handler(trigger: Trigger<MyEvent>) { }

// ✅ Richtig:
app.observe(my_handler);
```

#### 4. Messages kommen nicht an
```rust
// Debugging:
fn debug_messages(
    trigger: Trigger<FromClient<PlayerInput>>,
) {
    info!("Received input: {:?}", trigger.event());  // Wird nie geloggt?
}

// Checklist:
// 1. Message auf beiden Seiten registriert?
// 2. Observer registriert?
// 3. Client im Connected State?
// 4. Server im Running State?
// 5. Netzwerk-Backend korrekt eingerichtet?
```

### Debug-Hilfsmittel

```rust
// State-Debugging:
fn debug_states(
    server_state: Option<Res<State<ServerState>>>,
    client_state: Option<Res<State<ClientState>>>,
) {
    if let Some(state) = server_state {
        info!("Server State: {:?}", state.get());
    }
    if let Some(state) = client_state {
        info!("Client State: {:?}", state.get());
    }
}

// Connected Clients überwachen:
fn log_clients(
    clients: Query<(Entity, &NetworkId), With<ConnectedClient>>,
) {
    for (entity, network_id) in &clients {
        info!("Client {:?} - NetworkId: {}", entity, network_id.get());
    }
}
```

---

## 📋 Cheat Sheet

### Schnell-Referenz für häufige Tasks

#### Server starten:
```rust
app.add_plugins(RepliconPlugins)
   .replicate::<MyComponent>()
   .add_systems(Startup, setup_server);
```

#### Client verbinden:
```rust
app.add_plugins(RepliconPlugins)
   .replicate::<MyComponent>()
   .add_systems(OnEnter(GameState::InGame), connect_to_server);
```

#### Message senden (Client → Server):
```rust
fn send(mut commands: Commands) {
    commands.trigger(MyMessage { data: 42 });
}
```

#### Message empfangen (Server):
```rust
fn receive(trigger: Trigger<FromClient<MyMessage>>) {
    let msg = &trigger.event().message;
}
app.observe(receive);
```

#### Message senden (Server → Client):
```rust
fn send(mut commands: Commands) {
    commands.trigger(ToClients {
        mode: SendMode::Broadcast,
        message: MyMessage { data: 42 },
    });
}
```

#### Message empfangen (Client):
```rust
fn receive(trigger: Trigger<MyMessage>) {
    let msg = trigger.event();
}
app.observe(receive);
```

---

## 📚 Weiterführende Ressourcen

- [Offizielle Replicon Docs](https://docs.rs/bevy_replicon)
- [Replicon GitHub Examples](https://github.com/simgine/bevy_replicon/tree/master/bevy_replicon_example_backend/examples)
- [Bevy ECS Observer Guide](https://bevyengine.org/learn/book/ecs/)
