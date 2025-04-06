// src/simulation/events/mod.rs
pub mod genetics; // Bezieht sich auf die umbenannte genetics.rs
                  // Re-exportiere die Event-Typen für einfacheren Zugriff
pub use genetics::*;
