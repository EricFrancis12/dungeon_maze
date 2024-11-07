use crate::{
    maze::maze_from_rng,
    utils::rng::{rng_from_str, rng_from_xyz_seed},
    Cell, CellSpecial,
};

use rand::{rngs::StdRng, Rng};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const WALL_BREAK_PROB: f64 = 0.2;
const WORLD_STRUCTURE_GEN_PROB: f64 = 0.1;

#[derive(Clone)]
pub struct Chunk {
    pub cells: Vec<Vec<Cell>>,
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub world_structure: WorldStructure,
}

#[derive(Clone, Default, EnumIter)]
pub enum WorldStructure {
    #[default]
    None,
    EmptySpace1,
    EmptySpace2,
}

impl WorldStructure {
    fn radius(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::EmptySpace1 => 1,
            Self::EmptySpace2 => 2,
        }
    }

    fn max_radius() -> u32 {
        Self::iter().map(|ws| ws.radius()).max().unwrap_or(0)
    }

    fn choose(rng: &mut StdRng) -> Self {
        let all: Vec<Self> = Self::iter().collect();
        if all.is_empty() {
            return Self::default();
        }
        let i = rng.gen_range(0..all.len());
        all[i].clone()
    }

    fn gen_origin_chunk(&self, height: usize, width: usize, x: i64, y: i64, z: i64) -> Chunk {
        match self {
            Self::None | Self::EmptySpace1 | Self::EmptySpace2 => Chunk {
                cells: vec![vec![Cell::default(); width]; height],
                x,
                y,
                z,
                world_structure: self.clone(),
            },
        }
    }

    fn gen_chunks(&self, height: usize, width: usize, x: i64, y: i64, z: i64) -> Vec<Chunk> {
        match self {
            Self::None => Vec::new(),
            Self::EmptySpace1 | Self::EmptySpace2 => vec![
                Chunk {
                    cells: vec![vec![Cell::default(); width]; height],
                    x,
                    y,
                    z,
                    world_structure: self.clone(),
                };
                ((self.radius() * 2) - 1) as usize
            ],
        }
    }
}

pub fn chunk_from_xyz_seed(
    seed: u32,
    height: usize,
    width: usize,
    x: i64,
    y: i64,
    z: i64,
) -> Chunk {
    let mut rng = rng_from_xyz_seed(seed, x, y, z);

    if chunk_has_world_structure(seed, x, y, z) {
        return WorldStructure::choose(&mut rng).gen_origin_chunk(height, width, x, y, z);
    }

    let mut cells = maze_from_rng(&mut rng, height, width);

    let h = height / 2;
    let w = width / 2;

    // left and right walls
    cells[h][0].wall_left = false;
    cells[h][width - 1].wall_right = false;

    // top and bottom walls
    cells[0][w].wall_top = false;
    cells[height - 1][w].wall_bottom = false;

    // ceiling and floor (y axis)
    for h in 0..height {
        for w in 0..width {
            let mut y_minus_1_rng = rng_from_str(seed_str_from_neis(
                seed,
                (x, y - 1, z, w, h),
                (x, y, z, w, h),
            ));
            if y_minus_1_rng.gen_bool(WALL_BREAK_PROB) {
                cells[h][w].floor = false;
            }

            let mut y_plus_1_rng = rng_from_str(seed_str_from_neis(
                seed,
                (x, y, z, w, h),
                (x, y + 1, z, w, h),
            ));
            if y_plus_1_rng.gen_bool(WALL_BREAK_PROB) {
                cells[h][w].ceiling = false;
            }
        }
    }

    let mut floored_cells: Vec<(usize, usize)> = Vec::new();
    for h in 0..height {
        for w in 0..width {
            if cells[h][w].floor {
                floored_cells.push((w, h));
            }
        }
    }

    let rand_floored_cell = |r: &mut StdRng, fc: &mut Vec<(usize, usize)>| {
        let i = r.gen_range(0..fc.len());
        let (w, h) = fc[i];
        fc.splice(i..i + 1, []);
        (w, h)
    };

    for spec in CellSpecial::iter() {
        if floored_cells.is_empty() {
            break;
        }

        if rng.gen_bool(spec.spawn_prob()) {
            let (w, h) = rand_floored_cell(&mut rng, &mut floored_cells);
            cells[h][w].special = spec;
        }
    }

    let search_radius = WorldStructure::max_radius() as i64 - 1;
    if search_radius > 0 {
        // Reach out on all sides equal to max world structure radius
        // to see if any surrounding chunks have world structures.
        // The number of chunks to check in any one direction
        // is equal to the max world structure radius minus 1.

        let x_min = x - search_radius;
        let x_max = x + search_radius;
        let y_min = y - search_radius;
        let y_max = y + search_radius;
        let z_min = z - search_radius;
        let z_max = z + search_radius;

        for _x in x_min..=x_max {
            if _x == x {
                continue;
            }
            for _y in y_min..=y_max {
                if _y == y {
                    continue;
                }
                for _z in z_min..=z_max {
                    if _z == z {
                        continue;
                    }

                    if chunk_has_world_structure(seed, _x, _y, _z) {
                        let ws_chunk = chunk_from_xyz_seed(seed, height, width, _x, _y, _z);
                        let ws_chunks = ws_chunk
                            .world_structure
                            .gen_chunks(height, width, _x, _y, _z);

                        if let Some(ch) =
                            ws_chunks.iter().find(|c| c.x == x && c.y == y && c.z == z)
                        {
                            return ch.clone();
                        }
                    }
                }
            }
        }
    }

    Chunk {
        cells,
        x,
        y,
        z,
        world_structure: WorldStructure::None,
    }
}

fn chunk_has_world_structure(seed: u32, x: i64, y: i64, z: i64) -> bool {
    let mut rng = rng_from_xyz_seed(seed, x, y, z);
    rng.gen_bool(WORLD_STRUCTURE_GEN_PROB)
}

fn seed_str_from_neis(
    seed: u32,
    greater_nei: (i64, i64, i64, usize, usize),
    less_nei: (i64, i64, i64, usize, usize),
) -> String {
    let (g_chunk_x, g_chunk_y, g_chunk_z, g_x, g_z) = greater_nei;
    let (l_chunk_x, l_chunk_y, l_chunk_z, l_x, l_z) = less_nei;
    format!(
        "{}-{}_{}_{}_{}_{}-{}_{}_{}_{}_{}",
        seed, g_chunk_x, g_chunk_y, g_chunk_z, g_x, g_z, l_chunk_x, l_chunk_y, l_chunk_z, l_x, l_z,
    )
}
