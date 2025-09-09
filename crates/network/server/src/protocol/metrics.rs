/*!
Network Metrics (Forge of Stories – Server)

Scope (M1):
- Minimal counters for handshake & session lifecycle.
- Periodic dump system for logging (trace level recommended).
- Event-driven session counter adjustments.

Integration:
- Add `NetMetrics` + `NetMetricsDumpTimer` as resources during plugin startup.
- Register `metrics_session_events_system` (after handshake systems) so session
  establish/close events update `active_sessions`.
- Handshake code should increment the appropriate counters (see field names).
  (Current handshake implementation has TODO tags; integrate by mutating `ResMut<NetMetrics>`.)

Logging Target:
- server::net::metrics

Extensibility:
- Later extend with histograms (RTT, frame sizes), gauges (pending handshakes),
  and structured export (Prometheus / tracing metrics).
*/

use bevy::prelude::*;
use std::time::{Duration, Instant};

use crate::protocol::handshake::{NetSessionClosed, NetSessionEstablished};

/// Core network metric counters.
/// All fields are simple u64 counters (wrapping not expected in normal operation).
#[derive(Resource, Debug, Default)]
pub struct NetMetrics {
    // Handshake outcomes
    pub handshake_success: u64,
    pub handshake_fail_version: u64,
    pub handshake_fail_auth: u64,
    pub handshake_fail_rate_limited: u64,
    pub handshake_fail_malformed: u64,
    pub handshake_fail_internal: u64,
    pub handshake_timeout: u64,

    // Active sessions (updated via events)
    pub active_sessions: u64,
}

impl NetMetrics {
    pub fn reset(&mut self) {
        *self = NetMetrics::default();
    }
}

/// Timer resource controlling metrics dump cadence.
#[derive(Resource, Debug)]
pub struct NetMetricsDumpTimer {
    pub interval: Duration,
    pub last: Instant,
}

impl NetMetricsDumpTimer {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last: Instant::now(),
        }
    }
}

/// Setup helper – insert metrics resources with default interval (e.g. 10s).
pub fn setup_metrics_resources(commands: &mut Commands, interval: Duration) {
    commands.init_resource::<NetMetrics>();
    commands.insert_resource(NetMetricsDumpTimer::new(interval));
}

/// System: Adjust active session gauge based on establish / close events.
pub fn metrics_session_events_system(
    mut metrics: ResMut<NetMetrics>,
    mut ev_est: EventReader<NetSessionEstablished>,
    mut ev_closed: EventReader<NetSessionClosed>,
) {
    for _ in ev_est.read() {
        metrics.active_sessions = metrics.active_sessions.saturating_add(1);
    }
    for _ in ev_closed.read() {
        metrics.active_sessions = metrics.active_sessions.saturating_sub(1);
    }
}

/// System: Periodically dump metrics to log.
/// Keep this lightweight; avoid expensive formatting or allocations.
pub fn metrics_dump_system(metrics: Res<NetMetrics>, mut timer: ResMut<NetMetricsDumpTimer>) {
    let now = Instant::now();
    if now.duration_since(timer.last) < timer.interval {
        return;
    }
    timer.last = now;

    bevy::log::trace!(
        target: "server::net::metrics",
        "metrics: hs_ok={} hs_ver={} hs_auth={} hs_rate={} hs_malformed={} hs_internal={} hs_timeout={} active_sessions={}",
        metrics.handshake_success,
        metrics.handshake_fail_version,
        metrics.handshake_fail_auth,
        metrics.handshake_fail_rate_limited,
        metrics.handshake_fail_malformed,
        metrics.handshake_fail_internal,
        metrics.handshake_timeout,
        metrics.active_sessions
    );
}

// -------------------------------------------------------------------------------------------------
// Tests (basic behavior)
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::handshake::{NetSessionClosed, NetSessionEstablished};

    #[test]
    fn session_event_adjusts_active_counter() {
        let mut app = App::new();
        app.add_event::<NetSessionEstablished>();
        app.add_event::<NetSessionClosed>();
        app.insert_resource(NetMetrics::default());
        app.insert_resource(NetMetricsDumpTimer::new(Duration::from_secs(60)));
        app.add_systems(Update, metrics_session_events_system);

        // Fire establish events
        app.world_mut().send_event(NetSessionEstablished(1));
        app.world_mut().send_event(NetSessionEstablished(2));

        app.update();
        assert_eq!(
            app.world()
                .get_resource::<NetMetrics>()
                .unwrap()
                .active_sessions,
            2
        );

        // Fire close
        app.world_mut().send_event(NetSessionClosed(1));
        app.world_mut().send_event(NetSessionClosed(2));

        app.update();
        assert_eq!(
            app.world()
                .get_resource::<NetMetrics>()
                .unwrap()
                .active_sessions,
            0
        );
    }

    #[test]
    fn dump_respects_interval() {
        let mut app = App::new();
        app.insert_resource(NetMetrics::default());
        // Very short interval to trigger quickly
        app.insert_resource(NetMetricsDumpTimer::new(Duration::from_millis(1)));
        app.add_systems(Update, metrics_dump_system);

        // First update will likely dump
        app.update();
        // Sleep to exceed interval
        std::thread::sleep(std::time::Duration::from_millis(2));
        app.update();

        // No assertions on log output (not easily captured), ensure timer updated
        let timer = app.world().get_resource::<NetMetricsDumpTimer>().unwrap();
        assert!(timer.last.elapsed() < Duration::from_secs(1));
    }
}
