pub mod settings_bridge;

use bevy::log::LogPlugin;
use bevy::prelude::*;
use std::time::Duration;
use tracing;

use crate::bevy::settings_bridge::AetherSettingsPlugin;

// 1) Labels für unsere Pipeline
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum NetSets {
    Receive,     // Tokio → ECS (Inbound drain)
    Simulation,  // Gameplay (Aether)
    Replication, // Snapshots/Deltas bauen
    Send,        // ECS → Tokio (Outbound flush)
    Control,     // Control-Plane (Separate)
}

// Optional: Control-Plane separat
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ControlSet {
    Apply,
}

#[derive(Default)]
pub(crate) struct HeartbeatStats {
    start_real: Option<Duration>,
    last_real: Option<Duration>,
    ticks: u64,
}

// 2) Minimaler „Proof of Life“ im FixedUpdate
pub(crate) fn heartbeat_fixed(
    mut stats: Local<HeartbeatStats>,
    time_fixed: Res<Time<Fixed>>,
    time_real: Res<Time<Real>>,
) {
    let target = time_fixed.delta();
    let target_s = target.as_secs_f64();
    let target_ms = target_s * 1000.0;

    let now_real = time_real.elapsed();

    if stats.start_real.is_none() {
        stats.start_real = Some(now_real);
        stats.last_real = Some(now_real);
        stats.ticks = 0;
        return; // Warmup: noch kein valider Delta/Drift
    }

    let last_real = stats.last_real.unwrap();
    let real_delta = now_real.saturating_sub(last_real);
    let real_delta_s = real_delta.as_secs_f64();
    let real_delta_ms = real_delta_s * 1000.0;

    tracing::debug!(
        "fixed_tick={} target=({:.3} ms) real=({:.3} ms)",
        stats.ticks + 1,
        target_ms,
        real_delta_ms,
    );

    // Schritt als abgeschlossen markieren
    stats.ticks += 1;
    stats.last_real = Some(now_real);
}

// 3) Bevy-App Aufbau mit 30 Hz FixedUpdate (ohne UI)
pub(crate) fn build_bevy_app(app: &mut App) {
    app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>())
        .add_plugins(AetherSettingsPlugin)
        .insert_resource(Time::<Fixed>::from_hz(10.0))
        .configure_sets(
            FixedUpdate,
            (
                NetSets::Receive,
                NetSets::Simulation,
                NetSets::Replication,
                NetSets::Send,
                NetSets::Control,
            )
                .chain(),
        )
        .configure_sets(FixedUpdate, ControlSet::Apply)
        // Demo-System: zeigt den 30-Hz-Takt
        .add_systems(FixedUpdate, heartbeat_fixed.in_set(NetSets::Simulation));
}
