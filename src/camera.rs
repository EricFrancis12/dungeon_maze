use bevy::prelude::*;
use bevy_third_person_camera::*;

const CAMERA_X: f32 = -2.0;
const CAMERA_Y: f32 = 2.5;
const CAMERA_Z: f32 = 5.0;
const CAMERA_ZOOM_MIN: f32 = 0.1;
const CAMERA_ZOOM_MAX: f32 = 3.0;
const CAMERA_SENSITIVITY: f32 = 2.5;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ThirdPersonCameraPlugin)
            .add_systems(Startup, spawn_camera);
    }
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
        FogSettings {
            color: Color::srgba_u8(42, 41, 41, 255),
            directional_light_exponent: 7.8,
            falloff: FogFalloff::Linear {
                start: 10.0,
                end: 21.0,
            },
            ..default()
        },
        Name::new("Camera"),
    );

    commands.spawn(camera_bundle);
}
