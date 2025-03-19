use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Seedable};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Generate a random name based on culture parameters
pub fn generate_name(rng: &mut StdRng, culture_seed: u64, gender: &str) -> String {
    // This would be expanded with more sophisticated name generation
    // For now just a simple placeholder
    let syllables = [
        "ka", "ri", "ta", "lo", "mi", "su", "pa", "en", "dor", "mal", "gon", "bor",
    ];

    let len = rng.gen_range(2..=4);
    let mut name = String::new();

    for _ in 0..len {
        let idx = rng.gen_range(0..syllables.len());
        name.push_str(syllables[idx]);
    }

    // Capitalize first letter
    if !name.is_empty() {
        name = name
            .chars()
            .next()
            .unwrap()
            .to_uppercase()
            .collect::<String>()
            + &name[1..];
    }

    name
}

/// Calculate distance between two points considering world wrapping
pub fn world_distance(
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
    width: usize,
    height: usize,
) -> f32 {
    // Handle horizontal wrapping (for cylindrical or toroidal worlds)
    let dx1 = if x1 > x2 { x1 - x2 } else { x2 - x1 };
    let dx2 = width - dx1;
    let dx = dx1.min(dx2);

    // Handle vertical wrapping (only for toroidal worlds)
    let dy1 = if y1 > y2 { y1 - y2 } else { y2 - y1 };
    let dy2 = height - dy1;
    let dy = dy1.min(dy2);

    ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
}

/// Generate heightmap using multiple layers of noise
pub fn generate_heightmap(width: usize, height: usize, seed: u32) -> Vec<Vec<f32>> {
    let perlin = Perlin::new(seed);
    let mut heightmap = vec![vec![0.0; width]; height];

    // Parameters for fractal noise
    let octaves = 6;
    let persistence = 0.5;
    let lacunarity = 2.0;

    for y in 0..height {
        for x in 0..width {
            let mut amplitude = 1.0;
            let mut frequency = 1.0;
            let mut noise_height = 0.0;
            let mut max_value = 0.0;

            // Generate fractal noise
            for _ in 0..octaves {
                let sample_x = x as f64 * frequency / width as f64;
                let sample_y = y as f64 * frequency / height as f64;

                let noise_value = perlin.get([sample_x, sample_y]) as f32;
                noise_height += noise_value * amplitude;

                max_value += amplitude;
                amplitude *= persistence;
                frequency *= lacunarity;
            }

            // Normalize
            heightmap[y][x] = noise_height / max_value;
        }
    }

    heightmap
}

/// Calculate biome based on temperature and rainfall
pub fn determine_biome(elevation: f32, temperature: f32, rainfall: f32) -> crate::world::Biome {
    use crate::world::Biome;

    // Simple biome determination - would be more sophisticated in a real implementation
    if elevation < 0.2 {
        return Biome::Ocean;
    }

    if elevation > 0.8 {
        return Biome::Mountain;
    }

    // Temperature goes from cold (0.0) to hot (1.0)
    // Rainfall goes from dry (0.0) to wet (1.0)

    match (temperature, rainfall) {
        (t, r) if t > 0.7 && r < 0.2 => Biome::Desert,
        (t, r) if t > 0.7 && r < 0.5 => Biome::Savanna,
        (t, r) if t > 0.7 => Biome::TropicalRainforest,
        (t, r) if t > 0.4 && r < 0.3 => Biome::Grassland,
        (t, r) if t > 0.4 && r < 0.6 => Biome::Woodland,
        (t, r) if t > 0.4 => Biome::Forest,
        (t, r) if t > 0.2 => Biome::Taiga,
        _ => Biome::Tundra,
    }
}

/// Find a path between two points using A* algorithm
pub fn find_path(
    start_x: usize,
    start_y: usize,
    end_x: usize,
    end_y: usize,
    world_width: usize,
    world_height: usize,
    is_passable: impl Fn(usize, usize) -> bool,
) -> Option<Vec<(usize, usize)>> {
    // A* pathfinding implementation would go here
    // For brevity, this is a placeholder that would be implemented fully

    Some(vec![(start_x, start_y), (end_x, end_y)])
}
