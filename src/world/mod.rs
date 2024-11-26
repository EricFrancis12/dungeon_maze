pub mod bundle;
pub mod data;
pub mod systems;
pub mod world_structure;

use crate::utils::{
    maze::maze_from_rng,
    rng::{rng_from_str, rng_from_xyz_seed},
    CyclicCounter,
};
use data::update_world_data_treasure_chests;
use systems::*;

use bevy::prelude::*;
use rand::{rngs::StdRng, Rng};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use world_structure::WorldStructure;

pub const CELL_SIZE: f32 = 4.0;
pub const CHUNK_SIZE: f32 = 16.0;
pub const GRID_SIZE: usize = (CHUNK_SIZE / CELL_SIZE) as usize;

const WALL_BREAK_PROB: f64 = 0.2;
const WORLD_STRUCTURE_GEN_PROB: f64 = 0.18;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ActiveChunk>()
            .init_resource::<AssetLib>()
            .add_event::<ActiveChunkChangeRequest>()
            .add_systems(Startup, (preload_assets, spawn_initial_chunks))
            .add_systems(
                Update,
                (
                    manage_active_chunk,
                    handle_active_chunk_change,
                    advance_cyclic_transforms,
                    handle_cyclic_transform_interactions.after(advance_cyclic_transforms),
                    activate_items_inside_containers.after(advance_cyclic_transforms),
                    update_world_data_treasure_chests,
                ),
            );
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Resource)]
pub struct AssetLib {
    meshes: Vec<Handle<Mesh>>,
}

#[derive(Display)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Component, Debug, Default)]
pub struct Cell {
    pub wall_top: CellWall,
    pub wall_bottom: CellWall,
    pub wall_left: CellWall,
    pub wall_right: CellWall,
    pub floor: CellWall,
    pub ceiling: CellWall,
    pub door_top: bool,
    pub door_bottom: bool,
    pub door_left: bool,
    pub door_right: bool,
    pub window_top: bool,
    pub window_bottom: bool,
    pub window_left: bool,
    pub window_right: bool,
    pub special: CellSpecial,
}

impl Cell {
    pub fn new_floored() -> Self {
        Self {
            floor: CellWall::Solid,
            ..default()
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum CellWall {
    #[default]
    None,
    Solid,
    SolidWithDoorGap,
    SolidWithWindowGap,
}

#[derive(Clone, Debug, Default, EnumIter, PartialEq)]
pub enum CellSpecial {
    #[default]
    None,
    Chair,
    TreasureChest,
    Staircase,
}

impl CellSpecial {
    fn spawn_prob(&self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Chair => 0.48,
            Self::TreasureChest => 0.48,
            Self::Staircase => 0.18,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub cells: Vec<Vec<Cell>>,
    pub world_structure: WorldStructure,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
pub struct ActiveChunk(i64, i64, i64);

impl ActiveChunk {
    fn to_tuple(&self) -> (i64, i64, i64) {
        (self.0, self.1, self.2)
    }
}

#[derive(Event)]
pub struct ActiveChunkChangeRequest {
    pub value: ActiveChunk,
}

#[derive(Component)]
pub struct ChunkMarker((i64, i64, i64));

#[derive(Clone, Component, Debug, Default, Eq, Hash, PartialEq)]
pub struct ChunkCellMarker {
    pub chunk_x: i64,
    pub chunk_y: i64,
    pub chunk_z: i64,
    pub x: usize,
    pub z: usize,
}

impl ChunkCellMarker {
    pub fn from_global_transform(gt: &GlobalTransform) -> Self {
        let tl = gt.translation();

        let grid_size_minus_one = (CHUNK_SIZE / CELL_SIZE) - 1.0;
        let half_chunk_size = CHUNK_SIZE / 2.0;

        // Calculate the offset for centering at (0, 0, 0)
        let offset_x = tl.x + half_chunk_size;
        let offset_z = tl.z + half_chunk_size;

        // Calculate chunk coordinates
        let chunk_x = (offset_x / CHUNK_SIZE).floor() as i64;
        let chunk_y = (tl.y / CELL_SIZE).floor() as i64;
        let chunk_z = (offset_z / CHUNK_SIZE).floor() as i64;

        // Calculate local position within the chunk
        let x = (grid_size_minus_one
            - ((offset_x - (chunk_x as f32 * CHUNK_SIZE)) / CELL_SIZE).floor())
            as usize;
        let z = (grid_size_minus_one
            - ((offset_z - (chunk_z as f32 * CHUNK_SIZE)) / CELL_SIZE).floor())
            as usize;

        Self {
            chunk_x,
            chunk_y,
            chunk_z,
            x,
            z,
        }
    }

    pub fn chunk_xyz(&self) -> (i64, i64, i64) {
        (self.chunk_x, self.chunk_y, self.chunk_z)
    }

    pub fn cell_xz(&self) -> (usize, usize) {
        (self.x, self.z)
    }
}

#[derive(Component)]
pub struct CyclicTransform {
    counter: CyclicCounter,
    transforms: HashMap<u32, Vec<Transform>>,
    index: Option<usize>,
}

impl CyclicTransform {
    fn new(transforms: Vec<Vec<Transform>>) -> Self {
        let mut hm: HashMap<u32, Vec<Transform>> = HashMap::new();
        for (i, v) in transforms.iter().enumerate() {
            hm.insert(i as u32, v.clone());
        }

        Self {
            counter: CyclicCounter::new(0, (transforms.len() - 1) as u32),
            transforms: hm,
            index: None,
        }
    }

    fn new_cycled(transforms: Vec<Vec<Transform>>) -> Self {
        let mut ct = Self::new(transforms);
        ct.counter.cycle();
        ct
    }

    fn cycle(&mut self) -> u32 {
        self.index = Some(0);
        self.counter.cycle()
    }

    fn tick(&mut self) -> Option<&Transform> {
        if let Some(i) = self.index {
            let transforms = self.transforms.get(&self.counter.value()).unwrap();
            let next_index = i + 1;
            let max_index = transforms.len() - 1;

            self.index = if next_index > max_index {
                None
            } else {
                Some(next_index)
            };

            return transforms.get(i);
        }
        None
    }
}

pub fn chunk_from_xyz_seed(seed: u32, x: i64, y: i64, z: i64) -> Chunk {
    let mut rng = rng_from_xyz_seed(seed, x, y, z);

    if chunk_has_world_structure(seed, x, y, z) {
        return WorldStructure::choose(&mut rng).gen_origin_chunk(x, y, z);
    }

    let mut cells = maze_from_rng(&mut rng, GRID_SIZE, GRID_SIZE);

    let h = GRID_SIZE / 2;
    let w = GRID_SIZE / 2;

    // left and right walls
    cells[h][0].wall_left = CellWall::None;
    cells[h][GRID_SIZE - 1].wall_right = CellWall::None;

    // top and bottom walls
    cells[0][w].wall_top = CellWall::None;
    cells[GRID_SIZE - 1][w].wall_bottom = CellWall::None;

    // ceiling and floor (y axis)
    for h in 0..GRID_SIZE {
        for w in 0..GRID_SIZE {
            let mut y_minus_1_rng = rng_from_str(seed_str_from_neis(
                seed,
                (x, y - 1, z, w, h),
                (x, y, z, w, h),
            ));
            if y_minus_1_rng.gen_bool(WALL_BREAK_PROB) {
                cells[h][w].floor = CellWall::None;
            }

            let mut y_plus_1_rng = rng_from_str(seed_str_from_neis(
                seed,
                (x, y, z, w, h),
                (x, y + 1, z, w, h),
            ));
            if y_plus_1_rng.gen_bool(WALL_BREAK_PROB) {
                cells[h][w].ceiling = CellWall::None;
            }
        }
    }

    let mut floored_cells: Vec<(usize, usize)> = Vec::new();
    for h in 0..GRID_SIZE {
        for w in 0..GRID_SIZE {
            if cells[h][w].floor == CellWall::Solid {
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
            for _y in y_min..=y_max {
                for _z in z_min..=z_max {
                    if _x == x && _y == y && _z == z {
                        continue;
                    }

                    if chunk_has_world_structure(seed, _x, _y, _z) {
                        let ws_chunk = chunk_from_xyz_seed(seed, _x, _y, _z);
                        let ws_chunks = ws_chunk.world_structure.gen_chunks(_x, _y, _z);

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
        x,
        y,
        z,
        cells,
        world_structure: WorldStructure::None,
    }
}

pub fn make_nei_chunks_xyz(
    chunk: (i64, i64, i64),
    x_rend_dist: u32,
    y_rend_dist: u32,
    z_rend_dist: u32,
) -> Vec<(i64, i64, i64)> {
    if x_rend_dist == 0 || y_rend_dist == 0 || z_rend_dist == 0 {
        return Vec::new();
    }

    let (x, y, z) = chunk;

    let x_r = x_rend_dist as i64 - 1;
    let y_r = y_rend_dist as i64 - 1;
    let z_r = z_rend_dist as i64 - 1;

    (x - x_r..=x + x_r)
        .flat_map(|i| {
            (y - y_r..=y + y_r).flat_map(move |j| (z - z_r..=z + z_r).map(move |k| (i, j, k)))
        })
        .collect()
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
