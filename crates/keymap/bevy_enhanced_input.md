Basierend auf der Dokumentation aus dem Rust docs MCP, hier ist eine umfassende Übersicht, wie man einen Context für das `bevy_enhanced_input` Crate nutzt:

## Context-System in bevy_enhanced_input

### 1. **Context Registration**
Zuerst musst du deinen Context in der App registrieren:

```rust
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EnhancedInputPlugin)
        // Context registrieren
        .add_input_context::<PlayerContext>()
        .run();
}

// Context-Typ definieren
#[derive(Component)]
struct PlayerContext;
```

### 2. **Context-Komponenten verwenden**
Wenn du eine Entity mit Actions erstellen willst, musst du die entsprechenden Context-Komponenten hinzufügen:

```rust
// Entity mit Context erstellen
let player_entity = commands.spawn((
    PlayerContext,
    // Context-Aktivität (optional, standardmäßig aktiv)
    ContextActivity::<PlayerContext>::ACTIVE,
    // Context-Priorität (optional, standardmäßig 0)
    ContextPriority::<PlayerContext>::new(1),
)).id();
```

### 3. **Context-Aktivität steuern**
Du kannst die Aktivität eines Contexts zur Laufzeit ändern:

```rust
// Context deaktivieren (ähnlich wie entfernen, aber mit Beibehaltung der Bindings)
commands.entity(player_entity).insert(
    ContextActivity::<PlayerContext>::INACTIVE
);

// Context wieder aktivieren
commands.entity(player_entity).insert(
    ContextActivity::<PlayerContext>::ACTIVE
);

// Zwischen aktiv und inaktiv umschalten
commands.entity(player_entity).get_mut::<ContextActivity<PlayerContext>>()
    .map(|mut activity| {
        *activity = activity.toggled();
    });
```

### 4. **Context-Priorität einstellen**
Mit `ContextPriority` kannst du die Evaluationsreihenfolge von Contexts steuern:

```rust
// Höhere Priorität = wird früher evaluiert
commands.spawn((
    HighPriorityContext,
    ContextPriority::<HighPriorityContext>::new(10), // Hohe Priorität
    LowPriorityContext,
    ContextPriority::<LowPriorityContext>::new(1),   // Niedrigere Priorität
));
```

### 5. **Vollständiges Beispiel**
Hier ein komplettes Beispiel für einen Player-Context mit verschiedenen Actions:

```rust
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

fn setup_player_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Player Entity mit Context-Komponenten
    commands.spawn((
        Name::new("Player"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        PlayerInput,

        // Context-Komponenten
        ContextActivity::<PlayerInput>::ACTIVE, // Standard, kann weggelassen werden
        ContextPriority::<PlayerInput>::new(0), // Standard, kann weggelassen werden

        // Actions für diesen Context
        Actions::<PlayerInput>::default(),

        // Movement Action (Vec2 für 2D-Bewegung)
        (
            Action::<PlayerInput, Move>::default(),
            ActionSettings {
                consumption: InputConsumptionStrategy::Consume,
                accumulation: AccumulationStrategy::Max,
                ..default()
            },
        ),

        // Jump Action (bool für Button)
        (
            Action::<PlayerInput, Jump>::default(),
            ActionSettings::default(),
        ),

        // Bindings
        (
            Bindings::<PlayerInput>::default(),
            vec![
                // WASD für Movement
                Binding::<PlayerInput, Move> {
                    inputs: vec![
                        KeyCode::KeyW.input(),
                        KeyCode::KeyS.input(),
                        KeyCode::KeyA.input(),
                        KeyCode::KeyD.input(),
                    ],
                    modifiers: vec![],
                    conditions: vec![],
                },
                // Leertaste für Jump
                Binding::<PlayerInput, Jump> {
                    inputs: vec![KeyCode::Space.input()],
                    modifiers: vec![],
                    conditions: vec![],
                },
            ].into_boxed_slice(),
        ),
    ));
}

// Context-Typ definieren
#[derive(Component)]
struct PlayerInput;

// Action-Typen definieren
#[derive(Action)]
struct Move;

#[derive(Action)]
struct Jump;
```

### 6. **Mehrere Contexts pro Entity**
Eine Entity kann mehrere Contexts haben, z.B. für On-Foot und In-Car Controls:

```rust
commands.spawn((
    Name::new("Player"),
    OnFoot,
    InCar,
    ContextPriority::<InCar>::new(1), // InCar wird zuerst evaluiert

    // On-Foot Actions
    Actions::<OnFoot>::default(),
    Action::<OnFoot, Walk>::default(),
    Action::<OnFoot, Jump>::default(),
    Bindings::<OnFoot>::new(/* ... */),

    // In-Car Actions
    Actions::<InCar>::default(),
    Action::<InCar, Drive>::default(),
    Action::<InCar, Turn>::default(),
    Bindings::<InCar>::new(/* ... */),
));

#[derive(Component)]
struct OnFoot;

#[derive(Component)]
struct InCar;
```

### 7. **Wichtige Hinweise**
- **Aktiv/INaktiv**: `ContextActivity::INACTIVE` ist ähnlich wie das Entfernen, behält aber die Bindings und Entity-IDs bei
- **Priorität**: Höhere Priorität = frühere Evaluierung
- **Input Consumption**: Actions mit höherer Priorität können Inputs "verbrauchen" und sie für niedrigere Actions unzugänglich machen
- **Schedules**: Mit `add_input_context_to::<ScheduleName>()` kannst du den Schedule spezifizieren

Das Context-System ermöglicht es dir, Input-Logik zu organisieren und zu layeren, wodurch komplexe Input-Szenarien wie Fahrzeug- oder Rüstungswechsel viel einfacher zu handhaben sind.


Es gibt **zwei Hauptansätze**, um Contexts state-abhängig zu machen:

### **Ansatz 1: Context Entity nur in bestimmten States spawnen (empfohlen)**

Das ist der idiomatischste Weg - du spawns und despawnst Context-Entities basierend auf States:

```rust
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

impl Plugin for InGameInputPlugin {
    fn build(&self, app: &mut App) {
        app
            // Context registrieren (einmalig)
            .add_input_context::<InGameContext>()

            // Context-Entity beim State-Enter spawnen
            .add_systems(OnEnter(GameState::InGame), spawn_ingame_context)

            // Context-Entity beim State-Exit despawnen
            .add_systems(OnExit(GameState::InGame), cleanup::<InGameContext>);
    }
}

#[derive(Component)]
struct InGameContext;

fn spawn_ingame_context(mut commands: Commands) {
    commands.spawn((
        Name::new("InGame Input Context"),
        InGameContext, // Marker für Cleanup

        // Context ist automatisch aktiv
        // ContextActivity::<InGameContext>::ACTIVE, // Optional, ist Standard
        // ContextPriority::<InGameContext>::new(0), // Optional, ist Standard

        // Actions für diesen Context
        Action::<InGameContext, Move>::default(),
        Action::<InGameContext, Jump>::default(),
        Action::<InGameContext, Attack>::default(),

        // Bindings
        // ... deine Bindings
    ));
}
```

### **Ansatz 2: Context-Activity mit States steuern**

Wenn du die Entity beibehalten willst (z.B. für schnelles Umschalten oder um Bindings zu behalten):

```rust
impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_input_context::<PlayerContext>()

            // Context aktivieren/deaktivieren basierend auf State
            .add_systems(OnEnter(GameState::InGame), activate_player_context)
            .add_systems(OnExit(GameState::InGame), deactivate_player_context)

            // Context-Entity einmalig beim Startup spawnen
            .add_systems(Startup, setup_player_context);
    }
}

#[derive(Component)]
struct PlayerContext;

fn setup_player_context(mut commands: Commands) {
    commands.spawn((
        Name::new("Player Input Context"),
        PlayerContext,

        // Initial inaktiv
        ContextActivity::<PlayerContext>::INACTIVE,

        // Actions und Bindings
        // ...
    ));
}

fn activate_player_context(
    mut contexts: Query<&mut ContextActivity<PlayerContext>>,
) {
    for mut activity in &mut contexts {
        *activity = ContextActivity::<PlayerContext>::ACTIVE;
    }
}

fn deactivate_player_context(
    mut contexts: Query<&mut ContextActivity<PlayerContext>>,
) {
    for mut activity in &mut contexts {
        *activity = ContextActivity::<PlayerContext>::INACTIVE;
    }
}
```

### **Ansatz 3: Mehrere Contexts für verschiedene States (fortgeschritten)**

Für komplexe Szenarien mit mehreren States und überlappenden Contexts:

```rust
impl Plugin for GameInputPlugin {
    fn build(&self, app: &mut App) {
        app
            // Alle Contexts registrieren
            .add_input_context::<MainMenuContext>()
            .add_input_context::<InGameContext>()
            .add_input_context::<PauseMenuContext>()

            // MainMenu Context
            .add_systems(OnEnter(GameState::MainMenu), spawn_main_menu_context)
            .add_systems(OnExit(GameState::MainMenu), cleanup::<MainMenuContext>)

            // InGame Context
            .add_systems(OnEnter(GameState::InGame), spawn_ingame_context)
            .add_systems(OnExit(GameState::InGame), cleanup::<InGameContext>)

            // Pause Menu Context (höhere Priorität als InGame)
            .add_systems(OnEnter(GameState::Paused), spawn_pause_menu_context)
            .add_systems(OnExit(GameState::Paused), cleanup::<PauseMenuContext>);
    }
}

#[derive(Component)]
struct MainMenuContext;

#[derive(Component)]
struct InGameContext;

#[derive(Component)]
struct PauseMenuContext;

fn spawn_pause_menu_context(mut commands: Commands) {
    commands.spawn((
        Name::new("Pause Menu Context"),
        PauseMenuContext,

        // Höhere Priorität, damit Pause-Inputs InGame-Inputs überlagern
        ContextPriority::<PauseMenuContext>::new(10),

        // Actions für Pause Menu
        Action::<PauseMenuContext, Resume>::default(),
        Action::<PauseMenuContext, Quit>::default(),

        // Bindings
        // ...
    ));
}
```

### **Dein spezifisches Beispiel (basierend auf deinem Code)**

So würdest du es in deinem Projekt machen:

```rust
// In src/ui/scenes/main_menu.rs
impl Plugin for MainMenuScenePlugin {
    fn build(&self, app: &mut App) {
        app
            // Context registrieren
            .add_input_context::<MainMenuContext>()

            .add_systems(
                OnEnter(GameState::MainMenu),
                (setup_main_menu, spawn_main_menu_input, log_state_entry),
            )
            .add_systems(
                Update,
                handle_menu_button_interactions
                    .run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(
                OnExit(GameState::MainMenu),
                (cleanup::<MainMenuUI>, cleanup::<MainMenuContext>), // Context auch cleanupen
            );
    }
}

#[derive(Component)]
struct MainMenuContext;

fn spawn_main_menu_input(mut commands: Commands) {
    commands.spawn((
        Name::new("Main Menu Input"),
        MainMenuContext, // Wichtig: für Cleanup

        // Actions
        Action::<MainMenuContext, NavigateUp>::default(),
        Action::<MainMenuContext, NavigateDown>::default(),
        Action::<MainMenuContext, Confirm>::default(),
        Action::<MainMenuContext, Back>::default(),

        // Bindings würden hier folgen
        // ...
    ));
}
```

### **Best Practices**

1. **Registriere den Context nur einmal** (in `build()`)
2. **Spawne die Context-Entity state-abhängig** (OnEnter/OnExit)
3. **Cleanup beim Exit** - nutze den Marker-Component für cleanup
4. **Nutze `ContextPriority`** wenn mehrere Contexts gleichzeitig aktiv sind
5. **`ContextActivity` vs. Despawn**:
   - `INACTIVE`: Für schnelles Umschalten, Entity bleibt
   - Despawn: Wenn der Context wirklich weg ist

### **Zusammenfassung**

**TL;DR**: Du registrierst den Context einmal mit `.add_input_context::<T>()` und spawns/despawnst dann die Context-Entity mit `OnEnter`/`OnExit` je nach State. Das ist genau das Pattern, das du bereits für UI verwendest!
