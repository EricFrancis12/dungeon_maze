use crate::animation::PlayerAnimation;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_third_person_camera::*;
use std::f32::consts::PI;

const PLAYER_COLLIDER_HX: f32 = 0.4;
const PLAYER_COLLIDER_HY: f32 = 0.85;
const PLAYER_COLLIDER_HZ: f32 = 0.4;
const DEFAULT_PLAYER_SPEED: f32 = 450.0;
const DEFAULT_PLAYER_GRAVITY_SCALE: f32 = 70.0;
const PLAYER_SPAWN_XYZ: (f32, f32, f32) = (2.0, 1.0, 2.0);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Speed>()
            .init_state::<PlayerState>()
            .add_systems(Startup, spawn_player)
            .add_systems(Update, player_walking_movement);
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
enum PlayerState {
    #[default]
    Walking,
}

#[derive(Component, Reflect)]
pub struct Speed(f32);

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_bundle = (
        Player,
        Speed(DEFAULT_PLAYER_SPEED),
        RigidBody::Dynamic,
        Velocity::default(),
        ExternalImpulse::default(),
        GravityScale(DEFAULT_PLAYER_GRAVITY_SCALE),
        Collider::cuboid(PLAYER_COLLIDER_HX, PLAYER_COLLIDER_HY, PLAYER_COLLIDER_HZ),
        LockedAxes::ROTATION_LOCKED_X
            | LockedAxes::ROTATION_LOCKED_Y
            | LockedAxes::ROTATION_LOCKED_Z,
        SpatialBundle {
            transform: Transform::from_xyz(
                PLAYER_SPAWN_XYZ.0,
                PLAYER_SPAWN_XYZ.1,
                PLAYER_SPAWN_XYZ.2,
            ),
            ..default()
        },
        ThirdPersonCameraTarget,
        Name::new("Player"),
    );

    commands.spawn(player_bundle).with_children(|parent| {
        parent.spawn((
            SceneBundle {
                scene: asset_server.load(
                    GltfAssetLabel::Scene(PlayerAnimation::Idle.index())
                        .from_asset("models/Man.glb"),
                ),
                transform: Transform::from_xyz(0.0, -PLAYER_COLLIDER_HY, 0.0),
                ..default()
            },
            Name::new("Player Model"),
        ));

        parent.spawn((
            SpotLightBundle {
                transform: Transform::from_xyz(0.0, 0.0, 0.5)
                    .with_rotation(Quat::from_rotation_y(PI)),
                ..default()
            },
            Name::new("Spotlight"),
        ));
    });
}

fn player_walking_movement(
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    mut player_query: Query<
        (&mut Transform, &mut Velocity, &mut ExternalImpulse, &Speed),
        With<Player>,
    >,
    player_state: Res<State<PlayerState>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if *player_state.get() != PlayerState::Walking {
        return;
    }

    for (mut player_transform, mut player_velocity, mut player_external_impulse, player_speed) in
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

        let movement = direction.normalize_or_zero() * player_speed.0 * time.delta_seconds();

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

        player_velocity.linvel = Vec3::ZERO; // Reset player velocity
        player_external_impulse.impulse = movement;
    }
}
