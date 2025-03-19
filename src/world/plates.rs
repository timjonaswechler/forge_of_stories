use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

/// Component for a tectonic plate
#[derive(Component)]
pub struct TectonicPlate {
    pub id: usize,
    pub is_oceanic: bool,
    pub movement_x: f32,            // Movement vector X (-1.0 to 1.0)
    pub movement_y: f32,            // Movement vector Y (-1.0 to 1.0)
    pub tiles: Vec<(usize, usize)>, // Coordinates of tiles belonging to this plate
}

/// Resource containing all plate data
#[derive(Resource)]
pub struct PlateData {
    pub plates: Vec<TectonicPlate>,
    pub plate_map: Vec<Vec<usize>>, // Maps each tile to a plate ID
}

impl Default for PlateData {
    fn default() -> Self {
        Self {
            plates: Vec::new(),
            plate_map: Vec::new(),
        }
    }
}

/// Generate initial tectonic plates
pub fn generate_plates(width: usize, height: usize, num_plates: usize, seed: u64) -> PlateData {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut plates = Vec::with_capacity(num_plates);
    let mut plate_map = vec![vec![0; width]; height];

    // Generate plate centers
    let mut centers = Vec::with_capacity(num_plates);
    for i in 0..num_plates {
        let x = rng.gen_range(0..width);
        let y = rng.gen_range(0..height);
        let is_oceanic = rng.gen_bool(0.7); // 70% chance of being oceanic

        centers.push((x, y, is_oceanic));

        plates.push(TectonicPlate {
            id: i,
            is_oceanic,
            movement_x: rng.gen_range(-1.0..1.0),
            movement_y: rng.gen_range(-1.0..1.0),
            tiles: Vec::new(),
        });
    }

    // Assign each tile to nearest plate center (simple Voronoi)
    for y in 0..height {
        for x in 0..width {
            let mut min_dist = f32::MAX;
            let mut nearest_plate = 0;

            for (i, &(center_x, center_y, _)) in centers.iter().enumerate() {
                let dx = (x as isize - center_x as isize)
                    .abs()
                    .min((width as isize - (x as isize - center_x as isize).abs()))
                    as f32;
                let dy = (y as isize - center_y as isize).abs() as f32;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < min_dist {
                    min_dist = dist;
                    nearest_plate = i;
                }
            }

            plate_map[y][x] = nearest_plate;
            plates[nearest_plate].tiles.push((x, y));
        }
    }

    PlateData { plates, plate_map }
}

/// Calculate elevation based on plate tectonics
pub fn calculate_elevation_from_plates(
    plate_data: &PlateData,
    width: usize,
    height: usize,
) -> Vec<Vec<f32>> {
    let mut elevation = vec![vec![0.0; width]; height];
    let mut boundary_strength = vec![vec![0.0; width]; height];

    // Set base elevation based on oceanic vs continental plates
    for y in 0..height {
        for x in 0..width {
            let plate_id = plate_data.plate_map[y][x];
            let is_oceanic = plate_data.plates[plate_id].is_oceanic;

            // Oceanic plates start with lower elevation
            elevation[y][x] = if is_oceanic { 0.2 } else { 0.6 };
        }
    }

    // Calculate plate boundaries and their effects
    for y in 0..height {
        for x in 0..width {
            let current_plate = plate_data.plate_map[y][x];
            let mut is_boundary = false;
            let mut boundary_value = 0.0;

            // Check neighboring cells (including wrapping)
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let nx = (x as isize + dx).rem_euclid(width as isize) as usize;
                    let ny = (y as isize + dy).rem_euclid(height as isize) as usize;

                    let neighbor_plate = plate_data.plate_map[ny][nx];

                    if neighbor_plate != current_plate {
                        is_boundary = true;

                        // Calculate convergence/divergence based on movement vectors
                        let current_movement_x = plate_data.plates[current_plate].movement_x;
                        let current_movement_y = plate_data.plates[current_plate].movement_y;

                        let neighbor_movement_x = plate_data.plates[neighbor_plate].movement_x;
                        let neighbor_movement_y = plate_data.plates[neighbor_plate].movement_y;

                        // Direction vector from current to neighbor
                        let dir_x = dx as f32;
                        let dir_y = dy as f32;
                        let dir_len = (dir_x * dir_x + dir_y * dir_y).sqrt();
                        let dir_x = dir_x / dir_len;
                        let dir_y = dir_y / dir_len;

                        // Project movement vectors onto direction
                        let current_proj = current_movement_x * dir_x + current_movement_y * dir_y;
                        let neighbor_proj =
                            neighbor_movement_x * dir_x + neighbor_movement_y * dir_y;

                        // Relative movement: positive for convergence, negative for divergence
                        let rel_movement = current_proj - neighbor_proj;

                        // Convergent boundaries create mountains (especially continental-continental)
                        // Divergent boundaries create rifts/ridges
                        if rel_movement > 0.0 {
                            // Convergent
                            let is_current_oceanic = plate_data.plates[current_plate].is_oceanic;
                            let is_neighbor_oceanic = plate_data.plates[neighbor_plate].is_oceanic;

                            if !is_current_oceanic && !is_neighbor_oceanic {
                                // Continental collision - high mountains
                                boundary_value += 0.5 * rel_movement;
                            } else if is_current_oceanic && !is_neighbor_oceanic
                                || !is_current_oceanic && is_neighbor_oceanic
                            {
                                // Subduction zone - mountains on continental side, trench on oceanic
                                if !is_current_oceanic {
                                    boundary_value += 0.3 * rel_movement;
                                } else {
                                    boundary_value -= 0.2 * rel_movement;
                                }
                            } else {
                                // Oceanic collision - island arcs
                                boundary_value += 0.1 * rel_movement;
                            }
                        } else {
                            // Divergent
                            let is_current_oceanic = plate_data.plates[current_plate].is_oceanic;
                            let is_neighbor_oceanic = plate_data.plates[neighbor_plate].is_oceanic;

                            if !is_current_oceanic && !is_neighbor_oceanic {
                                // Continental rift - lower elevation
                                boundary_value -= 0.2 * -rel_movement;
                            } else {
                                // Mid-ocean ridge - slight elevation
                                boundary_value += 0.1 * -rel_movement;
                            }
                        }
                    }
                }
            }

            if is_boundary {
                boundary_strength[y][x] = boundary_value;
            }
        }
    }

    // Apply boundary effects to elevation with some smoothing
    for y in 0..height {
        for x in 0..width {
            if boundary_strength[y][x] != 0.0 {
                // Apply direct influence
                elevation[y][x] += boundary_strength[y][x];

                // Spread effect to neighbors with decay
                let spread_radius = 3;
                for dy in -spread_radius..=spread_radius {
                    for dx in -spread_radius..=spread_radius {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        let nx = (x as isize + dx).rem_euclid(width as isize) as usize;
                        let ny = (y as isize + dy).rem_euclid(height as isize) as usize;

                        let distance = ((dx * dx + dy * dy) as f32).sqrt();
                        let decay = 1.0 / (1.0 + distance);

                        elevation[ny][nx] += boundary_strength[y][x] * decay * 0.5;
                    }
                }
            }
        }
    }

    // Normalize elevation to 0.0-1.0 range
    let mut min_elevation: f32 = 1.0;
    let mut max_elevation: f32 = 0.0;

    for y in 0..height {
        for x in 0..width {
            min_elevation = f32::min(min_elevation, elevation[y][x]);
            max_elevation = f32::max(max_elevation, elevation[y][x]);
        }
    }

    let range = max_elevation - min_elevation;

    for y in 0..height {
        for x in 0..width {
            elevation[y][x] = (elevation[y][x] - min_elevation) / range;
        }
    }

    elevation
}
