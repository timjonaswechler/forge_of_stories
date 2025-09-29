//! Sichtbarkeits- und Discovery-Logik für den Server.
//!
//! Geplante API:
//! - `VisibilityState` Enum (Hidden, LanVisible, SteamVisible).
//! - LAN-Broadcaster (UDP/mDNS) und Steam-Lobby-Ankündigungen.
//! - Hooks, um Moduswechsel zur Laufzeit zu triggern.
