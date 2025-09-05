/*
Forge of Stories – Dedicated Server entry (skeleton)

This file is intentionally small. The actual networking (QUIC), control-plane
(UDS + axum), and session management live in the `crates/network/server` crate.

How to run a minimal server right now:
- Build: cargo run -p server
- Behavior: Starts a skeleton runtime that prints a liveness line and waits for Ctrl+C
- Control-plane: On unix, exposes /v1/health on a Unix Domain Socket (default path is derived from env or temp dir)

Architecture overview:
1) Data-Plane (QUIC) – quinn + rustls
   - Connection accept loop, length-prefixed bincode framing, streams topology (bidi control, uni broadcast)
2) Protocol Layer – shared types in crates/network/shared
   - Version handshake and simple Auth (token/join code)
3) Control-Plane (UDS) – axum HTTP API
   - GET /v1/health, more endpoints to be added
4) Domain Layer (Aether/Bevy)
   - ECS systems communicate with the transport via channels
5) Settings
   - Bind address/port and toggles will live in server settings; defaults are used when absent

Why is this in forge_of_stories/src/server.rs?
- This binary is a placeholder shim. It exists so the workspace can provide a 'server' entry point.
- The heavy lifting is implemented in the server crate; this file shouldn't duplicate that logic.
*/

fn main() {
    println!("Forge of Stories server shim.");
    println!("The dedicated server currently lives in `crates/network/server`.");
    println!("Run: cargo run -p server");
}
