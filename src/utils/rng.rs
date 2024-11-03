use rand::prelude::*;

pub fn seed_to_rng(value: u32) -> StdRng {
    let mut array: [u8; 32] = [0; 32];
    array[..4].copy_from_slice(&value.to_le_bytes());
    StdRng::from_seed(array)
}

pub fn seed_from_seed_str(seed_str: String) -> u32 {
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
