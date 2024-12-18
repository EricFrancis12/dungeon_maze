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
            .add_systems(Startup, (spawn_main_camera, spawn_ray_collider_camera))
            .add_systems(Update, manage_cameras)
            .configure_sets(PostUpdate, CameraSyncSet.after(PhysicsSet::StepSimulation));
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
struct RayColliderCamera;

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
        Name::new("Camera"),
    );

    commands.spawn(main_camera_bundle);
}

fn spawn_ray_collider_camera(mut commands: Commands) {
    let ray_collider_camera_bundle = (
        RayColliderCamera,
        Camera3dBundle {
            camera: Camera {
                is_active: false,
                ..default()
            },
            ..default()
        },
        Name::new("Ray Collider Camera"),
    );

    commands.spawn(ray_collider_camera_bundle);
}

fn manage_cameras(
    mut camera_query: Query<(&mut Camera, &GlobalTransform), With<MainCamera>>,
    mut camera_tracer_query: Query<
        (&mut Camera, &mut Transform),
        (
            With<RayColliderCamera>,
            Without<MainCamera>,
            Without<Player>,
        ),
    >,
    player_query: Query<(Entity, &GlobalTransform), With<Player>>,
    rapier_context: Res<RapierContext>,
) {
    let (mut camera, camera_gl_transform) = camera_query.get_single_mut().unwrap();
    let (mut tracer_camera, mut tracer_camera_transform) =
        camera_tracer_query.get_single_mut().unwrap();
    let (player_entity, player_gl_transform) = player_query.get_single().unwrap();

    let camera_translation = camera_gl_transform.translation();
    let player_translation = player_gl_transform.translation();

    let ray_pos = player_translation;
    let ray_dir = (camera_translation - player_translation).normalize();
    let distance = camera_translation.distance(player_translation) + CAMERA_RAY_EXTENSION;
    let solid = true;
    let filter = QueryFilter::default().exclude_collider(player_entity);

    if let Some((_, intersection)) =
        rapier_context.cast_ray_and_get_normal(ray_pos, ray_dir, distance, solid, filter)
    {
        let direction = intersection.point + intersection.normal;
        let projected_hit_point = intersection.point.move_towards(direction, CAMERA_MARGIN);

        camera.is_active = false;
        tracer_camera.is_active = true;

        tracer_camera_transform.translation = projected_hit_point; // TODO: move camera slightly toward player to avoid the camera being inside of walls
        tracer_camera_transform.look_at(player_translation, Vec3::Y);

        return;
    }

    camera.is_active = true;
    tracer_camera.is_active = false;
}
