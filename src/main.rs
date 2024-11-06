mod maze;
mod maze_test;
mod utils;

use bevy::{animation::animate_targets, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use bevy_third_person_camera::*;
use maze::{calc_maze_dims, maze_from_xyz_seed};
use std::{
    borrow::Cow,
    collections::HashSet,
    env,
    f32::consts::PI,
    fmt::{Display, Formatter, Result},
    time::Duration,
};
use strum_macros::EnumIter;
use utils::dev::write_mazes_to_html_file;

const CELL_SIZE: f32 = 4.0;
const CHUNK_SIZE: f32 = 16.0;
const DEFAULT_CHUNK_XYZ: (i64, i64, i64) = (0, 0, 0);

const PLAYER_COLLIDER_HX: f32 = 0.4;
const PLAYER_COLLIDER_HY: f32 = 0.85;
const PLAYER_COLLIDER_HZ: f32 = 0.4;
const PLAYER_SPAWN_XYZ: (f32, f32, f32) = (2.0, 1.0, 2.0);
const DEFAULT_PLAYER_SPEED: f32 = 300.0;

const CAMERA_X: f32 = -2.0;
const CAMERA_Y: f32 = 2.5;
const CAMERA_Z: f32 = 5.0;
const CAMERA_ZOOM_MIN: f32 = 1.0;
const CAMERA_ZOOM_MAX: f32 = 10.0;
const CAMERA_SENSITIVITY: f32 = 2.5;

const CHAIR_COLLIDER_HX: f32 = 0.2;
const CHAIR_COLLIDER_HY: f32 = 0.25;
const CHAIR_COLLIDER_HZ: f32 = 0.2;

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

#[derive(Resource)]
struct Animations {
    nodes: Vec<AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
enum PlayerState {
    #[default]
    Walking,
    ClimbingLadder(String),
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
enum PlayerAnimation {
    #[default]
    Idle,
    Jogging,
}

impl PlayerAnimation {
    fn index(&self) -> usize {
        match self {
            PlayerAnimation::Idle => 0,
            PlayerAnimation::Jogging => 1,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
struct PendingInteractable(Option<String>);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
struct ActiveChunk(i64, i64, i64);

#[derive(Event)]
struct ActiveChunkChange {
    value: ActiveChunk,
}

#[derive(Component)]
struct Player;

#[derive(Component, Reflect)]
struct Speed(f32);

#[derive(Component)]
struct Chunk(i64, i64, i64);

#[derive(Clone, Debug, Default, EnumIter, PartialEq)]
enum CellSpecial {
    #[default]
    None,
    Ladder,
    Slope,
    Chair,
}

impl CellSpecial {
    fn spawn_prob(&self) -> f64 {
        match self {
            CellSpecial::None => 0.0,
            CellSpecial::Ladder => 0.08,
            CellSpecial::Slope => 0.08,
            CellSpecial::Chair => 0.06,
        }
    }
}

#[derive(Clone, Component, Debug, Default)]
pub struct Cell {
    wall_top: bool,
    wall_bottom: bool,
    wall_left: bool,
    wall_right: bool,
    floor: bool,
    ceiling: bool,
    special: CellSpecial,
}

enum InteractableKind {
    Ladder,
}

#[derive(Component)]
struct Interactable {
    id: String,
    range: f32,
    kind: InteractableKind,
}

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

        ChunkCellMarker {
            chunk_x,
            chunk_y,
            chunk_z,
            x,
            z,
        }
    }
}

fn make_neighboring_xyz_chunks(chunk: (i64, i64, i64)) -> Vec<(i64, i64, i64)> {
    let (x, y, z) = chunk;
    (x - 1..=x + 1)
        .flat_map(|i| (y - 1..=y + 1).flat_map(move |j| (z - 1..=z + 1).map(move |k| (i, j, k))))
        .collect()
}

fn spawn_initial_chunks(
    mut commands: Commands,
    active_chunk: Res<State<ActiveChunk>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let chunks = make_neighboring_xyz_chunks((active_chunk.0, active_chunk.1, active_chunk.2));
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
    mut active_chunk_change_event_writer: EventWriter<ActiveChunkChange>,
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
        active_chunk_change_event_writer.send(ActiveChunkChange { value: chunk });
    }
}

fn handle_active_chunk_change(
    mut active_chunk_change_event_reader: EventReader<ActiveChunkChange>,
    mut commands: Commands,
    chunks_query: Query<(Entity, &Chunk)>,
    mut next_active_chunk: ResMut<NextState<ActiveChunk>>,
    asset_server: Res<AssetServer>,
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
        Name::new(format!("Chunk_({},{},{})", chunk_x, chunk_y, chunk_z)),
    );

    commands.spawn(chunk_bundle).with_children(|parent| {
        // One maze is created per chunk
        let (height, width) = calc_maze_dims(CHUNK_SIZE, CELL_SIZE);
        let maze = &maze_from_xyz_seed(SEED, height, width, chunk_x, chunk_y, chunk_z);

        for (x, row) in maze.iter().enumerate() {
            for (z, cell) in row.iter().enumerate() {
                let ccm = ChunkCellMarker {
                    chunk_x,
                    chunk_y,
                    chunk_z,
                    x,
                    z,
                };

                let cell_bundle = (
                    TransformBundle {
                        local: Transform::from_xyz(calc_floor_pos(x), 0.0, calc_floor_pos(z)),
                        ..default()
                    },
                    InheritedVisibility::default(),
                    cell.clone(),
                    ccm.clone(),
                    Name::new(format!("Cell_({},{})", x, z)),
                );

                parent.spawn(cell_bundle).with_children(|grandparent| {
                    // Special
                    match cell.special {
                        CellSpecial::None => (),
                        CellSpecial::Ladder => {
                            grandparent.spawn((
                                PbrBundle {
                                    mesh: meshes.add(Cuboid::new(0.5, CELL_SIZE, 0.5)),
                                    material: materials.add(Color::linear_rgba(0.3, 0.2, 0.7, 1.0)),
                                    transform: Transform::from_xyz(1.0, CELL_SIZE / 2.0, 1.0),
                                    ..default()
                                },
                                Collider::cuboid(0.25, CELL_SIZE / 2.0, 0.25),
                                ccm.clone(),
                                Interactable {
                                    id: format!(
                                        "Ladder_({},{},{})_({},{})",
                                        chunk_x, chunk_y, chunk_z, x, z
                                    ),
                                    range: 2.0,
                                    kind: InteractableKind::Ladder,
                                },
                                Name::new("Ladder"),
                            ));
                        }
                        CellSpecial::Slope => {
                            let mut transform = Transform::from_xyz(0.0, CELL_SIZE / 2.0, 0.0);
                            let z_45_deg_rotation = Quat::from_rotation_z(PI / 4.0);
                            transform.rotate(z_45_deg_rotation);

                            let cell_size_squared = CELL_SIZE.powi(2);
                            let height = (cell_size_squared + cell_size_squared).sqrt(); // calculate hypotenuse

                            grandparent.spawn((
                                PbrBundle {
                                    mesh: meshes
                                        .add(Plane3d::default().mesh().size(height, CELL_SIZE)),
                                    material: materials.add(Color::linear_rgba(0.3, 0.2, 0.7, 1.0)),
                                    transform,
                                    ..default()
                                },
                                Collider::cuboid(height / 2.0, 0.1, CELL_SIZE / 2.0),
                                Name::new("Slope"),
                            ));
                        }
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
                                            scene: asset_server.load("Chair.glb#Scene0"),
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
                    }

                    // Floor
                    if cell.floor {
                        grandparent.spawn((
                            PbrBundle {
                                mesh: meshes
                                    .add(Plane3d::default().mesh().size(CELL_SIZE, CELL_SIZE)),
                                material: materials.add(Color::linear_rgba(0.55, 0.0, 0.0, 1.0)),
                                ..default()
                            },
                            Collider::cuboid(CELL_SIZE / 2.0, 0.1, CELL_SIZE / 2.0),
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
        Collider::cuboid(CELL_SIZE / 2.0, 0.1, CELL_SIZE / 2.0),
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
            sensitivity: Vec2 {
                x: CAMERA_SENSITIVITY,
                y: CAMERA_SENSITIVITY,
            },
            ..default()
        },
        Name::new("Camera"),
    );

    commands.spawn(camera_bundle);
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_bundle = (
        SpatialBundle {
            transform: Transform::from_xyz(
                PLAYER_SPAWN_XYZ.0,
                PLAYER_SPAWN_XYZ.1,
                PLAYER_SPAWN_XYZ.2,
            ),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(PLAYER_COLLIDER_HX, PLAYER_COLLIDER_HY, PLAYER_COLLIDER_HZ),
        ExternalImpulse::default(),
        Velocity::default(),
        GravityScale(30.0),
        LockedAxes::ROTATION_LOCKED_X
            | LockedAxes::ROTATION_LOCKED_Y
            | LockedAxes::ROTATION_LOCKED_Z,
        Player,
        ThirdPersonCameraTarget,
        Speed(DEFAULT_PLAYER_SPEED),
        Name::new("Player"),
    );

    commands.spawn(player_bundle).with_children(|parent| {
        parent.spawn((
            SceneBundle {
                scene: asset_server.load(
                    GltfAssetLabel::Scene(PlayerAnimation::Idle.index()).from_asset("Man.glb"),
                ),
                transform: Transform::from_xyz(0.0, -PLAYER_COLLIDER_HY, 0.0),
                ..default()
            },
            Name::new("Player Model"),
        ));
    });
}

fn player_movement(
    mut player_query: Query<
        (
            &mut Transform,
            &GlobalTransform,
            &mut Velocity,
            &mut ExternalImpulse,
            &Speed,
        ),
        With<Player>,
    >,
    interactables_query: Query<(&Interactable, &GlobalTransform, &ChunkCellMarker)>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    player_state: Res<State<PlayerState>>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (
        mut player_transform,
        player_gl_transform,
        mut player_velocity,
        mut player_external_impulse,
        player_speed,
    ) in player_query.iter_mut()
    {
        let camera_transform = match camera_query.get_single() {
            Ok(ct) => ct,
            Err(err) => Err(format!("Error retrieving camera: {}", err)).unwrap(),
        };

        let mut player_gl_translation = player_gl_transform.translation();

        let movement = match player_state.get().to_owned() {
            PlayerState::ClimbingLadder(id) => {
                let mut direction = Vec3::default();

                if let Some((_, ladder_gl_transform, ladder_marker)) =
                    interactables_query.iter().find(|(i, _, _)| i.id == id)
                {
                    let ladder_floor_gl_y = ladder_gl_transform.translation().y - (CELL_SIZE / 2.0);
                    let ladder_ceiling_gl_y = ladder_floor_gl_y + CELL_SIZE;

                    // Up ladder
                    if keys.pressed(KeyCode::KeyW) {
                        if player_gl_translation.y < ladder_ceiling_gl_y {
                            // Move player up ladder
                            direction.y += camera_transform.up().y;
                        } else if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
                            // Check if there is a ladder above this cell
                            if let Some((ladder_above, _, _)) =
                                interactables_query.iter().find(|(_, _, m)| {
                                    m.chunk_x == ladder_marker.chunk_x
                                        && m.chunk_y == ladder_marker.chunk_y + 1
                                        && m.chunk_z == ladder_marker.chunk_z
                                        && m.x == ladder_marker.x
                                        && m.z == ladder_marker.z
                                })
                            {
                                next_player_state
                                    .set(PlayerState::ClimbingLadder(ladder_above.id.to_owned()));
                            }
                        } else {
                            // Exit ladder climb
                            player_transform.translation.y = ladder_ceiling_gl_y;
                            next_player_state.set(PlayerState::Walking);
                        }
                    }
                    // Down ladder
                    if keys.pressed(KeyCode::KeyS) {
                        if player_gl_translation.y > ladder_floor_gl_y {
                            // Move player down ladder
                            direction.y += camera_transform.down().y;
                        } else if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
                            // Check if there is a ladder below this cell
                            if let Some((ladder_below, _, _)) =
                                interactables_query.iter().find(|(_, _, m)| {
                                    m.chunk_x == ladder_marker.chunk_x
                                        && m.chunk_y == ladder_marker.chunk_y - 1
                                        && m.chunk_z == ladder_marker.chunk_z
                                        && m.x == ladder_marker.x
                                        && m.z == ladder_marker.z
                                })
                            {
                                next_player_state
                                    .set(PlayerState::ClimbingLadder(ladder_below.id.to_owned()));
                            }
                        } else {
                            // Exit ladder climb
                            player_transform.translation.y = ladder_floor_gl_y;
                            next_player_state.set(PlayerState::Walking);
                        }
                    }
                }

                let mvmt = direction.normalize_or_zero() * player_speed.0 * time.delta_seconds();
                player_gl_translation += mvmt;

                mvmt
            }
            _ => {
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

                let mvmt = direction.normalize_or_zero() * player_speed.0 * time.delta_seconds();

                if direction.length_squared() > 0.0 {
                    // Face player in inverse direction of impulse
                    let inv = direction
                        * Vec3 {
                            x: -1.0,
                            y: -1.0,
                            z: -1.0,
                        };
                    player_transform.look_to(inv, Vec3::Y);
                }

                mvmt
            }
        };

        player_velocity.linvel = Vec3::ZERO; // Reset player velocity

        player_external_impulse.impulse = movement;
    }
}

fn update_pending_interactable(
    mut player_query: Query<&GlobalTransform, With<Player>>,
    interactables_query: Query<(&Interactable, &GlobalTransform)>,
    mut next_pending_interactable: ResMut<NextState<PendingInteractable>>,
) {
    let player_gl_transform = player_query
        .get_single_mut()
        .expect("Error retrieving player");
    let player_gl_translation = player_gl_transform.translation();

    // Check if player is in range of any interactables
    let mut in_range = false;
    for (ibl, ibl_gl_transform) in interactables_query.iter() {
        let dist = player_gl_translation.distance(ibl_gl_transform.translation());

        if dist <= ibl.range {
            next_pending_interactable.set(PendingInteractable(Some(ibl.id.clone())));
            in_range = true;
            break;
        }
    }
    if !in_range {
        next_pending_interactable.set(PendingInteractable(None));
    }
}

fn handle_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    interactables_query: Query<(&Interactable, &GlobalTransform)>,
    pending_interactable: Res<State<PendingInteractable>>,
    player_state: Res<State<PlayerState>>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
) {
    let mut player_transform = player_query
        .get_single_mut()
        .expect("Error retrieving player");

    if keyboard_input.just_pressed(KeyCode::KeyE) {
        if let Some(id) = pending_interactable.get().0.to_owned() {
            for (interactable, interactable_gl_transform) in interactables_query.iter() {
                if interactable.id != id {
                    continue;
                }

                match interactable.kind {
                    InteractableKind::Ladder => match player_state.get() {
                        PlayerState::ClimbingLadder(_) => {
                            next_player_state.set(PlayerState::Walking)
                        }
                        _ => {
                            // Position player directly in front of ladder
                            let tl = interactable_gl_transform.translation();

                            player_transform.translation.x = tl.x;
                            player_transform.translation.y += 0.1;
                            player_transform.translation.z = tl.z;

                            let y = player_transform.translation.y;
                            player_transform.look_to(
                                Vec3 {
                                    x: tl.x,
                                    y,
                                    z: tl.z,
                                },
                                Dir3::Y,
                            );

                            next_player_state.set(PlayerState::ClimbingLadder(id));
                        }
                    },
                }
                return;
            }
        }
    }
}

fn setup_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Build the animation graph
    let mut graph = AnimationGraph::new();
    let nodes = graph
        .add_clips(
            [
                GltfAssetLabel::Animation(PlayerAnimation::Idle.index())
                    .from_asset("Man.glb#Animation1"),
                GltfAssetLabel::Animation(PlayerAnimation::Jogging.index())
                    .from_asset("Man.glb#Animation2"),
            ]
            .into_iter()
            .map(|path| asset_server.load(path)),
            1.0,
            graph.root,
        )
        .collect();

    // Insert a resource with the current scene information
    let graph = graphs.add(graph);
    commands.insert_resource(Animations {
        nodes,
        graph: graph.clone(),
    });
}

fn play_player_animation(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        transitions
            .play(&mut player, animations.nodes[0], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
}

fn change_player_animation(
    animations: Res<Animations>,
    mut player_query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    keys: Res<ButtonInput<KeyCode>>,
    player_animation: Res<State<PlayerAnimation>>,
    mut next_player_animation: ResMut<NextState<PlayerAnimation>>,
) {
    for (mut player, mut transitions) in player_query.iter_mut() {
        let is_moving =
            keys.any_pressed([KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD]);
        let animation = *player_animation.get();

        let i = if is_moving && animation != PlayerAnimation::Jogging {
            next_player_animation.set(PlayerAnimation::Jogging);
            PlayerAnimation::Jogging.index()
        } else if !is_moving && animation != PlayerAnimation::Idle {
            next_player_animation.set(PlayerAnimation::Idle);
            PlayerAnimation::Idle.index()
        } else {
            continue;
        };

        transitions
            .play(&mut player, animations.nodes[i], Duration::from_millis(250))
            .repeat();
    }
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
            let maze = maze_from_xyz_seed(SEED, height, width, chunk_x, chunk_y, chunk_z);
            mazes.push(maze);
        }

        write_mazes_to_html_file(&mazes, HTML_FILE_OUTPUT_PATH).unwrap();
    }

    App::new()
        .register_type::<Speed>()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
            ThirdPersonCameraPlugin,
            WorldInspectorPlugin::new(),
        ))
        .init_state::<ActiveChunk>()
        .init_state::<PendingInteractable>()
        .init_state::<PlayerState>()
        .init_state::<PlayerAnimation>()
        .add_event::<ActiveChunkChange>()
        .add_systems(
            Startup,
            (
                setup_animations,
                spawn_initial_chunks,
                spawn_camera,
                spawn_player,
            ),
        )
        .add_systems(
            Update,
            (
                player_movement,
                update_pending_interactable,
                manage_active_chunk,
                handle_active_chunk_change,
                handle_keyboard_input,
                play_player_animation.before(animate_targets),
                change_player_animation,
            ),
        )
        .run();
}
