# Camera System Documentation

## Übersicht

Das Camera-System basiert auf einem **Single-Camera-Ansatz** mit verschiedenen Modi, die durch Components und System-Conditions gesteuert werden. Alle Modi nutzen dieselbe `SceneCamera` Entity, wechseln aber ihre Behavior-Components je nach aktivem Modus.

## Architektur

```
CameraPlugin (src/ui/cameras.rs)
├── SceneCamera (Eine Entity für alle Modi)
├── CameraMode (Resource, steuert aktiven Modus)
├── CameraTransitionState (Resource, blockiert Input während Transitions)
└── Modi:
    ├── Splashscreen (statische Position)
    ├── MainMenu (statische Position)
    └── InGame
        ├── FirstPerson (Ego-Perspektive)
        └── PanOrbit (Orbit-Kamera)
```

## Wichtige Konzepte

### 1. CameraMode Resource

```rust
#[derive(Resource, Clone, Debug, PartialEq)]
pub enum CameraMode {
    Splashscreen,
    MainMenu,
    InGame(InGameCameraMode),
}

#[derive(Clone, Debug, PartialEq)]
pub enum InGameCameraMode {
    FirstPerson,
    PanOrbit,
}
```

Die `CameraMode` Resource ist die **zentrale Steuerung**. Alle Systems nutzen `run_if` Conditions um nur im passenden Modus zu laufen.

### 2. Single Camera Entity

Es gibt **eine** `SceneCamera` Entity, die beim Start gespawnt wird:

```rust
fn spawn_scene_camera(mut commands: Commands, defaults: Res<CameraDefaults>) {
    let transform = Transform::from_translation(defaults.splashscreen.position)
        .looking_at(defaults.splashscreen.look_at, Vec3::Y);

    commands.spawn((
        Camera3d::default(),
        transform,
        SceneCamera,
        Name::new("Scene Camera"),
    ));
}
```

### 3. Mode-spezifische Components

Jeder Modus fügt seine eigenen Components zur Camera-Entity hinzu:

- **FirstPerson**: `FirstPersonView`, `FollowLocalPlayer`
- **PanOrbit**: `PanOrbitCamera`

Components werden beim Mode-Wechsel dynamisch hinzugefügt/entfernt via `add_mode_components()` und `remove_mode_components()`.

### 4. Transitions

Transitions nutzen `bevy_tweening` um smooth zwischen Modi zu animieren:

```rust
pub struct CameraModeChangeEvent {
    pub new_mode: CameraMode,
    pub animate: bool, // true = smooth transition, false = instant
}
```

**Wichtig**: Während einer aktiven Transition (`CameraTransitionState.active = true`) werden alle Mode-Systems blockiert, damit die Tween-Animation nicht überschrieben wird.

## Einen neuen Camera-Modus hinzufügen

### Schritt 1: CameraMode erweitern

**Beispiel**: Wir fügen eine ThirdPerson-Kamera hinzu.

```rust
// In src/ui/cameras.rs

#[derive(Clone, Debug, PartialEq)]
pub enum InGameCameraMode {
    FirstPerson,
    PanOrbit,
    ThirdPerson,  // NEU
}
```

### Schritt 2: Neues Modul erstellen

Erstelle `src/ui/cameras/third_person.rs`:

```rust
use super::{CameraMode, InGameCameraMode, SceneCamera};
use crate::client::LocalPlayer;
use bevy::prelude::*;
use game_server::components::Position;

/// Component für Third Person Camera
#[derive(Component)]
pub struct ThirdPersonCamera {
    pub distance: f32,
    pub height: f32,
    pub pitch: f32,
}

impl Default for ThirdPersonCamera {
    fn default() -> Self {
        Self {
            distance: 5.0,
            height: 2.0,
            pitch: 0.3,
        }
    }
}

/// Folgt dem Spieler mit Abstand
pub(super) fn follow_player_third_person(
    local_player: Query<
        (Option<&Transform>, Option<&Position>),
        (With<LocalPlayer>, Without<SceneCamera>),
    >,
    mut camera_query: Query<(&ThirdPersonCamera, &mut Transform), With<SceneCamera>>,
) {
    let Ok((transform, position)) = local_player.single() else {
        return;
    };
    let Ok((third_person, mut cam_transform)) = camera_query.single_mut() else {
        return;
    };

    let player_pos = transform
        .map(|t| t.translation)
        .or_else(|| position.map(|p| p.translation))
        .unwrap_or(Vec3::ZERO);

    // Position hinter dem Spieler
    let offset = Vec3::new(0.0, third_person.height, -third_person.distance);
    let target_pos = player_pos + offset;

    cam_transform.translation = target_pos;
    cam_transform.look_at(player_pos + Vec3::Y * 1.5, Vec3::Y);
}
```

### Schritt 3: Modul registrieren

In `src/ui/cameras.rs`:

```rust
mod defaults;
mod first_person;
mod main_menu;
mod mode_handler;
mod pan_orbit;
mod third_person;  // NEU
mod transition;
mod ui_camera;
```

### Schritt 4: Component-Switching erweitern

In `src/ui/cameras/mode_handler.rs`:

```rust
fn remove_mode_components(commands: &mut Commands, entity: Entity, mode: &CameraMode) {
    match mode {
        CameraMode::Splashscreen | CameraMode::MainMenu => {},
        CameraMode::InGame(InGameCameraMode::FirstPerson) => {
            commands
                .entity(entity)
                .remove::<FirstPersonView>()
                .remove::<FollowLocalPlayer>();
        }
        CameraMode::InGame(InGameCameraMode::PanOrbit) => {
            commands.entity(entity).remove::<PanOrbitCamera>();
        }
        // NEU
        CameraMode::InGame(InGameCameraMode::ThirdPerson) => {
            commands.entity(entity).remove::<ThirdPersonCamera>();
        }
    }
}

fn add_mode_components(
    commands: &mut Commands,
    entity: Entity,
    mode: &CameraMode,
    defaults: &CameraDefaults,
    local_player_query: &Query<...>,
) {
    match mode {
        CameraMode::Splashscreen | CameraMode::MainMenu => {},
        CameraMode::InGame(InGameCameraMode::FirstPerson) => { /* ... */ }
        CameraMode::InGame(InGameCameraMode::PanOrbit) => { /* ... */ }
        // NEU
        CameraMode::InGame(InGameCameraMode::ThirdPerson) => {
            commands.entity(entity).insert(ThirdPersonCamera::default());
        }
    }
}

fn calculate_transform_for_mode(
    mode: &CameraMode,
    defaults: &CameraDefaults,
    local_player_query: &Query<...>,
) -> Transform {
    match mode {
        CameraMode::Splashscreen => { /* ... */ }
        CameraMode::MainMenu => { /* ... */ }
        CameraMode::InGame(InGameCameraMode::FirstPerson) => { /* ... */ }
        CameraMode::InGame(InGameCameraMode::PanOrbit) => { /* ... */ }
        // NEU
        CameraMode::InGame(InGameCameraMode::ThirdPerson) => {
            let player_pos = get_player_translation(local_player_query);
            Transform::from_translation(player_pos + Vec3::new(0.0, 2.0, -5.0))
                .looking_at(player_pos + Vec3::Y * 1.5, Vec3::Y)
        }
    }
}
```

### Schritt 5: Systems registrieren

In `src/ui/cameras.rs` im `CameraPlugin`:

```rust
// ThirdPerson Mode Update-Systems
.add_systems(
    Update,
    third_person::follow_player_third_person
        .run_if(in_state(GameState::InGame))
        .run_if(resource_exists::<CameraMode>)
        .run_if(|mode: Res<CameraMode>| {
            matches!(*mode, CameraMode::InGame(InGameCameraMode::ThirdPerson))
        }),
)
```

### Schritt 6: Toggle erweitern (optional)

Wenn du zwischen Modi togglen möchtest:

```rust
fn toggle_ingame_camera_mode(
    keys: Res<ButtonInput<KeyCode>>,
    transition_state: Res<CameraTransitionState>,
    current_mode: Res<CameraMode>,
    mut events: MessageWriter<CameraModeChangeEvent>,
) {
    if transition_state.active {
        return;
    }

    if keys.just_pressed(KeyCode::KeyC) {
        if let CameraMode::InGame(ingame_mode) = current_mode.as_ref() {
            let new_ingame_mode = match ingame_mode {
                InGameCameraMode::FirstPerson => InGameCameraMode::PanOrbit,
                InGameCameraMode::PanOrbit => InGameCameraMode::ThirdPerson, // NEU
                InGameCameraMode::ThirdPerson => InGameCameraMode::FirstPerson, // NEU
            };

            events.write(CameraModeChangeEvent {
                new_mode: CameraMode::InGame(new_ingame_mode),
                animate: true,
            });
        }
    }
}
```

### Schritt 7: Defaults hinzufügen (optional)

In `src/ui/cameras/defaults.rs`:

```rust
pub struct ThirdPersonDefaults {
    pub distance: f32,
    pub height: f32,
    pub pitch: f32,
}

impl Default for ThirdPersonDefaults {
    fn default() -> Self {
        Self {
            distance: 5.0,
            height: 2.0,
            pitch: 0.3,
        }
    }
}

#[derive(Resource)]
pub struct CameraDefaults {
    pub splashscreen: SplashscreenDefaults,
    pub main_menu: MainMenuDefaults,
    pub first_person: FirstPersonDefaults,
    pub pan_orbit: PanOrbitDefaults,
    pub third_person: ThirdPersonDefaults, // NEU
}
```

## Wichtige Hinweise

### ⚠️ Transition-Blockierung

**KRITISCH**: Wenn dein neuer Modus ein System hat, das die `Transform` manipuliert, muss es während Transitions blockiert werden:

```rust
.add_systems(
    Update,
    your_system
        .run_if(|transition: Res<CameraTransitionState>| !transition.active)
)
```

**Warum?** Sonst überschreibt dein System die Tween-Animation jeden Frame und die Transition springt am Ende.

### 🎯 System Scheduling

- **Update**: Für Logic (Input, Berechnungen)
- **PostUpdate**: Für Transform-Manipulation (nach Physics, vor Rendering)
- **before(TransformSystems::Propagate)**: Wenn Transform vor Hierarchie-Updates gesetzt werden muss

Beispiel PanOrbit:

```rust
.add_systems(
    PostUpdate,
    (
        (active_viewport_data, mouse_key_tracker, touch_tracker),
        pan_orbit_camera,
    )
        .chain()
        .before(TransformSystems::Propagate)
        .before(CameraUpdateSystems)
)
```

### 🔄 Mode-Wechsel triggern

Es gibt zwei Wege:

**1. Via GameState** (automatisch bei State-Transitions):

```rust
.add_systems(OnEnter(GameState::InGame), switch_to_ingame_mode)
```

**2. Via Event** (manuell, z.B. bei Key-Press):

```rust
events.write(CameraModeChangeEvent {
    new_mode: CameraMode::InGame(InGameCameraMode::ThirdPerson),
    animate: true, // false für instant wechsel
});
```

## PanOrbit Integration

Das PanOrbit-System ist **vollständig integriert** (kein externes Plugin mehr):

```
src/ui/cameras/pan_orbit/
├── mod.rs (Haupt-System)
├── input.rs (Maus/Keyboard Input)
├── touch.rs (Touch-Gesten)
├── traits.rs (Helper-Traits)
└── util.rs (Math-Utilities)
```

### Warum integriert?

1. ✅ **Volle Kontrolle** über alle Aspekte des Verhaltens
2. ✅ **Direkte Integration** mit CameraMode-System
3. ✅ **Keine Konflikte** mit Transition-System
4. ✅ **Anpassbar** für projektspezifische Anforderungen

## Debugging

### Camera Position ausgeben

```rust
fn debug_camera(query: Query<(&Transform, &CameraMode), With<SceneCamera>>) {
    if let Ok((transform, mode)) = query.single() {
        info!("Camera: {:?} at {:?}", mode, transform.translation);
    }
}
```

### Transition State checken

```rust
fn debug_transition(state: Res<CameraTransitionState>) {
    if state.active {
        info!("Transition active!");
    }
}
```

## Best Practices

1. **Immer `run_if` nutzen** für mode-spezifische Systems
2. **Transitions blockieren** wenn Transform manipuliert wird
3. **Defaults in `CameraDefaults`** für einfache Tweaks
4. **Player-Position via Query** nicht direkt auf Transform zugreifen
5. **Component-Cleanup** in `remove_mode_components` nicht vergessen

## Weitere Ressourcen

- `defaults.rs` - Alle Kamera-Parameter
- `mode_handler.rs` - Component-Switching Logik
- `transition.rs` - Deprecated, nur für Referenz
- `ReadCamera.md` - Empfohlene Kamera-Einstellungen aus Game-Design

## TODOs / Zukünftige Optimierungen

- [ ] PanOrbit's eigenes Smoothing für Transitions nutzen (statt bevy_tweening)
- [ ] Component-Switching durch Mode-basierte Logik ersetzen
- [ ] Camera FOV/ADS System hinzufügen
- [ ] Camera Shake System
- [ ] Collision Detection für alle Modi
