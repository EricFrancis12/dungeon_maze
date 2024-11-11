use crate::{
    animation::CyclicAnimation,
    interaction::Interactable,
    maze::maze_from_rng,
    player::Player,
    settings::GameSettings,
    utils::{
        noise::noise_from_xyz_seed,
        rng::{rng_from_str, rng_from_xyz_seed},
    },
    SEED,
};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::{rngs::StdRng, Rng};
use std::{collections::HashSet, f32::consts::PI};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub const CELL_SIZE: f32 = 4.0;
pub const CHUNK_SIZE: f32 = 16.0;
pub const GRID_SIZE: usize = (CHUNK_SIZE / CELL_SIZE) as usize;
pub const WALL_THICKNESS: f32 = 0.1;

const WALL_BREAK_PROB: f64 = 0.2;
const WORLD_STRUCTURE_GEN_PROB: f64 = 0.08;

const CHAIR_COLLIDER_HX: f32 = 0.2;
const CHAIR_COLLIDER_HY: f32 = 0.25;
const CHAIR_COLLIDER_HZ: f32 = 0.2;

const TREASURE_CHEST_COLLIDER_HX: f32 = 0.5;
const TREASURE_CHEST_COLLIDER_HY: f32 = 0.3;
const TREASURE_CHEST_COLLIDER_HZ: f32 = 0.3;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ActiveChunk>()
            .init_resource::<AssetLib>()
            .add_event::<ActiveChunkChangeRequest>()
            .add_systems(Startup, (load_assets, spawn_initial_chunks))
            .add_systems(Update, (manage_active_chunk, handle_active_chunk_change));
    }
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
    pub special: CellSpecial,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum CellWall {
    #[default]
    None,
    Solid,
    SolidWithDoorGap,
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

#[derive(Clone, Debug, Default, EnumIter)]
pub enum WorldStructure {
    #[default]
    None,
    EmptySpace1,
    FilledWithChairs1,
    StaircaseTower2,
}

impl WorldStructure {
    fn radius(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::EmptySpace1 => 1,
            Self::FilledWithChairs1 => 1,
            Self::StaircaseTower2 => 2,
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
            Self::EmptySpace1 | Self::FilledWithChairs1 => {
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
struct ChunkMarker((i64, i64, i64));

#[derive(Clone, Component, Debug, PartialEq)]
struct ChunkCellMarker {
    chunk_x: i64,
    chunk_y: i64,
    chunk_z: i64,
    x: usize,
    z: usize,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Resource)]
struct AssetLib {
    meshes: Vec<Handle<Mesh>>,
    models: Vec<Handle<Gltf>>,
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let asset_lib = AssetLib {
        meshes: vec![asset_server.load("meshes/wall_with_door_gap.glb#Mesh0/Primitive0")],
        models: vec![
            // TODO: ...
        ],
    };
    commands.insert_resource(asset_lib);
}

fn spawn_initial_chunks(
    mut commands: Commands,
    active_chunk: Res<State<ActiveChunk>>,
    game_settings: Res<State<GameSettings>>,
    asset_server: Res<AssetServer>,
    asset_lib: Res<AssetLib>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let render_dist = game_settings.chunk_render_dist;
    let chunks = make_nei_chunks_xyz(
        (active_chunk.0, active_chunk.1, active_chunk.2),
        render_dist.0,
        render_dist.1,
        render_dist.2,
    );
    for xyz in chunks {
        spawn_new_chunk_bundle(
            xyz,
            &mut commands,
            &asset_server,
            &asset_lib,
            &mut meshes,
            &mut materials,
        );
    }
}

fn manage_active_chunk(
    mut event_writer: EventWriter<ActiveChunkChangeRequest>,
    player_query: Query<&GlobalTransform, With<Player>>,
    active_chunk: Res<State<ActiveChunk>>,
) {
    let player_gl_transform = player_query.get_single().expect("Error retrieving player");
    let player_gl_translation = player_gl_transform.translation();

    let mut chunk = active_chunk.clone();
    let half_chunk_size = CHUNK_SIZE / 2.0;
    let half_cell_size = CELL_SIZE / 2.0;

    // x
    let x_chunk_size = active_chunk.0 as f32 * CHUNK_SIZE;
    let x_min_crossed = player_gl_translation.x < x_chunk_size - half_chunk_size;
    let x_max_crossed = player_gl_translation.x > x_chunk_size + half_chunk_size;

    if x_min_crossed {
        chunk.0 -= 1;
    } else if x_max_crossed {
        chunk.0 += 1;
    }

    // y
    let y_chunk_size = active_chunk.1 as f32 * CELL_SIZE;
    let y_min_crossed = player_gl_translation.y < y_chunk_size - half_cell_size;
    let y_max_crossed = player_gl_translation.y > y_chunk_size + half_cell_size;

    if y_min_crossed {
        chunk.1 -= 1;
    } else if y_max_crossed {
        chunk.1 += 1;
    }

    // z
    let z_chunk_size = active_chunk.2 as f32 * CHUNK_SIZE;
    let z_min_crossed = player_gl_translation.z < z_chunk_size - half_chunk_size;
    let z_max_crossed = player_gl_translation.z > z_chunk_size + half_chunk_size;

    if z_min_crossed {
        chunk.2 -= 1;
    } else if z_max_crossed {
        chunk.2 += 1;
    }

    if x_min_crossed
        || x_max_crossed
        || y_min_crossed
        || y_max_crossed
        || z_min_crossed
        || z_max_crossed
    {
        event_writer.send(ActiveChunkChangeRequest { value: chunk });
    }
}

fn handle_active_chunk_change(
    mut commands: Commands,
    mut event_reader: EventReader<ActiveChunkChangeRequest>,
    chunks_query: Query<(Entity, &ChunkMarker)>,
    game_settings: Res<State<GameSettings>>,
    mut next_active_chunk: ResMut<NextState<ActiveChunk>>,
    asset_server: Res<AssetServer>,
    asset_lib: Res<AssetLib>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in event_reader.read() {
        let chunk_xyz = event.value.to_tuple();
        next_active_chunk.set(event.value);

        let rend_dist = game_settings.chunk_render_dist;
        let new_chunks = make_nei_chunks_xyz(chunk_xyz, rend_dist.0, rend_dist.1, rend_dist.2);

        let mut existing_chunks: HashSet<(i64, i64, i64)> = HashSet::new();

        // Despawn chunks that are not in the new chunks
        for (chunk_entity, chunk_marker) in chunks_query.iter() {
            if !new_chunks.contains(&chunk_marker.0) {
                commands.entity(chunk_entity).despawn_recursive();
            }
            existing_chunks.insert(chunk_marker.0);
        }

        // Spawn new chunks that are not currently existing
        for (x, y, z) in new_chunks {
            if !existing_chunks.contains(&(x, y, z)) {
                spawn_new_chunk_bundle(
                    (x, y, z),
                    &mut commands,
                    &asset_server,
                    &asset_lib,
                    &mut meshes,
                    &mut materials,
                );
            }
        }
    }
}

fn spawn_new_chunk_bundle(
    (chunk_x, chunk_y, chunk_z): (i64, i64, i64),
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    asset_lib: &Res<AssetLib>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let chunk_bundle = (
        SpatialBundle {
            transform: Transform::from_xyz(
                chunk_x as f32 * CHUNK_SIZE,
                chunk_y as f32 * CELL_SIZE,
                chunk_z as f32 * CHUNK_SIZE,
            ),
            ..default()
        },
        ChunkMarker((chunk_x, chunk_y, chunk_z)),
        Name::new(format!("Chunk_({},{},{})", chunk_x, chunk_y, chunk_z)),
    );

    commands.spawn(chunk_bundle).with_children(|parent| {
        let chunk = chunk_from_xyz_seed(SEED, chunk_x, chunk_y, chunk_z);

        for (x, row) in chunk.cells.iter().enumerate() {
            for (z, cell) in row.iter().enumerate() {
                let ccm = ChunkCellMarker {
                    chunk_x,
                    chunk_y,
                    chunk_z,
                    x,
                    z,
                };

                let cell_bundle = (
                    SpatialBundle {
                        transform: Transform::from_xyz(calc_floor_pos(x), 0.0, calc_floor_pos(z)),
                        ..default()
                    },
                    cell.clone(),
                    ccm.clone(),
                    Name::new(format!("Cell_({},{})", x, z)),
                );

                parent.spawn(cell_bundle).with_children(|grandparent| {
                    // Special
                    match cell.special {
                        CellSpecial::None => (),
                        CellSpecial::Chair => {
                            grandparent
                                .spawn((
                                    SpatialBundle {
                                        transform: Transform::from_xyz(
                                            0.0,
                                            CHAIR_COLLIDER_HY * 2.0,
                                            0.0,
                                        ),
                                        ..default()
                                    },
                                    RigidBody::Dynamic,
                                    Collider::cuboid(
                                        CHAIR_COLLIDER_HX,
                                        CHAIR_COLLIDER_HY,
                                        CHAIR_COLLIDER_HZ,
                                    ),
                                    Name::new("Chair"),
                                ))
                                .with_children(|ggp| {
                                    ggp.spawn((
                                        SceneBundle {
                                            scene: asset_server.load(
                                                GltfAssetLabel::Scene(0)
                                                    .from_asset("models/Chair.glb"),
                                            ),
                                            transform: Transform::from_xyz(
                                                0.0,
                                                -CHAIR_COLLIDER_HY,
                                                0.0,
                                            ),
                                            ..default()
                                        },
                                        Name::new("Chair Model"),
                                    ));
                                });
                        }
                        CellSpecial::TreasureChest => {
                            grandparent
                                .spawn((
                                    SpatialBundle {
                                        transform: Transform::from_xyz(
                                            0.0,
                                            TREASURE_CHEST_COLLIDER_HY,
                                            0.0,
                                        ),
                                        ..default()
                                    },
                                    Collider::cuboid(
                                        TREASURE_CHEST_COLLIDER_HX,
                                        TREASURE_CHEST_COLLIDER_HY,
                                        TREASURE_CHEST_COLLIDER_HZ,
                                    ),
                                    Interactable { range: 2.0 },
                                    CyclicAnimation::new(2, 3),
                                    Name::new("Treasure Chest"),
                                ))
                                .with_children(|ggp| {
                                    ggp.spawn((
                                        SceneBundle {
                                            scene: asset_server.load(
                                                GltfAssetLabel::Scene(0)
                                                    .from_asset("models/Treasure_Chest.glb"),
                                            ),
                                            transform: Transform::from_xyz(
                                                0.0,
                                                -TREASURE_CHEST_COLLIDER_HY,
                                                0.0,
                                            ),
                                            ..default()
                                        },
                                        Name::new("Treasure Chest Model"),
                                    ));
                                });
                        }
                        CellSpecial::Staircase => {
                            let mut shapes: Vec<(Vec3, Quat, Collider)> = Vec::new();

                            // lower steps
                            for i in 0..7 {
                                shapes.push((
                                    Vec3 {
                                        x: -0.9 + (i as f32 * 0.3),
                                        y: 0.1 + (i as f32 * 0.3),
                                        z: -1.18,
                                    },
                                    Quat::default(),
                                    Collider::cuboid(0.2, 0.1, 0.82),
                                ));
                            }

                            // upper steps
                            for j in 0..5 {
                                shapes.push((
                                    Vec3 {
                                        x: 0.3 - (j as f32 * 0.3),
                                        y: 2.5 + (j as f32 * 0.3),
                                        z: 1.18,
                                    },
                                    Quat::default(),
                                    Collider::cuboid(0.2, 0.1, 0.82),
                                ));
                            }

                            // lower flat section
                            shapes.push((
                                Vec3 {
                                    x: 1.5,
                                    y: 2.2,
                                    z: 0.0,
                                },
                                Quat::default(),
                                Collider::cuboid(0.5, 0.01, 2.0),
                            ));

                            // upper flat section
                            shapes.push((
                                Vec3 {
                                    x: -1.5,
                                    y: 4.0,
                                    z: 0.0,
                                },
                                Quat::default(),
                                Collider::cuboid(0.5, 0.01, 2.0),
                            ));

                            grandparent
                                .spawn((
                                    SpatialBundle {
                                        transform: Transform::from_xyz(0.0, 0.0, 0.0),
                                        ..default()
                                    },
                                    RigidBody::Fixed,
                                    Collider::compound(shapes),
                                    Name::new("Staircase"),
                                ))
                                .with_children(|ggp| {
                                    ggp.spawn((
                                        SceneBundle {
                                            scene: asset_server.load(
                                                GltfAssetLabel::Scene(0)
                                                    .from_asset("models/Staircase.glb"),
                                            ),
                                            transform: Transform {
                                                translation: Vec3 {
                                                    x: 0.0,
                                                    y: 2.0,
                                                    z: -2.0,
                                                },
                                                scale: Vec3 {
                                                    x: 2.0,
                                                    y: 2.0,
                                                    z: 2.0,
                                                },
                                                ..default()
                                            },
                                            ..default()
                                        },
                                        Name::new("Staircase Model"),
                                    ));
                                });
                        }
                    }

                    let mesh = meshes.add(
                        Cuboid::from_size(Vec3 {
                            x: CELL_SIZE,
                            y: WALL_THICKNESS,
                            z: CELL_SIZE,
                        })
                        .mesh(),
                    );

                    // Floor
                    if cell.floor == CellWall::Solid {
                        grandparent.spawn((
                            PbrBundle {
                                mesh: mesh.clone(),
                                material: materials.add(Color::linear_rgba(0.55, 0.0, 0.0, 1.0)),
                                transform: Transform {
                                    translation: Vec3 {
                                        y: WALL_THICKNESS / 2.0,
                                        ..default()
                                    },
                                    ..default()
                                },
                                ..default()
                            },
                            Collider::cuboid(
                                CELL_SIZE / 2.0,
                                WALL_THICKNESS / 2.0,
                                CELL_SIZE / 2.0,
                            ),
                            Name::new("Floor"),
                        ));
                    }

                    // Ceiling
                    if cell.ceiling == CellWall::Solid {
                        grandparent.spawn((
                            PbrBundle {
                                mesh: mesh.clone(),
                                material: materials.add(Color::linear_rgba(0.0, 0.2, 0.4, 1.0)),
                                transform: Transform {
                                    translation: Vec3 {
                                        y: CELL_SIZE - WALL_THICKNESS / 2.0,
                                        ..default()
                                    },
                                    ..default()
                                }
                                .with_rotation(Quat::from_rotation_x(PI)),
                                ..default()
                            },
                            Collider::cuboid(
                                CELL_SIZE / 2.0,
                                WALL_THICKNESS / 2.0,
                                CELL_SIZE / 2.0,
                            ),
                            Name::new("Ceiling"),
                        ));
                    }

                    let noise_xyz = noise_from_xyz_seed(SEED, chunk_x, chunk_y, chunk_z);

                    let path = if noise_xyz < -0.2 {
                        "images/wall-1.png"
                    } else if noise_xyz < 0.0 {
                        "images/wall-2.png"
                    } else if noise_xyz < 0.2 {
                        "images/wall-3.png"
                    } else {
                        "images/wall-4.png"
                    };

                    let wall_texture_handle = asset_server.load(path);
                    let material = materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        base_color_texture: Some(wall_texture_handle),
                        ..Default::default()
                    });

                    // Top wall
                    if cell.wall_top == CellWall::Solid {
                        grandparent.spawn((
                            PbrBundle {
                                mesh: mesh.clone(),
                                material: material.clone(),
                                transform: Transform::from_xyz(
                                    CELL_SIZE / 2.0 - WALL_THICKNESS / 2.0,
                                    CELL_SIZE / 2.0,
                                    0.0,
                                )
                                .with_rotation(Quat::from_rotation_z(PI / 2.0)),
                                ..default()
                            },
                            Collider::cuboid(
                                CELL_SIZE / 2.0,
                                WALL_THICKNESS / 2.0,
                                CELL_SIZE / 2.0,
                            ),
                            Name::new("Top Wall"),
                        ));
                    }

                    // Top wall with door
                    if cell.wall_top == CellWall::SolidWithDoorGap {
                        let solid_with_door_mesh_handle = &asset_lib.meshes[0];
                        let solid_with_door_mesh = meshes.get(solid_with_door_mesh_handle).unwrap();

                        grandparent.spawn((
                            PbrBundle {
                                mesh: solid_with_door_mesh_handle.clone(),
                                material: material.clone(),
                                transform: Transform::from_xyz(
                                    CELL_SIZE / 2.0,
                                    CELL_SIZE / 2.0,
                                    0.0,
                                )
                                .with_scale(Vec3 {
                                    x: 2.0,
                                    y: WALL_THICKNESS * 2.0,
                                    z: 2.0,
                                })
                                .with_rotation(
                                    Quat::from_rotation_x(PI / 2.0)
                                        * Quat::from_rotation_z(PI / 2.0),
                                ),
                                ..default()
                            },
                            Collider::from_bevy_mesh(
                                solid_with_door_mesh,
                                &ComputedColliderShape::TriMesh,
                            )
                            .unwrap(),
                            Name::new("Top Wall With Door Gap"),
                        ));
                    }

                    // Bottom wall
                    if cell.wall_bottom == CellWall::Solid {
                        grandparent.spawn((
                            PbrBundle {
                                mesh: mesh.clone(),
                                material: material.clone(),
                                transform: Transform::from_xyz(
                                    -CELL_SIZE / 2.0 + WALL_THICKNESS / 2.0,
                                    CELL_SIZE / 2.0,
                                    0.0,
                                )
                                .with_rotation(Quat::from_rotation_z(PI * 3.0 / 2.0)),
                                ..default()
                            },
                            Collider::cuboid(
                                CELL_SIZE / 2.0,
                                WALL_THICKNESS / 2.0,
                                CELL_SIZE / 2.0,
                            ),
                            Name::new("Bottom Wall"),
                        ));
                    }

                    // Bottom wall with door
                    if cell.wall_bottom == CellWall::SolidWithDoorGap {
                        let solid_with_door_mesh_handle = &asset_lib.meshes[0];
                        let solid_with_door_mesh = meshes.get(solid_with_door_mesh_handle).unwrap();

                        grandparent.spawn((
                            PbrBundle {
                                mesh: solid_with_door_mesh_handle.clone(),
                                material: material.clone(),
                                transform: Transform::from_xyz(
                                    -CELL_SIZE / 2.0 + WALL_THICKNESS,
                                    CELL_SIZE / 2.0,
                                    0.0,
                                )
                                .with_scale(Vec3 {
                                    x: 2.0,
                                    y: WALL_THICKNESS * 2.0,
                                    z: 2.0,
                                })
                                .with_rotation(
                                    Quat::from_rotation_x(PI / 2.0)
                                        * Quat::from_rotation_z(PI / 2.0),
                                ),
                                ..default()
                            },
                            Collider::from_bevy_mesh(
                                solid_with_door_mesh,
                                &ComputedColliderShape::TriMesh,
                            )
                            .unwrap(),
                            Name::new("Bottom Wall With Door Gap"),
                        ));
                    }

                    // Left wall
                    if cell.wall_left == CellWall::Solid {
                        grandparent.spawn((
                            PbrBundle {
                                mesh: mesh.clone(),
                                material: material.clone(),
                                transform: Transform::from_xyz(
                                    0.0,
                                    CELL_SIZE / 2.0,
                                    CELL_SIZE / 2.0 - WALL_THICKNESS / 2.0,
                                )
                                .with_rotation(Quat::from_rotation_x(PI * 3.0 / 2.0)),
                                ..default()
                            },
                            Collider::cuboid(
                                CELL_SIZE / 2.0,
                                WALL_THICKNESS / 2.0,
                                CELL_SIZE / 2.0,
                            ),
                            Name::new("Left Wall"),
                        ));
                    }

                    // Left wall with door
                    if cell.wall_left == CellWall::SolidWithDoorGap {
                        let solid_with_door_mesh_handle = &asset_lib.meshes[0];
                        let solid_with_door_mesh = meshes.get(solid_with_door_mesh_handle).unwrap();

                        grandparent.spawn((
                            PbrBundle {
                                mesh: solid_with_door_mesh_handle.clone(),
                                material: material.clone(),
                                transform: Transform::from_xyz(
                                    0.0,
                                    CELL_SIZE / 2.0,
                                    CELL_SIZE / 2.0 - WALL_THICKNESS,
                                )
                                .with_scale(Vec3 {
                                    x: 2.0,
                                    y: WALL_THICKNESS * 2.0,
                                    z: 2.0,
                                })
                                .with_rotation(Quat::from_rotation_x(PI / 2.0)),
                                ..default()
                            },
                            Collider::from_bevy_mesh(
                                solid_with_door_mesh,
                                &ComputedColliderShape::TriMesh,
                            )
                            .unwrap(),
                            Name::new("Left Wall With Door Gap"),
                        ));
                    }

                    // Right wall
                    if cell.wall_right == CellWall::Solid {
                        grandparent.spawn((
                            PbrBundle {
                                mesh: mesh.clone(),
                                material: material.clone(),
                                transform: Transform::from_xyz(
                                    0.0,
                                    CELL_SIZE / 2.0,
                                    -CELL_SIZE / 2.0 + WALL_THICKNESS / 2.0,
                                )
                                .with_rotation(Quat::from_rotation_x(PI / 2.0)),
                                ..default()
                            },
                            Collider::cuboid(
                                CELL_SIZE / 2.0,
                                WALL_THICKNESS / 2.0,
                                CELL_SIZE / 2.0,
                            ),
                            Name::new("Right Wall"),
                        ));
                    }

                    // Right wall with door
                    if cell.wall_right == CellWall::SolidWithDoorGap {
                        let solid_with_door_mesh_handle = &asset_lib.meshes[0];
                        let solid_with_door_mesh = meshes.get(solid_with_door_mesh_handle).unwrap();

                        grandparent.spawn((
                            PbrBundle {
                                mesh: solid_with_door_mesh_handle.clone(),
                                material: material.clone(),
                                transform: Transform::from_xyz(
                                    0.0,
                                    CELL_SIZE / 2.0,
                                    -CELL_SIZE / 2.0,
                                )
                                .with_scale(Vec3 {
                                    x: 2.0,
                                    y: WALL_THICKNESS * 2.0,
                                    z: 2.0,
                                })
                                .with_rotation(Quat::from_rotation_x(PI / 2.0)),
                                ..default()
                            },
                            Collider::from_bevy_mesh(
                                solid_with_door_mesh,
                                &ComputedColliderShape::TriMesh,
                            )
                            .unwrap(),
                            Name::new("Right Wall With Door Gap"),
                        ));
                    }
                });
            }
        }
    });
}

fn calc_floor_pos(index: usize) -> f32 {
    let mut positions = vec![CELL_SIZE / 2.0, -CELL_SIZE / 2.0];
    while positions.len() < GRID_SIZE {
        positions.insert(0, positions[0] + CELL_SIZE);
        positions.push(positions.last().unwrap() - CELL_SIZE);
    }
    positions.get(index).unwrap().to_owned()
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
