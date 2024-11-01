use rand::prelude::*;

pub fn seed_to_rng(value: u32) -> StdRng {
    let mut array: [u8; 32] = [0; 32];
    array[..4].copy_from_slice(&value.to_le_bytes());
    StdRng::from_seed(array)
}
