use bevy::prelude::*;

use super::interaction::IO;
use super::ui_style::Style; // Importiere IO und Style aus dem ui_style Modul // Importiere IO aus dem interaction Modul

#[derive(Resource, Debug)] // Default könnte man manuell implementieren oder von Style/IO ableiten
pub struct NodesSettings {
    pub io: IO,
    pub style: Style,
}

// Manuelle Implementierung von Default, falls nötig
impl Default for NodesSettings {
    fn default() -> Self {
        Self {
            io: IO::default(),       // Nimmt Default von IO
            style: Style::default(), // Nimmt Default von Style (z.B. blender_light)
        }
    }
}
