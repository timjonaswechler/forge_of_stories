use bevy::prelude::*;

/// Zentrale Kamera-Defaultwerte (aus ReadCamera.md abgeleitet)
#[derive(Resource, Clone)]
pub struct CameraDefaults {
    pub first_person: FirstPersonDefaults,
    pub pan_orbit: PanOrbitDefaults,
}

#[derive(Clone)]
pub struct FirstPersonDefaults {
    /// Augenhöhe
    pub height_offset: f32,
    /// Maus-Sensitivität (radians per pixel)
    pub mouse_sensitivity: f32,
}

#[derive(Clone)]
pub struct PanOrbitDefaults {
    /// Fokus-Höhe relativ zum Player-Root
    pub focus_height: f32,
    /// Winkel:
    /// - yaw: 0.0 blickt entlang -Z (links/rechts Orbit um Y)
    /// - pitch: 0.0 ist waagerecht; positive pitch schaut nach unten (-Y)
    pub yaw: f32,
    pub pitch: f32,
    pub radius: f32,
    /// Limits
    pub zoom_min: f32,
    pub zoom_max: f32,
    pub pitch_min: f32,
    pub pitch_max: f32,
    /// Eingabe-Sensitivitäten
    pub orbit_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub zoom_sensitivity: f32,
}

impl Default for CameraDefaults {
    fn default() -> Self {
        Self {
            first_person: FirstPersonDefaults {
                height_offset: 1.7,       // 1.6–1.8m → 1.7 Standard
                mouse_sensitivity: 0.002, // kompatibel zu aktueller Steuerung
            },
            pan_orbit: PanOrbitDefaults {
                focus_height: 1.2,
                // yaw/pitch-Semantik: yaw=0.0 -> Blick entlang -Z; pitch>0 -> Blick nach unten
                yaw: 0.0,
                pitch: 0.35, // ~20° leicht nach unten
                radius: 6.0,
                zoom_min: 1.5,
                zoom_max: 35.0,
                // Pitch-Bereich: leicht über Horizont bis deutlich nach unten
                pitch_min: -0.3,
                pitch_max: 1.2,
                orbit_sensitivity: 1.5,
                pan_sensitivity: 0.5,
                zoom_sensitivity: 0.5,
            },
        }
    }
}
