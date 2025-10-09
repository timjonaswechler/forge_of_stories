//! Core server gameplay systems.

use bevy::prelude::*;

/// Local state for heartbeat tracking.
#[derive(Default)]
pub struct HeartbeatState {
    start_time: Option<std::time::Duration>,
    last_tick: Option<std::time::Duration>,
    tick_count: u64,
}

/// Server heartbeat system that logs tick timing information.
///
/// This system runs in FixedUpdate and provides diagnostic information about
/// the server's tick rate stability.
pub fn heartbeat_system(
    mut state: Local<HeartbeatState>,
    time_fixed: Res<Time<Fixed>>,
    time_real: Res<Time<Real>>,
) {
    let target_delta = time_fixed.delta();
    let target_ms = target_delta.as_secs_f64() * 1000.0;

    let now = time_real.elapsed();

    if state.start_time.is_none() {
        state.start_time = Some(now);
        state.last_tick = Some(now);
        state.tick_count = 0;
        tracing::debug!("Server heartbeat initialized");
        return;
    }

    let last_tick = state.last_tick.unwrap();
    let real_delta = now.saturating_sub(last_tick);
    let real_ms = real_delta.as_secs_f64() * 1000.0;

    state.tick_count += 1;
    state.last_tick = Some(now);

    let uptime_secs = (now - state.start_time.unwrap()).as_secs_f64();

    // Sende strukturierte Metrik-Events mit Target "metrics::heartbeat"

    // Optional weiterhin lesbare Logs in höheren Leveln ausgeben, z.B. alle 100 Ticks
    if state.tick_count % 100 == 0 {
        tracing::info!(
            "Server tick #{} | target: {:.2}ms | actual: {:.2}ms | uptime: {:.1}s",
            state.tick_count,
            target_ms,
            real_ms,
            uptime_secs
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_system() {
        let mut app = App::new();
        app.insert_resource(Time::<Fixed>::from_hz(20.0))
            .add_systems(FixedUpdate, heartbeat_system);

        // Run a few ticks
        for _ in 0..5 {
            app.update();
        }

        // Test passes if no panic
    }
}
