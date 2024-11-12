use super::{Cell, CellSpecial, CellWall, Chunk, GRID_SIZE};

use bevy::prelude::*;
use rand::{rngs::StdRng, Rng};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone, Debug, Default, EnumIter)]
pub enum WorldStructure {
    #[default]
    None,
    EmptySpace1,
    FilledWithChairs1,
    House1,
    StaircaseTower2,
}

impl WorldStructure {
    pub fn radius(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::EmptySpace1 => 1,
            Self::FilledWithChairs1 => 1,
            Self::House1 => 1,
            Self::StaircaseTower2 => 2,
        }
    }

    pub fn max_radius() -> u32 {
        Self::iter().map(|ws| ws.radius()).max().unwrap_or(0)
    }

    pub fn choose(rng: &mut StdRng) -> Self {
        let all: Vec<Self> = Self::iter().collect();
        if all.is_empty() {
            return Self::default();
        }
        let i = rng.gen_range(0..all.len());
        all[i].clone()
    }

    pub fn gen_origin_chunk(&self, x: i64, y: i64, z: i64) -> Chunk {
        match self {
            Self::None | Self::EmptySpace1 => Chunk {
                x,
                y,
                z,
                cells: vec![vec![super::Cell::default(); GRID_SIZE]; GRID_SIZE],
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
                            wall_top: CellWall::SolidWithWindowGap,
                            wall_right: CellWall::Solid,
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
                            wall_bottom: CellWall::Solid,
                            wall_left: CellWall::Solid,
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

    pub fn gen_chunks(&self, _x: i64, _y: i64, _z: i64) -> Vec<Chunk> {
        match self {
            Self::None => Vec::new(),
            Self::EmptySpace1 | Self::FilledWithChairs1 | Self::House1 => {
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
                    world_structure: WorldStructure::None,
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
                    world_structure: WorldStructure::None,
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
                    chunk_y_plus_1.cells[3][i] = Cell {
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
