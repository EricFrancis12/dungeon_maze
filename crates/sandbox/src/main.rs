use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_inspector_egui::{
    bevy_egui::{egui, EguiContexts},
    quick::WorldInspectorPlugin,
};
use bevy_third_person_camera::*;
use dungeon_maze_common::{
    utils::io::read_dir_to_vec,
    world::{data::WorldData, world_structure::WorldStructure, ChunkMarker},
};
use dungeon_maze_game::plugins::world::bundle::chunk::spawn_chunk_bundle;
use std::{collections::HashMap, env, path::Path};

const MOVEMENT_SPEED: f32 = 4.0;

#[derive(Component)]
struct Player;

#[derive(Clone, Default, Resource)]
struct AssetLib {
    ws_handles: HashMap<String, (Handle<WorldStructure>, bool)>,
}

fn get_assets_dir_path() -> String {
    env::current_dir()
        .unwrap()
        .join(Path::new("assets"))
        .to_str()
        .unwrap()
        .to_owned()
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                watch_for_changes_override: Some(true),
                file_path: get_assets_dir_path(),
                processed_file_path: get_assets_dir_path(),
                ..default()
            }),
            EmbeddedAssetPlugin::default(),
            ThirdPersonCameraPlugin,
            JsonAssetPlugin::<WorldStructure>::new(&["json"]),
            WorldInspectorPlugin::default(),
        ))
        .init_resource::<WorldData>()
        .init_resource::<AssetLib>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
                render_gui,
                handle_assets_modified,
                update_chunks.run_if(resource_changed::<AssetLib>),
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, asset_lib: Res<AssetLib>) {
    // Init assets
    update_assets_lib(&mut commands, &asset_server, &asset_lib);

    // Spawn player
    let player_bundle = (
        SpatialBundle {
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Player,
        ThirdPersonCameraTarget,
    );
    commands.spawn(player_bundle);

    // Spawn camera
    let camera_bundle = (
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ThirdPersonCamera::default(),
    );
    commands.spawn(camera_bundle);

    // Spawn light
    let light_bundle = PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0 * 1000.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 5.0, 0.0),
        ..default()
    };
    commands.spawn(light_bundle);
}

fn player_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<&mut Transform, With<Player>>,
    cam_q: Query<&Transform, (With<Camera3d>, Without<Player>)>,
) {
    for mut player_transform in player_q.iter_mut() {
        let cam = match cam_q.get_single() {
            Ok(c) => c,
            Err(e) => Err(format!("Error retrieving camera: {}", e)).unwrap(),
        };

        let mut direction = Vec3::ZERO;

        // forward
        if keys.pressed(KeyCode::KeyW) {
            direction += *cam.forward();
        }

        // back
        if keys.pressed(KeyCode::KeyS) {
            direction += *cam.back();
        }

        // left
        if keys.pressed(KeyCode::KeyA) {
            direction += *cam.left();
        }

        // right
        if keys.pressed(KeyCode::KeyD) {
            direction += *cam.right();
        }

        direction.y = 0.0;

        // up
        if keys.pressed(KeyCode::ShiftLeft) {
            direction.y += 1.0;
        }

        // down
        if keys.pressed(KeyCode::ControlLeft) {
            direction.y -= 1.0;
        }

        let movement = direction.normalize_or_zero() * MOVEMENT_SPEED * time.delta_seconds();
        player_transform.translation += movement;

        if direction.length_squared() > 0.0 {
            player_transform.look_to(direction, Vec3::Y);
        }
    }
}

fn render_gui(mut commands: Commands, mut contexts: EguiContexts, asset_lib: Res<AssetLib>) {
    let ctx = contexts.ctx_mut();

    egui::SidePanel::right("side_panel")
        .default_width(400.0)
        .show(ctx, |ui| {
            for (path, (handle, active)) in &asset_lib.ws_handles {
                let text = format!("[{}] {}", if *active { "on" } else { "off" }, path);

                if ui.button(text).clicked() {
                    let mut new_asset_lib = asset_lib.clone();
                    new_asset_lib
                        .ws_handles
                        .insert(path.clone(), (handle.clone(), !active));

                    commands.insert_resource(new_asset_lib);
                }
            }
        });
}

fn update_assets_lib(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    asset_lib: &Res<AssetLib>,
) {
    let ws_paths: Vec<String> =
        read_dir_to_vec(&format!("{}/world_structures", get_assets_dir_path()))
            .unwrap()
            .iter()
            .map(|file_name| format!("world_structures/{}", file_name))
            .collect();

    let mut ws_handles: HashMap<String, (Handle<WorldStructure>, bool)> = HashMap::new();

    for path in ws_paths {
        let ws_handle: Handle<WorldStructure> = asset_server.load(&path);

        let active = asset_lib
            .ws_handles
            .get(&path)
            .map(|(_, a)| *a)
            .unwrap_or(false);

        ws_handles.insert(path, (ws_handle, active));
    }

    commands.insert_resource(AssetLib { ws_handles });
}

fn handle_assets_modified(
    mut commands: Commands,
    mut ws_event_reader: EventReader<AssetEvent<WorldStructure>>,
    asset_server: Res<AssetServer>,
    asset_lib: Res<AssetLib>,
) {
    // TODO: fix other programs saving files to the assets dir
    // do not trigger this event:
    for event in ws_event_reader.read() {
        if let AssetEvent::Added { id: _ } | AssetEvent::Modified { id: _ } = event {
            update_assets_lib(&mut commands, &asset_server, &asset_lib);
        }
    }
}

fn update_chunks(
    mut commands: Commands,
    chunk_marker_query: Query<Entity, With<ChunkMarker>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    world_structures: Res<Assets<WorldStructure>>,
    asset_lib: Res<AssetLib>,
    world_data: Res<WorldData>,
) {
    for entity in chunk_marker_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for (_, (handle, active)) in &asset_lib.ws_handles {
        if *active {
            let ws = world_structures.get(handle.id()).unwrap();

            for chunk in &ws.chunks {
                spawn_chunk_bundle(
                    &chunk,
                    &mut commands,
                    &asset_server,
                    &mut meshes,
                    &mut materials,
                    &world_data,
                );
            }
        }
    }
}
