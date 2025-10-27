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
