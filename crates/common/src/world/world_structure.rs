use crate::world::{Cell, CellSpecial, CellWall, Chunk};
use bevy::prelude::*;
use rand::{rngs::StdRng, Rng};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

const GRID_SIZE: usize = 4; // TODO: remove

#[derive(Clone, Deserialize, Serialize)]
pub struct WorldStructure {
    pub chunks: Vec<Chunk>,
}

#[derive(Clone, Debug, Default, Deserialize, Display, EnumIter, Eq, Hash, PartialEq, Serialize)]
pub enum WorldStructureName {
    #[default]
    None,
    EmptySpace1,
    FilledWithChairs1,
    House1,
    StairsAltar1,
    StaircaseTower2,
}

impl WorldStructureName {
    pub fn radius(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::EmptySpace1 | Self::FilledWithChairs1 | Self::House1 | Self::StairsAltar1 => 1,
            Self::StaircaseTower2 => 2,
        }
    }

    pub fn max_radius() -> u32 {
        Self::iter().map(|ws| ws.radius()).max().unwrap_or(0)
    }

    pub fn weight(&self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::EmptySpace1 => 3.0,
            Self::FilledWithChairs1 => 1.0,
            Self::House1 => 3.0,
            Self::StairsAltar1 => 4.0,
            Self::StaircaseTower2 => 4.0,
        }
    }

    pub fn total_weight() -> f32 {
        Self::iter().fold(0.0, |acc, curr| acc + curr.weight())
    }

    pub fn choose(rng: &mut StdRng) -> Self {
        let all: Vec<Self> = Self::iter().collect();
        if all.is_empty() {
            return Self::default();
        }

        let weights: Vec<f32> = all.iter().map(|ws| ws.weight()).collect();
        let rand_weight = rng.gen_range(0.0..Self::total_weight());

        let mut cumulative_weight = 0.0;
        for (index, &weight) in weights.iter().enumerate() {
            cumulative_weight += weight;
            if rand_weight < cumulative_weight {
                return all[index].clone();
            }
        }

        Self::default()
    }

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
            Self::House1 => {
                let cells = vec![
                    vec![Cell::new_floored(); GRID_SIZE],
                    vec![
                        Cell::new_floored(),
                        Cell {
                            wall_top: CellWall::SolidWithDoorGap,
                            wall_left: CellWall::Solid,
                            floor: CellWall::Solid,
                            ceiling: CellWall::Solid,
                            door_top: true,
                            ..default()
                        },
                        Cell {
                            wall_bottom: CellWall::SolidWithWindowGap,
                            wall_left: CellWall::Solid,
                            floor: CellWall::Solid,
                            ceiling: CellWall::Solid,
                            window_top: true,
                            ..default()
                        },
                        Cell::new_floored(),
                    ],
                    vec![
                        Cell::new_floored(),
                        Cell {
                            wall_top: CellWall::Solid,
                            wall_right: CellWall::Solid,
                            floor: CellWall::Solid,
                            ceiling: CellWall::Solid,
                            special: CellSpecial::Chair,
                            ..default()
                        },
                        Cell {
                            wall_bottom: CellWall::Solid,
                            wall_right: CellWall::Solid,
                            floor: CellWall::Solid,
                            ceiling: CellWall::Solid,
                            // TODO: Fix items removed from TreasureChests inside Houses
                            // are not being saved:
                            special: CellSpecial::TreasureChest,
                            ..default()
                        },
                        Cell::new_floored(),
                    ],
                    vec![Cell::new_floored(); GRID_SIZE],
                ];

                Chunk {
                    x,
                    y,
                    z,
                    cells,
                    world_structure: self.clone(),
                }
            }
            Self::StairsAltar1 => {
                let door_row = vec![
                    Cell {
                        floor: CellWall::Solid,
                        ceiling: CellWall::Solid,
                        ..default()
                    },
                    Cell {
                        wall_bottom: CellWall::SolidWithDoorGap,
                        wall_left: CellWall::Solid,
                        wall_right: CellWall::Solid,
                        floor: CellWall::Solid,
                        ceiling: CellWall::Solid,
                        door_bottom: true,
                        ..default()
                    },
                    Cell::new_floored(),
                    Cell::new_floored(),
                ];

                let stair_row = vec![
                    Cell {
                        wall_bottom: CellWall::Solid,
                        floor: CellWall::Solid,
                        ..default()
                    },
                    Cell {
                        floor: CellWall::Solid,
                        special: CellSpecial::Stairs,
                        ..default()
                    },
                    Cell::new_floored(),
                    Cell::new_floored(),
                ];

                Chunk {
                    x,
                    y,
                    z,
                    cells: vec![door_row.clone(), stair_row.clone(), stair_row, door_row],
                    world_structure: self.clone(),
                }
            }
            Self::StaircaseTower2 => Chunk {
                x,
                y,
                z,
                cells: vec![
                    vec![Cell::default(); GRID_SIZE],
                    vec![Cell::default(); GRID_SIZE],
                    vec![
                        Cell::default(),
                        Cell::default(),
                        Cell {
                            wall_top: CellWall::Solid,
                            wall_bottom: CellWall::Solid,
                            wall_left: CellWall::Solid,
                            wall_right: CellWall::Solid,
                            special: CellSpecial::Staircase,
                            ..default()
                        },
                        Cell::default(),
                    ],
                    vec![Cell::default(); GRID_SIZE],
                ],
                world_structure: self.clone(),
            },
        }
    }

    fn gen_chunks(&self, _x: i64, _y: i64, _z: i64) -> Vec<Chunk> {
        match self {
            Self::None => Vec::new(),
            Self::EmptySpace1 | Self::FilledWithChairs1 | Self::House1 | Self::StairsAltar1 => {
                vec![self.gen_origin_chunk(_x, _y, _z)]
            }
            Self::StaircaseTower2 => {
                let mut chunk_y_minus_1 = Chunk {
                    x: _x,
                    y: _y - 1,
                    z: _z,
                    cells: vec![
                        vec![
                            Cell {
                                floor: CellWall::Solid,
                                ..default()
                            };
                            GRID_SIZE
                        ];
                        GRID_SIZE
                    ],
                    world_structure: WorldStructureName::None,
                };
                // tower cell
                chunk_y_minus_1.cells[2][2] = Cell {
                    wall_top: CellWall::Solid,
                    wall_left: CellWall::Solid,
                    wall_right: CellWall::Solid,
                    floor: CellWall::Solid,
                    special: CellSpecial::Staircase,
                    ..Default::default()
                };

                let mut chunk_y_plus_1 = Chunk {
                    x: _x,
                    y: _y + 1,
                    z: _z,
                    cells: vec![vec![Cell::default(); GRID_SIZE]; GRID_SIZE],
                    world_structure: WorldStructureName::None,
                };
                // tower cell
                chunk_y_plus_1.cells[2][2] = Cell {
                    wall_top: CellWall::Solid,
                    wall_left: CellWall::Solid,
                    wall_right: CellWall::Solid,
                    ..default()
                };
                // loft cells
                for i in 0..=3 {
                    chunk_y_plus_1.cells[i][3] = Cell {
                        floor: CellWall::Solid,
                        ..default()
                    };
                }

                vec![
                    chunk_y_minus_1,
                    self.gen_origin_chunk(_x, _y, _z),
                    chunk_y_plus_1,
                ]
            }
        }
    }
}
