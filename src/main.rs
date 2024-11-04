mod maze;
mod utils;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_third_person_camera::*;
use maze::{calc_maze_dims, maze_from_xyz_seed};
use std::{
    borrow::Cow,
    collections::HashSet,
    env,
    f32::consts::PI,
    fmt::{Display, Formatter, Result},
};
use utils::dev::write_mazes_to_html_file;

const CELL_SIZE: f32 = 4.0;
const CHUNK_SIZE: f32 = 16.0;
const DEFAULT_CHUNK_XYZ: (i64, i64, i64) = (0, 0, 0);

const PLAYER_SIZE: f32 = 1.0;
const DEFAULT_PLAYER_SPEED: f32 = 4.0;

const CAMERA_X: f32 = -2.0;
const CAMERA_Y: f32 = 2.5;
const CAMERA_Z: f32 = 5.0;
const CAMERA_ZOOM_MIN: f32 = 1.0;
const CAMERA_ZOOM_MAX: f32 = 1000.0;

const SEED: u32 = 1234;
const HTML_FILE_OUTPUT_PATH: &str = "maze.html";

#[derive(Debug)]
enum ArgName {
    Html,
}

impl Display for ArgName {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            ArgName::Html => write!(f, "html"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
struct ActiveChunk(i64, i64, i64);

#[derive(Event)]
struct ActiveChunkChange {
    value: ActiveChunk,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Speed(f32);

#[derive(Component)]
struct Chunk(i64, i64, i64);

#[derive(Clone, Component, Debug, Default)]
pub struct Cell {
    wall_top: bool,
    wall_bottom: bool,
    wall_left: bool,
    wall_right: bool,
    floor: bool,
    ceiling: bool,
}

#[derive(Component)]
struct CellObject;

#[derive(Component, Reflect)]
struct Collider(f32, f32, f32);

fn make_neighboring_xyz_chunks(chunk: (i64, i64, i64)) -> Vec<(i64, i64, i64)> {
    let (x, y, z) = chunk;
    (x - 1..=x + 1)
        .flat_map(|i| (y - 1..=y + 1).flat_map(move |j| (z - 1..=z + 1).map(move |k| (i, j, k))))
        .collect()
}

fn spawn_initial_chunks(
    mut commands: Commands,
    active_chunk: Res<State<ActiveChunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let chunks = make_neighboring_xyz_chunks((active_chunk.0, active_chunk.1, active_chunk.2));
    for xyz in chunks {
        spawn_new_chunk_bundle(xyz, &mut commands, &mut meshes, &mut materials);
    }
}

fn manage_active_chunk(
    mut active_chunk_change_event_writer: EventWriter<ActiveChunkChange>,
    player_query: Query<&GlobalTransform, With<Player>>,
    active_chunk: Res<State<ActiveChunk>>,
) {
    let player_gl_transform = player_query.get_single().expect("Error retrieving player:");
    let player_gl_translation = player_gl_transform.translation();

    let mut chunk = active_chunk.clone();

    let half_chunk_size = CHUNK_SIZE / 2.0;

    let x_chunk_size = active_chunk.0 as f32 * CHUNK_SIZE;
    let x_min_crossed = player_gl_translation.x < x_chunk_size - half_chunk_size;
    let x_max_crossed = player_gl_translation.x > x_chunk_size + half_chunk_size;

    if x_min_crossed {
        // Spawn new chunk column at far left, and despawn right-most chunk column
        chunk.0 -= 1;
    } else if x_max_crossed {
        // Spawn new chunk column at far right, and despawn left-most chunk column
        chunk.0 += 1;
    }

    let z_chunk_size = active_chunk.2 as f32 * CHUNK_SIZE;
    let z_min_crossed = player_gl_translation.z < z_chunk_size - half_chunk_size;
    let z_max_crossed = player_gl_translation.z > z_chunk_size + half_chunk_size;

    if z_min_crossed {
        // Spawn new chunk row at far top, and despawn lowest chunk row
        chunk.2 -= 1;
    } else if z_max_crossed {
        // Spawn new chunk row at far bottom, and despawn highest chunk row
        chunk.2 += 1;
    }

    if x_min_crossed || x_max_crossed || z_min_crossed || z_max_crossed {
        active_chunk_change_event_writer.send(ActiveChunkChange { value: chunk });
    }
}

fn handle_active_chunk_change(
    mut commands: Commands,
    chunks_query: Query<(Entity, &Chunk)>,
    mut active_chunk_change_event_reader: EventReader<ActiveChunkChange>,
    mut next_active_chunk: ResMut<NextState<ActiveChunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in active_chunk_change_event_reader.read() {
        let new_active_chunk = event.value;
        next_active_chunk.set(event.value);

        let new_chunks = make_neighboring_xyz_chunks((
            new_active_chunk.0,
            new_active_chunk.1,
            new_active_chunk.2,
        ));

        let mut existing_chunks: HashSet<(i64, i64, i64)> = HashSet::new();

        // Despawn chunks that are not in the new chunks
        for (chunk_entity, chunk) in chunks_query.iter() {
            let xyz = (chunk.0, chunk.1, chunk.2);
            if !new_chunks.contains(&xyz) {
                commands.entity(chunk_entity).despawn_recursive();
            }
            existing_chunks.insert(xyz);
        }

        // Spawn new chunks that are not currently existing
        for (x, y, z) in new_chunks {
            if !existing_chunks.contains(&(x, y, z)) {
                spawn_new_chunk_bundle((x, y, z), &mut commands, &mut meshes, &mut materials);
            }
        }
    }
}

fn spawn_new_chunk_bundle(
    (chunk_x, chunk_y, chunk_z): (i64, i64, i64),
    commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
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
        Chunk(chunk_x, chunk_y, chunk_z),
        Name::new(format!("Chunk ({},{},{})", chunk_x, chunk_y, chunk_z)),
    );

    commands.spawn(chunk_bundle).with_children(|parent| {
        // One maze is created per chunk
        let (height, width) = calc_maze_dims(CHUNK_SIZE, CELL_SIZE);
        let maze = &maze_from_xyz_seed(chunk_x, chunk_y, chunk_z, SEED, height, width);

        for (x, row) in maze.iter().enumerate() {
            for (z, cell) in row.iter().enumerate() {
                let cell_bundle = (
                    TransformBundle {
                        local: Transform::from_xyz(calc_floor_pos(x), 0.0, calc_floor_pos(z)),
                        ..default()
                    },
                    InheritedVisibility::default(),
                    cell.clone(),
                    Name::new(format!("Cell ({},{})", x, z)),
                );

                parent.spawn(cell_bundle).with_children(|grandparent| {
                    // Floor
                    if cell.floor {
                        grandparent.spawn((
                            PbrBundle {
                                mesh: meshes
                                    .add(Plane3d::default().mesh().size(CELL_SIZE, CELL_SIZE)),
                                material: materials.add(Color::linear_rgba(0.55, 0.0, 0.0, 1.0)),
                                ..default()
                            },
                            CellObject,
                            Name::new("Floor"),
                        ));
                    }

                    // Ceiling
                    if cell.ceiling {
                        let mut transform = Transform::default();
                        let x_180_deg_rotation = Quat::from_rotation_x(PI);
                        transform.rotate(x_180_deg_rotation);

                        grandparent.spawn((
                            PbrBundle {
                                mesh: meshes
                                    .add(Plane3d::default().mesh().size(CELL_SIZE, CELL_SIZE)),
                                material: materials.add(Color::linear_rgba(0.0, 0.2, 0.4, 1.0)),
                                transform,
                                ..default()
                            },
                            CellObject,
                            Name::new("Ceiling"),
                        ));
                    }

                    // Top wall
                    if cell.wall_top {
                        let mut transform =
                            Transform::from_xyz(CELL_SIZE / 2.0, CELL_SIZE / 2.0, 0.0);
                        let z_90_deg_rotation = Quat::from_rotation_z(PI / 2.0);
                        transform.rotate(z_90_deg_rotation);

                        grandparent.spawn(new_cell_wall_bundle(
                            "Top Wall",
                            transform,
                            Collider(0.1, CELL_SIZE / 2.0, CELL_SIZE),
                            &mut meshes,
                            &mut materials,
                        ));
                    }

                    // Left wall
                    if cell.wall_left {
                        let mut transform =
                            Transform::from_xyz(0.0, CELL_SIZE / 2.0, CELL_SIZE / 2.0);
                        let x_270_deg_rotation = Quat::from_rotation_x(PI * 3.0 / 2.0);
                        transform.rotate(x_270_deg_rotation);

                        grandparent.spawn(new_cell_wall_bundle(
                            "Left Wall",
                            transform,
                            Collider(CELL_SIZE, CELL_SIZE / 2.0, 0.1),
                            &mut meshes,
                            &mut materials,
                        ));
                    }

                    // Bottom wall
                    if cell.wall_bottom {
                        let mut transform =
                            Transform::from_xyz(-CELL_SIZE / 2.0, CELL_SIZE / 2.0, 0.0);
                        let z_270_deg_rotation = Quat::from_rotation_z(PI * 3.0 / 2.0);
                        transform.rotate(z_270_deg_rotation);

                        grandparent.spawn(new_cell_wall_bundle(
                            "Bottom Wall",
                            transform,
                            Collider(0.1, CELL_SIZE / 2.0, CELL_SIZE),
                            &mut meshes,
                            &mut materials,
                        ));
                    }

                    // Right wall
                    if cell.wall_right {
                        let mut transform =
                            Transform::from_xyz(0.0, CELL_SIZE / 2.0, -CELL_SIZE / 2.0);
                        let x_90_deg_rotation = Quat::from_rotation_x(PI / 2.0);
                        transform.rotate(x_90_deg_rotation);

                        grandparent.spawn(new_cell_wall_bundle(
                            "Right Wall",
                            transform,
                            Collider(CELL_SIZE, CELL_SIZE / 2.0, 0.1),
                            &mut meshes,
                            &mut materials,
                        ));
                    }
                });
            }
        }
    });
}

fn new_cell_wall_bundle(
    name: impl Into<Cow<'static, str>>,
    transform: Transform,
    collider: Collider,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> impl Bundle {
    (
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(CELL_SIZE, CELL_SIZE)),
            material: materials.add(Color::linear_rgba(0.15, 0.0, 0.55, 1.0)),
            transform,
            ..default()
        },
        CellObject,
        collider,
        Name::new(name),
    )
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

fn spawn_camera(mut commands: Commands) {
    let camera_bundle = (
        Camera3dBundle {
            transform: Transform::from_xyz(CAMERA_X, CAMERA_Y, CAMERA_Z)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ThirdPersonCamera {
            zoom: Zoom::new(CAMERA_ZOOM_MIN, CAMERA_ZOOM_MAX),
            ..default()
        },
        Name::new("Camera"),
    );

    commands.spawn(camera_bundle);
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_bundle = (
        PbrBundle {
            mesh: meshes.add(Cuboid::new(PLAYER_SIZE, PLAYER_SIZE, PLAYER_SIZE)),
            material: materials.add(Color::linear_rgba(0.0, 0.0, 0.3, 1.0)),
            transform: Transform::from_xyz(2.0, PLAYER_SIZE / 2.0, 2.0),
            ..default()
        },
        Collider(PLAYER_SIZE, PLAYER_SIZE, PLAYER_SIZE),
        Player,
        ThirdPersonCameraTarget,
        Speed(DEFAULT_PLAYER_SPEED),
        Name::new("Player"),
    );

    commands.spawn(player_bundle);
}

fn player_movement(
    mut player_query: Query<(&mut Transform, &GlobalTransform, &Collider, &Speed), With<Player>>,
    chunks_query: Query<(&Chunk, &Children), Without<Player>>,
    cells_query: Query<&Children, (With<Cell>, Without<Player>)>,
    cell_objects_query: Query<(&GlobalTransform, &Collider), (With<CellObject>, Without<Player>)>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    active_chunk: Res<State<ActiveChunk>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut player_transform, player_gl_transform, player_collider, player_speed) in
        player_query.iter_mut()
    {
        let camera_transform = match camera_query.get_single() {
            Ok(ct) => ct,
            Err(err) => Err(format!("Error retrieving camera: {}", err)).unwrap(),
        };

        let mut direction = Vec3::default();

        // Forward
        if keys.pressed(KeyCode::KeyW) {
            let d = camera_transform.forward();
            direction.x += d.x;
            direction.z += d.z;
        }
        // Back
        if keys.pressed(KeyCode::KeyS) {
            let d = camera_transform.back();
            direction.x += d.x;
            direction.z += d.z;
        }
        // Left
        if keys.pressed(KeyCode::KeyA) {
            let d = camera_transform.left();
            direction.x += d.x;
            direction.z += d.z;
        }
        // Right
        if keys.pressed(KeyCode::KeyD) {
            let d = camera_transform.right();
            direction.x += d.x;
            direction.z += d.z;
        }

        let mut player_gl_translation = player_gl_transform.translation();
        let movement = direction.normalize_or_zero() * player_speed.0 * time.delta_seconds();
        player_gl_translation += movement;

        if direction.length_squared() > 0.0 {
            player_transform.look_to(direction, Vec3::Y);
        }

        for (chunk, children) in chunks_query.iter() {
            // Check collision of only cells in the active chunk
            if chunk.0 != active_chunk.0 || chunk.1 != active_chunk.1 || chunk.2 != active_chunk.2 {
                continue;
            }

            for &child in children.iter() {
                let grandchildren = match cells_query.get(child) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                for &grandchild in grandchildren.iter() {
                    let (cell_object_gl_transform, cell_object_collider) =
                        match cell_objects_query.get(grandchild) {
                            Ok(c) => c,
                            Err(_) => continue,
                        };

                    if are_colliding(
                        (&player_gl_translation, player_collider),
                        (
                            &cell_object_gl_transform.translation(),
                            cell_object_collider,
                        ),
                    ) {
                        return;
                    }
                }
            }
        }

        player_transform.translation += movement;
    }
}

fn are_colliding(e1: (&Vec3, &Collider), e2: (&Vec3, &Collider)) -> bool {
    let (t1, c1) = e1;
    let (t2, c2) = e2;

    // x
    let e1_x_top = t1.x + c1.0 / 2.0;
    let e1_x_bottom = t1.x - c1.0 / 2.0;
    let e2_x_top = t2.x + c2.0 / 2.0;
    let e2_x_bottom = t2.x - c2.0 / 2.0;

    let e1_x_overlapping_above = e1_x_top >= e2_x_top && e2_x_top >= e1_x_bottom;
    let e2_x_overlapping_above = e1_x_top >= e2_x_bottom && e2_x_bottom >= e1_x_bottom;
    let e1_x_overlapping_below = e2_x_top >= e1_x_top && e1_x_top >= e2_x_bottom;
    let e2_x_overlapping_below = e2_x_top >= e1_x_bottom && e1_x_bottom >= e2_x_bottom;

    let x_overlapping = e1_x_overlapping_above
        || e2_x_overlapping_above
        || e1_x_overlapping_below
        || e2_x_overlapping_below;

    // y
    let e1_y_top = t1.y + c1.1 / 2.0;
    let e1_y_bottom = t1.y - c1.1 / 2.0;
    let e2_y_top = t2.y + c2.1 / 2.0;
    let e2_y_bottom = t2.y - c2.1 / 2.0;

    let e1_y_overlapping_above = e1_y_top >= e2_y_top && e2_y_top >= e1_y_bottom;
    let e2_y_overlapping_above = e1_y_top >= e2_y_bottom && e2_y_bottom >= e1_y_bottom;
    let e1_y_overlapping_below = e2_y_top >= e1_y_top && e1_y_top >= e2_y_bottom;
    let e2_y_overlapping_below = e2_y_top >= e1_y_bottom && e1_y_bottom >= e2_y_bottom;

    let y_overlapping = e1_y_overlapping_above
        || e2_y_overlapping_above
        || e1_y_overlapping_below
        || e2_y_overlapping_below;

    // z
    let e1_z_top = t1.z + c1.2 / 2.0;
    let e1_z_bottom = t1.z - c1.2 / 2.0;
    let e2_z_top = t2.z + c2.2 / 2.0;
    let e2_z_bottom = t2.z - c2.2 / 2.0;

    let e1_z_overlapping_above = e1_z_top >= e2_z_top && e2_z_top >= e1_z_bottom;
    let e2_z_overlapping_above = e1_z_top >= e2_z_bottom && e2_z_bottom >= e1_z_bottom;
    let e1_z_overlapping_below = e2_z_top >= e1_z_top && e1_z_top >= e2_z_bottom;
    let e2_z_overlapping_below = e2_z_top >= e1_z_bottom && e1_z_bottom >= e2_z_bottom;

    let z_overlapping = e1_z_overlapping_above
        || e2_z_overlapping_above
        || e1_z_overlapping_below
        || e2_z_overlapping_below;

    x_overlapping && y_overlapping && z_overlapping
}

fn main() {
    assert_eq!(
        CHUNK_SIZE % CELL_SIZE,
        0.0,
        "expected chunk size ({}) to be divisible by cell size ({})",
        CHUNK_SIZE,
        CELL_SIZE,
    );

    let args: Vec<String> = env::args().collect();

    if args.contains(&ArgName::Html.to_string()) {
        let (height, width) = calc_maze_dims(CHUNK_SIZE, CELL_SIZE);
        let chunks = make_neighboring_xyz_chunks(DEFAULT_CHUNK_XYZ);

        let mut mazes = vec![];
        for (chunk_x, chunk_y, chunk_z) in chunks {
            let maze = maze_from_xyz_seed(chunk_x, chunk_y, chunk_z, SEED, height, width);
            mazes.push(maze);
        }

        write_mazes_to_html_file(&mazes, HTML_FILE_OUTPUT_PATH).unwrap();
    }

    App::new()
        .register_type::<Collider>()
        .add_plugins((
            DefaultPlugins,
            ThirdPersonCameraPlugin,
            WorldInspectorPlugin::new(),
        ))
        .init_state::<ActiveChunk>()
        .add_event::<ActiveChunkChange>()
        .add_systems(Startup, (spawn_initial_chunks, spawn_camera, spawn_player))
        .add_systems(
            Update,
            (
                player_movement,
                manage_active_chunk,
                handle_active_chunk_change,
            ),
        )
        .run();
}
