use bevy::{
    color::palettes::basic::{BLUE, GREEN, RED, YELLOW},
    prelude::*,
};

#[derive(Component, Debug)]
pub struct LocalCoordinateSystem {
    pub origin: Vec3,
    pub right: Option<Vec3>,
    pub up: Option<Vec3>,
    pub forward: Option<Vec3>,
    pub length: f32,
}

impl Default for LocalCoordinateSystem {
    fn default() -> Self {
        Self {
            origin: Vec3::ZERO,
            right: Some(Vec3::X),
            up: Some(Vec3::Y),
            forward: Some(Vec3::Z),
            length: 1.0,
        }
    }
}

pub fn draw_local_coordinate_systems(query: Query<&LocalCoordinateSystem>, mut gizmos: Gizmos) {
    for coord_system in query.iter() {
        if let Some(right) = coord_system.right {
            gizmos.line(
                coord_system.origin,
                coord_system.origin + right.normalize() * coord_system.length,
                RED,
            );
        }
        if let Some(up) = coord_system.up {
            gizmos.line(
                coord_system.origin,
                coord_system.origin + up.normalize() * coord_system.length,
                GREEN,
            );
        }
        if let Some(forward) = coord_system.forward {
            gizmos.line(
                coord_system.origin,
                coord_system.origin + forward.normalize() * coord_system.length,
                BLUE,
            );
        }
    }
}
