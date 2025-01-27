use crate::{gen_chunks, gen_origin_chunk, plugins::world::GRID_SIZE};
use bevy::prelude::default;
use dungeon_maze_common::world::{
    world_structure::WorldStructureName, Cell, CellSpecial, CellWall, Chunk,
};

// TODO: make it so that WorlsStructures in .json format can omit properties,
// and they will be assigned as default when parsed.

pub trait ChunkGenerator {
    fn gen_origin_chunk(&self, x: i64, y: i64, z: i64) -> Chunk;
    fn gen_chunks(&self, x: i64, y: i64, z: i64) -> Vec<Chunk>;
}

impl ChunkGenerator for WorldStructureName {
    fn gen_origin_chunk(&self, x: i64, y: i64, z: i64) -> Chunk {
        match self {
            Self::None | Self::EmptySpace1 => Chunk {
                x,
                y,
                z,
                cells: vec![vec![Cell::default(); GRID_SIZE]; GRID_SIZE],
                world_structure: self.clone(),
            },
            Self::FilledWithChairs1 => Chunk {
                x,
                y,
                z,
                cells: vec![
                    vec![
                        Cell {
                            floor: CellWall::Solid,
                            special: CellSpecial::Chair,
                            ..default()
                        };
                        GRID_SIZE
                    ];
                    GRID_SIZE
                ],
                world_structure: self.clone(),
            },
            Self::House1 | Self::StairsAltar1 | Self::StaircaseTower2 => {
                gen_origin_chunk(self, x, y, z)
            }
        }
    }

    fn gen_chunks(&self, x: i64, y: i64, z: i64) -> Vec<Chunk> {
        match self {
            Self::None
            | Self::EmptySpace1
            | Self::FilledWithChairs1
            // TODO: Fix items removed from TreasureChests inside House1
            // are not being saved:
            | Self::House1
            | Self::StairsAltar1 => {
                vec![self.gen_origin_chunk(x, y, z)]
            }
            Self::StaircaseTower2 => gen_chunks(self, x, y, z),
        }
    }
}
