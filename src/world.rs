use crate::{
    animation::CyclicAnimation,
    interaction::Interactable,
    maze::{calc_maze_dims, maze_from_rng},
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

const WALL_BREAK_PROB: f64 = 0.2;
const WORLD_STRUCTURE_GEN_PROB: f64 = 0.1;

const CHAIR_COLLIDER_HX: f32 = 0.2;
const CHAIR_COLLIDER_HY: f32 = 0.25;
const CHAIR_COLLIDER_HZ: f32 = 0.2;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ActiveChunk>()
            .add_event::<ActiveChunkChangeRequest>()
            .add_systems(Startup, spawn_initial_chunks)
            .add_systems(Update, (manage_active_chunk, handle_active_chunk_change));
    }
}

#[derive(Clone, Component, Debug, Default)]
pub struct Cell {
    pub wall_top: bool,
    pub wall_bottom: bool,
    pub wall_left: bool,
    pub wall_right: bool,
    pub floor: bool,
    pub ceiling: bool,
    pub special: CellSpecial,
}

#[derive(Clone, Debug, Default, EnumIter, PartialEq)]
pub enum CellSpecial {
    #[default]
    None,
    Chair,
    TreasureChest,
}

impl CellSpecial {
    fn spawn_prob(&self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Chair => 0.48,
            Self::TreasureChest => 0.98,
        }
    }
}

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

impl ChunkCellMarker {
    fn _from_global_transform(gt: &GlobalTransform) -> Self {
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
}

fn spawn_initial_chunks(
    mut commands: Commands,
    active_chunk: Res<State<ActiveChunk>>,
    game_settings: Res<State<GameSettings>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let render_dist = game_settings.chunk_render_dist;
    let chunks = make_neighboring_chunks_xyz(
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in event_reader.read() {
        let chunk_xyz = event.value.to_tuple();
        next_active_chunk.set(event.value);

        let rend_dist = game_settings.chunk_render_dist;
        let new_chunks =
            make_neighboring_chunks_xyz(chunk_xyz, rend_dist.0, rend_dist.1, rend_dist.2);

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
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let chunk_bundle = (
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(CHUNK_SIZE, CHUNK_SIZE)),
            material: materials.add(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
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
        // One maze is created per chunk
        let (height, width) = calc_maze_dims(CHUNK_SIZE, CELL_SIZE);
        let chunk = &chunk_from_xyz_seed(SEED, height, width, chunk_x, chunk_y, chunk_z);

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
                                            scene: asset_server.load("models/Chair.glb#Scene0"),
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
                                            CHAIR_COLLIDER_HY * 2.0,
                                            0.0,
                                        ),
                                        ..default()
                                    },
                                    Collider::cuboid(
                                        CHAIR_COLLIDER_HX,
                                        CHAIR_COLLIDER_HY,
                                        CHAIR_COLLIDER_HZ,
                                    ),
                                    Interactable { range: 2.0 },
                                    CyclicAnimation::new(0, 1),
                                    Name::new("Treasure Chest"),
                                ))
                                .with_children(|ggp| {
                                    ggp.spawn((
                                        SceneBundle {
                                            scene: asset_server
                                                .load(GltfAssetLabel::Scene(0).from_asset(
                                                    "models/Treasure_Chest.glb#Scene0",
                                                )),
                                            transform: Transform::from_xyz(
                                                0.0,
                                                -CHAIR_COLLIDER_HY,
                                                0.0,
                                            ),
                                            ..default()
                                        },
                                        Name::new("Treasure Chest Model"),
                                    ));
                                });
                        }
                    }

                    let mesh = meshes.add(Plane3d::default().mesh().size(CELL_SIZE, CELL_SIZE));

                    // Floor
                    if cell.floor {
                        grandparent.spawn((
                            PbrBundle {
                                mesh: mesh.clone(),
                                material: materials.add(Color::linear_rgba(0.55, 0.0, 0.0, 1.0)),
                                ..default()
                            },
                            Collider::cuboid(CELL_SIZE / 2.0, 0.1, CELL_SIZE / 2.0),
                            Name::new("Floor"),
                        ));
                    }

                    // Ceiling
                    if cell.ceiling {
                        grandparent.spawn((
                            PbrBundle {
                                mesh: mesh.clone(),
                                material: materials.add(Color::linear_rgba(0.0, 0.2, 0.4, 1.0)),
                                transform: Transform::default()
                                    .with_rotation(Quat::from_rotation_x(PI)),
                                ..default()
                            },
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
                    if cell.wall_top {
                        grandparent.spawn((
                            PbrBundle {
                                mesh: mesh.clone(),
                                material: material.clone(),
                                transform: Transform::from_xyz(
                                    CELL_SIZE / 2.0,
                                    CELL_SIZE / 2.0,
                                    0.0,
                                )
                                .with_rotation(Quat::from_rotation_z(PI / 2.0)),
                                ..default()
                            },
                            Name::new("Top Wall"),
                        ));
                    }

                    // Bottom wall
                    if cell.wall_bottom {
                        grandparent.spawn((
                            PbrBundle {
                                mesh: mesh.clone(),
                                material: material.clone(),
                                transform: Transform::from_xyz(
                                    -CELL_SIZE / 2.0,
                                    CELL_SIZE / 2.0,
                                    0.0,
                                )
                                .with_rotation(Quat::from_rotation_z(PI * 3.0 / 2.0)),
                                ..default()
                            },
                            Collider::cuboid(CELL_SIZE / 2.0, 0.1, CELL_SIZE / 2.0),
                            Name::new("Bottom Wall"),
                        ));
                    }

                    // Left wall
                    if cell.wall_left {
                        grandparent.spawn((
                            PbrBundle {
                                mesh: mesh.clone(),
                                material: material.clone(),
                                transform: Transform::from_xyz(
                                    0.0,
                                    CELL_SIZE / 2.0,
                                    CELL_SIZE / 2.0,
                                )
                                .with_rotation(Quat::from_rotation_x(PI * 3.0 / 2.0)),
                                ..default()
                            },
                            Name::new("Left Wall"),
                        ));
                    }

                    // Right wall
                    if cell.wall_right {
                        grandparent.spawn((
                            PbrBundle {
                                mesh: mesh.clone(),
                                material: material.clone(),
                                transform: Transform::from_xyz(
                                    0.0,
                                    CELL_SIZE / 2.0,
                                    -CELL_SIZE / 2.0,
                                )
                                .with_rotation(Quat::from_rotation_x(PI / 2.0)),
                                ..default()
                            },
                            Collider::cuboid(CELL_SIZE / 2.0, 0.1, CELL_SIZE / 2.0),
                            Name::new("Right Wall"),
                        ));
                    }
                });
            }
        }
    });
}

fn calc_floor_pos(index: usize) -> f32 {
    let num_cells_per_chunk = (CHUNK_SIZE / CELL_SIZE) as usize;
    let mut positions = vec![CELL_SIZE / 2.0, -CELL_SIZE / 2.0];
    while positions.len() < num_cells_per_chunk {
        positions.insert(0, positions[0] + CELL_SIZE);
        positions.push(positions.last().unwrap() - CELL_SIZE);
    }
    positions.get(index).unwrap().to_owned()
}

pub fn make_neighboring_chunks_xyz(
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
