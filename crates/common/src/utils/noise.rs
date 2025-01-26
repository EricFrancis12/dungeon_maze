use noise::{NoiseFn, Perlin};

const PERLIN_SCALE: f64 = 0.08; // Decreasing this value increases average biome size generated

pub fn noise_from_xyz_seed(
    seed: u32,
    x: i64,
    y: i64,
    z: i64,
    chunk_size: f32,
    cell_size: f32,
) -> f64 {
    let perlin = Perlin::new(seed);
    let noise_xy = perlin.get([x as f64 * PERLIN_SCALE, y as f64 * PERLIN_SCALE]);
    let noise_yz = perlin.get([y as f64 * PERLIN_SCALE, z as f64 * PERLIN_SCALE])
        / (chunk_size / cell_size) as f64;
    let noise_zx = perlin.get([z as f64 * PERLIN_SCALE, x as f64 * PERLIN_SCALE]);
    let noise_xyz = (noise_xy + noise_yz + noise_zx) / 3.0;
    noise_xyz
}
