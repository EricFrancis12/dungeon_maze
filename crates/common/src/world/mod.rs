pub mod data;
pub mod world_structure;

use crate::utils::{rng::rng_from_str, CyclicCounter};
use bevy::{
    ecs::system::EntityCommands,
    prelude::{
        default, Bundle, ChildBuilder, Commands, Component, GlobalTransform, States, Transform,
    },
    utils::HashMap,
};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};
use world_structure::WorldStructureName;

#[derive(Display)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Component, Debug, Default, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum CellWall {
    #[default]
    None,
    Solid,
    SolidWithDoorGap,
    SolidWithWindowGap,
}

#[derive(Clone, Debug, Default, Deserialize, EnumIter, PartialEq, Serialize)]
pub enum CellSpecial {
    #[default]
    None,
    Chair,
    TreasureChest,
    Staircase,
    Stairs, // Stairs currently run from -x to +x (going from base to peak)
}

impl CellSpecial {
    pub fn spawn_prob(&self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Chair => 0.38,
            Self::TreasureChest => 0.38,
            Self::Staircase => 0.18,
            Self::Stairs => 0.18,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Chunk {
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub cells: Vec<Vec<Cell>>,
    pub world_structure: WorldStructureName,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
pub struct ActiveChunk(pub i64, pub i64, pub i64);

impl ActiveChunk {
    pub fn to_tuple(&self) -> (i64, i64, i64) {
        (self.0, self.1, self.2)
    }
}

#[derive(Component)]
pub struct ChunkMarker(pub (i64, i64, i64));

#[derive(Clone, Component, Debug, Default, Eq, Hash, PartialEq)]
pub struct ChunkCellMarker {
    pub chunk_x: i64,
    pub chunk_y: i64,
    pub chunk_z: i64,
    pub x: usize,
    pub z: usize,
}

impl ChunkCellMarker {
    pub fn from_global_transform(gt: &GlobalTransform, chunk_size: f32, cell_size: f32) -> Self {
        let tl = gt.translation();

        let grid_size_minus_one = (chunk_size / cell_size) - 1.0;
        let half_chunk_size = chunk_size / 2.0;

        // Calculate the offset for centering at (0, 0, 0)
        let offset_x = tl.x + half_chunk_size;
        let offset_z = tl.z + half_chunk_size;

        // Calculate chunk coordinates
        let chunk_x = (offset_x / chunk_size).floor() as i64;
        let chunk_y = (tl.y / cell_size).floor() as i64;
        let chunk_z = (offset_z / chunk_size).floor() as i64;

        // Calculate local position within the chunk
        let x = (grid_size_minus_one
            - ((offset_x - (chunk_x as f32 * chunk_size)) / cell_size).floor())
            as usize;
        let z = (grid_size_minus_one
            - ((offset_z - (chunk_z as f32 * chunk_size)) / cell_size).floor())
            as usize;

        Self {
            chunk_x,
            chunk_y,
            chunk_z,
            x,
            z,
        }
    }

    pub fn to_rng(&self) -> StdRng {
        rng_from_str(format!(
            "{},{},{}_{},{}",
            self.chunk_x, self.chunk_y, self.chunk_z, self.x, self.z
        ))
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

    pub fn new_cycled(transforms: Vec<Vec<Transform>>) -> Self {
        let mut ct = Self::new(transforms);
        ct.counter.cycle();
        ct
    }

    pub fn cycle(&mut self) -> u32 {
        self.index = Some(0);
        self.counter.cycle()
    }

    pub fn tick(&mut self) -> Option<&Transform> {
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

pub trait EntitySpawner {
    fn spawn(&mut self, entity: impl Bundle) -> EntityCommands;
}

impl EntitySpawner for Commands<'_, '_> {
    fn spawn(&mut self, entity: impl Bundle) -> EntityCommands {
        self.spawn(entity)
    }
}

impl EntitySpawner for ChildBuilder<'_> {
    fn spawn(&mut self, entity: impl Bundle) -> EntityCommands {
        self.spawn(entity)
    }
}

#[derive(Component)]
pub struct OCItemContainer;
