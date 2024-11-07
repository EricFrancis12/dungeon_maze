use rand::prelude::*;

pub fn seed_from_str(seed_str: String) -> u32 {
    seed_str
        .trim()
        .split("")
        .map(|c| {
            c.as_bytes()
                .iter()
                .map(|i| i.to_owned() as u32)
                .fold(0, |acc, i| acc + i)
        })
        .fold(0, |acc: u32, j: u32| acc.wrapping_add(j))
}

pub fn seed_to_rng(value: u32) -> StdRng {
    let mut array: [u8; 32] = [0; 32];
    array[..4].copy_from_slice(&value.to_le_bytes());
    StdRng::from_seed(array)
}

pub fn rng_from_str(s: impl Into<String>) -> StdRng {
    let seed = seed_from_str(s.into());
    seed_to_rng(seed)
}

pub fn rng_from_xyz_seed(seed: u32, x: i64, y: i64, z: i64) -> StdRng {
    rng_from_str(fmt_seed_str(seed, x, y, z))
}

fn fmt_seed_str(seed: u32, x: i64, y: i64, z: i64) -> String {
    format!("{}-{}_{}_{}", seed, x, y, z)
}
