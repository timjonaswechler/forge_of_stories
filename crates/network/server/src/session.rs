//! Session Registry (Forge of Stories – Network Server)
//!
//! Purpose:
//! - Maintain mapping of logical game sessions to underlying transport connections.
//! - Provide fast lookup by `SessionId` and reverse lookup by `ConnectionId`.
//! - Supply allocation of monotonically increasing session identifiers.
//!
//! Responsibilities (M1):
//! - Insert session after successful handshake.
//! - Remove session on disconnect (future system will hook provider disconnect events).
//! - Track minimal metadata (timestamps, provider kind).
//!
//! Not (yet) included:
//! - Persistent session resumption
//! - Multi-connection (multi-endpoint) per session
//! - Heartbeat / liveness timestamps updates (will be extended when frame handling progresses)
//!
//! Integration Points:
//! - Handshake system calls `allocate_session_id()` then `insert()`.
//! - Future disconnect system will call `remove_by_conn()` and emit `NetSessionClosed` event.
//!
//! Concurrency / Threading:
//! - Stored as a Bevy `Resource` – mutated only on main thread systems currently.
//! - If off-thread access is needed later, wrap in `Arc<Mutex<...>>` (not required for M1).
//!
//! Logging Targets Suggested:
//! - server::net::session (insert/remove operations)
//!
//! TODO Tags:
//! - TODO(M1-SESS-METRICS): Hook into metrics counters upon insert/remove
//! - TODO(M1-SESS-LIVENESS): Update `last_frame_at` when decoded transport frames arrive
//! - TODO(M1-SESS-LIMITS): Enforce global / per-provider session limits here (currently enforced during handshake)
//!
//! (C) Forge of Stories
use std::collections::HashMap;
use std::time::{Instant, SystemTime};

use bevy::prelude::*;

use crate::transport::{ConnectionId, ProviderKind};

/// Opaque server-assigned session identifier.
///
/// Chosen as `u64` for large lifetime space and simple serialization.
/// (Future: could adopt snowflake / timestamp embedding if needed.)
pub type SessionId = u64;

/// Metadata about a live session.
#[derive(Debug, Clone)]
pub struct SessionMeta {
    pub id: SessionId,
    pub connection_id: ConnectionId,
    pub provider: ProviderKind,
    pub established_at: Instant,
    pub established_wall: SystemTime,
    pub last_frame_at: Instant,
}

impl SessionMeta {
    pub fn new(id: SessionId, connection_id: ConnectionId, provider: ProviderKind) -> Self {
        let now = Instant::now();
        Self {
            id,
            connection_id,
            provider,
            established_at: now,
            established_wall: SystemTime::now(),
            last_frame_at: now,
        }
    }
}

/// Central registry of active sessions.
///
/// Fields:
/// - `sessions`: Primary store keyed by `SessionId`.
/// - `by_conn`: Reverse index for quick removal on provider disconnect.
/// - `next_session`: Monotonic allocator (wrapping unlikely in practical runtime).
#[derive(Resource, Debug)]
pub struct SessionRegistry {
    pub sessions: HashMap<SessionId, SessionMeta>,
    pub by_conn: HashMap<ConnectionId, SessionId>,
    next_session: SessionId,
    pub max_sessions: u32,
}

impl Default for SessionRegistry {
    fn default() -> Self {
        Self::new(100_000)
    }
}

impl SessionRegistry {
    /// Create with a specified upper bound (soft) on sessions; enforcement currently occurs
    /// in the handshake layer before insertion.
    pub fn new(max_sessions: u32) -> Self {
        Self {
            sessions: HashMap::new(),
            by_conn: HashMap::new(),
            next_session: 1,
            max_sessions,
        }
    }

    /// Allocate a new unique session id (monotonic).
    pub fn allocate_session_id(&mut self) -> SessionId {
        let id = self.next_session;
        // Simple monotonic increment; wrapping not handled (extremely long uptime scenario).
        self.next_session = self.next_session.wrapping_add(1);
        id
    }

    /// Insert a new session (caller guarantees no conflict).
    pub fn insert(&mut self, id: SessionId, conn: ConnectionId, provider: ProviderKind) {
        let meta = SessionMeta::new(id, conn, provider);
        self.sessions.insert(id, meta);
        self.by_conn.insert(conn, id);
        bevy::log::debug!(
            target:"server::net::session",
            "Session inserted id={} conn={:?} provider={:?} (total={})",
            id,
            conn,
            provider,
            self.sessions.len()
        );
        // TODO(M1-SESS-METRICS): increment metrics.handshake_success / active_sessions gauge (currently via events)
    }

    /// Remove session by its id.
    pub fn remove(&mut self, id: SessionId) -> Option<SessionMeta> {
        if let Some(meta) = self.sessions.remove(&id) {
            self.by_conn.remove(&meta.connection_id);
            bevy::log::debug!(
                target:"server::net::session",
                "Session removed id={} conn={:?} (remaining={})",
                id,
                meta.connection_id,
                self.sessions.len()
            );
            Some(meta)
        } else {
            None
        }
    }

    /// Remove session associated with a transport connection.
    pub fn remove_by_conn(&mut self, conn: ConnectionId) -> Option<SessionMeta> {
        if let Some(sid) = self.by_conn.remove(&conn) {
            self.sessions.remove(&sid).map(|meta| {
                bevy::log::debug!(
                    target:"server::net::session",
                    "Session removed by connection conn={:?} id={} (remaining={})",
                    conn,
                    sid,
                    self.sessions.len()
                );
                meta
            })
        } else {
            None
        }
    }

    /// Update liveness timestamp (e.g. upon receiving a valid frame for that session).
    pub fn touch(&mut self, sid: SessionId) {
        if let Some(meta) = self.sessions.get_mut(&sid) {
            meta.last_frame_at = Instant::now();
        }
    }

    /// Fetch immutable session metadata.
    pub fn get(&self, sid: SessionId) -> Option<&SessionMeta> {
        self.sessions.get(&sid)
    }

    /// Fetch by connection id (reverse index).
    pub fn get_by_conn(&self, conn: ConnectionId) -> Option<&SessionMeta> {
        self.by_conn
            .get(&conn)
            .and_then(|sid| self.sessions.get(sid))
    }

    /// Current count of active sessions.
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

// -------------------------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{ConnectionId, ProviderKind};

    #[test]
    fn allocate_and_insert_session() {
        let mut reg = SessionRegistry::new(10);
        let id = reg.allocate_session_id();
        reg.insert(id, ConnectionId(7), ProviderKind::Quic);
        assert_eq!(reg.len(), 1);
        let meta = reg.get(id).expect("session meta");
        assert_eq!(meta.id, id);
        assert_eq!(meta.connection_id.0, 7);
    }

    #[test]
    fn remove_by_id() {
        let mut reg = SessionRegistry::new(10);
        let id = reg.allocate_session_id();
        reg.insert(id, ConnectionId(1), ProviderKind::Local);
        assert!(reg.remove(id).is_some());
        assert!(reg.get(id).is_none());
        assert!(reg.by_conn.get(&ConnectionId(1)).is_none());
    }

    #[test]
    fn remove_by_connection() {
        let mut reg = SessionRegistry::new(10);
        let id = reg.allocate_session_id();
        reg.insert(id, ConnectionId(99), ProviderKind::Steam);
        assert!(reg.remove_by_conn(ConnectionId(99)).is_some());
        assert!(reg.get(id).is_none());
    }

    #[test]
    fn touch_updates_last_frame_at() {
        let mut reg = SessionRegistry::new(10);
        let id = reg.allocate_session_id();
        reg.insert(id, ConnectionId(5), ProviderKind::Quic);
        let before = reg.get(id).unwrap().last_frame_at;
        std::thread::sleep(std::time::Duration::from_millis(5));
        reg.touch(id);
        let after = reg.get(id).unwrap().last_frame_at;
        assert!(after > before);
    }
}
