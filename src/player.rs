use crate::{
    animation::{ContinuousAnimation, PlayerAnimation},
    utils::{IncrCounter, _max, _min_max_or_betw},
};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_third_person_camera::*;
use std::f32::consts::PI;

const PLAYER_COLLIDER_HX: f32 = 0.4;
const PLAYER_COLLIDER_HY: f32 = 0.85;
const PLAYER_COLLIDER_HZ: f32 = 0.4;

const PLAYER_WALKING_SPEED: f32 = 200.0;
const PLAYER_SPRINTING_SPEED: f32 = 400.0;

const PLAYER_MAX_STAMINA: f32 = 100.0;
const PLAYER_STAMINA_REGEN: f32 = 1.0;

const DEFAULT_PLAYER_GRAVITY_SCALE: f32 = 2.0;
const PLAYER_SPAWN_XYZ: (f32, f32, f32) = (2.0, 1.0, 2.0);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Speed>()
            .init_state::<PlayerState>()
            .add_systems(Startup, spawn_player)
            .add_systems(Update, toggle_player_sprinting)
            .add_systems(OnEnter(PlayerState::Walking), change_player_speed)
            .add_systems(OnEnter(PlayerState::Sprinting), change_player_speed)
            .add_systems(Update, player_ground_movement)
            .add_systems(Update, manage_player_stamina);
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum PlayerState {
    #[default]
    Walking,
    Sprinting,
}

impl PlayerState {
    fn is_ground_movement(&self) -> bool {
        *self == Self::Walking || *self == Self::Sprinting
    }
}

#[derive(Component, Reflect)]
pub struct Speed(pub f32);

#[derive(Component)]
struct Stamina {
    value: f32,
    max_value: f32,
    base_regen: f32,
    regen_modifiers: Vec<RegenModifier>,
}

impl Stamina {
    fn new(value: f32, max_value: f32, base_regen: f32) -> Self {
        Self {
            value,
            max_value,
            base_regen,
            regen_modifiers: Vec::new(),
        }
    }

    fn add_regen_modifier(&mut self, amt: f32, durr: u32) {
        self.regen_modifiers.push(RegenModifier::new(amt, durr));
    }

    fn tick_all_regen_modifiers(&mut self) {
        self.regen_modifiers.retain_mut(|m| m.tick() != 0);
    }

    fn regen(&mut self) -> f32 {
        self.regen_modifiers
            .iter()
            .fold(self.base_regen, |acc, curr| acc * curr.amt)
    }
}

#[derive(Clone, Copy)]
struct RegenModifier {
    amt: f32,
    counter: IncrCounter,
}

impl RegenModifier {
    fn new(amt: f32, durr: u32) -> Self {
        Self {
            amt,
            counter: IncrCounter::new(durr as i32, -1),
        }
    }

    fn tick(&mut self) -> i32 {
        self.counter.tick()
    }
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_bundle = (
        Player,
        Speed(PLAYER_WALKING_SPEED),
        Stamina::new(PLAYER_MAX_STAMINA, PLAYER_MAX_STAMINA, PLAYER_STAMINA_REGEN),
        RigidBody::Dynamic,
        Velocity::default(),
        GravityScale(DEFAULT_PLAYER_GRAVITY_SCALE),
        Collider::cuboid(PLAYER_COLLIDER_HX, PLAYER_COLLIDER_HY, PLAYER_COLLIDER_HZ),
        KinematicCharacterController {
            autostep: Some(CharacterAutostep {
                max_height: CharacterLength::Absolute(1.0),
                min_width: CharacterLength::Absolute(0.1),
                include_dynamic_bodies: false,
            }),
            ..default()
        },
        SpatialBundle {
            transform: Transform::from_xyz(
                PLAYER_SPAWN_XYZ.0,
                PLAYER_SPAWN_XYZ.1,
                PLAYER_SPAWN_XYZ.2,
            ),
            ..default()
        },
        ContinuousAnimation,
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

fn player_ground_movement(
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &Speed), With<Player>>,
    player_state: Res<State<PlayerState>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if !player_state.get().is_ground_movement() {
        return;
    }

    for (mut player_transform, mut player_velocity, player_speed) in player_query.iter_mut() {
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

        if player_velocity.linvel.y > 0.0 {
            player_velocity.linvel.y = 0.0;
        }

        player_velocity.angvel = Vec3::ZERO;
        player_velocity.linvel.x = movement.x;
        player_velocity.linvel.z = movement.z;
    }
}

fn toggle_player_sprinting(
    keys: Res<ButtonInput<KeyCode>>,
    player_state: Res<State<PlayerState>>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
) {
    // using just_pressed() here instead of pressed() because if
    // the player runs out of stamina, they are forced to release
    // ShiftLeft and press it again to resume sprinting
    if keys.just_pressed(KeyCode::ShiftLeft) && *player_state.get() == PlayerState::Walking {
        next_player_state.set(PlayerState::Sprinting);
    } else if !keys.pressed(KeyCode::ShiftLeft) && *player_state.get() == PlayerState::Sprinting {
        next_player_state.set(PlayerState::Walking);
    }
}

fn change_player_speed(
    mut player_query: Query<&mut Speed, With<Player>>,
    player_state: Res<State<PlayerState>>,
) {
    if let Ok(mut player_speed) = player_query.get_single_mut() {
        *player_speed = match *player_state.get() {
            PlayerState::Walking => Speed(PLAYER_WALKING_SPEED),
            PlayerState::Sprinting => Speed(PLAYER_SPRINTING_SPEED),
        };
    }
}

fn manage_player_stamina(
    mut player_query: Query<&mut Stamina, With<Player>>,
    player_state: Res<State<PlayerState>>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
) {
    let mut player_stamina = player_query.get_single_mut().unwrap();
    player_stamina.tick_all_regen_modifiers();

    if *player_state.get() == PlayerState::Sprinting {
        if player_stamina.value > 0.0 {
            player_stamina.value = _max(player_stamina.value - 1.0, 0.0);
        } else {
            next_player_state.set(PlayerState::Walking);
            player_stamina.add_regen_modifier(0.0, 30);
        }
    } else if player_stamina.value < player_stamina.max_value {
        player_stamina.value = _min_max_or_betw(
            0.0,
            player_stamina.max_value,
            player_stamina.value + player_stamina.regen(),
        );
    }
}
