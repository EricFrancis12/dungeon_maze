use crate::player::Player;

use bevy::prelude::*;
use bevy_rapier3d::{
    plugin::{PhysicsSet, RapierContext},
    prelude::QueryFilter,
};
use bevy_third_person_camera::*;

const CAMERA_ZOOM_MIN: f32 = 0.1;
const CAMERA_ZOOM_MAX: f32 = 3.0;
const CAMERA_SENSITIVITY: f32 = 2.5;

const CAMERA_MARGIN: f32 = 0.3;
const CAMERA_RAY_EXTENSION: f32 = 1.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ThirdPersonCameraPlugin)
            .add_systems(Startup, (spawn_main_camera, spawn_alt_camera))
            .add_systems(Update, switch_cameras)
            .configure_sets(PostUpdate, CameraSyncSet.after(PhysicsSet::StepSimulation));
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
struct AltCamera;

fn spawn_main_camera(mut commands: Commands) {
    let main_camera_bundle = (
        MainCamera,
        Camera3dBundle::default(),
        ThirdPersonCamera {
            zoom: Zoom::new(CAMERA_ZOOM_MIN, CAMERA_ZOOM_MAX),
            sensitivity: Vec2 {
                x: CAMERA_SENSITIVITY,
                y: CAMERA_SENSITIVITY,
            },
            ..default()
        },
        Name::new("Main Camera"),
    );

    commands.spawn(main_camera_bundle);
}

fn spawn_alt_camera(mut commands: Commands) {
    let alt_camera_bundle = (
        AltCamera,
        Camera3dBundle {
            camera: Camera {
                is_active: false,
                ..default()
            },
            ..default()
        },
        Name::new("Alt Camera"),
    );

    commands.spawn(alt_camera_bundle);
}

fn switch_cameras(
    mut main_camera_query: Query<(&mut Camera, &GlobalTransform), With<MainCamera>>,
    mut alt_camera_query: Query<
        (&mut Camera, &mut Transform),
        (With<AltCamera>, Without<MainCamera>),
    >,
    player_query: Query<(Entity, &GlobalTransform), With<Player>>,
    rapier_context: Res<RapierContext>,
) {
    let (mut main_camera, main_camera_gl_transform) = main_camera_query.get_single_mut().unwrap();
    let (mut alt_camera, mut alt_camera_transform) = alt_camera_query.get_single_mut().unwrap();
    let (player_entity, player_gl_transform) = player_query.get_single().unwrap();

    let main_camera_translation = main_camera_gl_transform.translation();
    let player_translation = player_gl_transform.translation();

    let ray_dir = (main_camera_translation - player_translation).normalize();
    let distance = main_camera_translation.distance(player_translation) + CAMERA_RAY_EXTENSION;

    if let Some((_, intersection)) = rapier_context.cast_ray_and_get_normal(
        player_translation,
        ray_dir,
        distance,
        true,
        QueryFilter::default().exclude_collider(player_entity),
    ) {
        let direction = intersection.point + intersection.normal;
        let projected_hit_point = intersection.point.move_towards(direction, CAMERA_MARGIN);

        alt_camera_transform.translation = projected_hit_point;
        alt_camera_transform.look_at(player_translation, Vec3::Y);

        activate_camera(&mut alt_camera);
        deactivate_camera(&mut main_camera);

        return;
    }

    activate_camera(&mut main_camera);
    deactivate_camera(&mut alt_camera);
}

fn activate_camera(camera: &mut Camera) {
    camera.is_active = true;
    camera.order = 1;
}

fn deactivate_camera(camera: &mut Camera) {
    camera.is_active = false;
    camera.order = 0;
}
