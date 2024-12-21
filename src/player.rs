use crate::{
    animation::{ContinuousAnimation, PlayerAnimation},
    camera::MainCamera,
    should_not_happen,
    utils::{IncrCounter, _max, _min_max_or_betw},
};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_third_person_camera::*;
use std::{collections::HashMap, f32::consts::PI};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const PLAYER_COLLIDER_HX: f32 = 0.4;
const PLAYER_COLLIDER_HY: f32 = 0.85;
const PLAYER_COLLIDER_HZ: f32 = 0.4;

const PLAYER_MAX_HEALTH: f32 = 100.0;
const PLAYER_BASE_HEALTH_REGEN: f32 = 0.1;

const PLAYER_MAX_STAMINA: f32 = 100.0;
const PLAYER_BASE_STAMINA_REGEN: f32 = 1.0;

const PLAYER_WALKING_SPEED: f32 = 200.0;
const PLAYER_SPRINTING_SPEED: f32 = 400.0;

const DEFAULT_PLAYER_GRAVITY_SCALE: f32 = 2.0;
const PLAYER_SPAWN_XYZ: (f32, f32, f32) = (2.0, 1.0, 2.0);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Speed>()
            .add_event::<TakeDamage>()
            .add_event::<HealHealth>()
            .add_event::<HealStamina>()
            .init_state::<PlayerState>()
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (
                    toggle_player_sprinting,
                    player_ground_movement,
                    temp_health_regen,
                    temp_stamina_regen,
                    temp_dmg_resists,
                    player_stamina_while_sprinting,
                    handle_take_damage,
                    handle_heal_health,
                    handle_heal_stamina,
                ),
            )
            .add_systems(OnEnter(PlayerState::Walking), change_player_speed)
            .add_systems(OnEnter(PlayerState::Sprinting), change_player_speed);
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

#[derive(Event)]
pub struct TakeDamage(pub Vec<(DmgType, f32)>, pub Entity);

#[derive(Event)]
pub struct HealHealth(pub f32, pub Entity);

#[derive(Event)]
pub struct HealStamina(pub f32, pub Entity);

#[derive(Component, Reflect)]
pub struct Speed(pub f32);

// TODO: create derive macro for Regenerator
trait Regenerator {
    fn get_base_regen(&mut self) -> f32;

    fn get_static_modifiers(&mut self) -> &mut Vec<f32>;

    fn get_temp_modifiers(&mut self) -> &mut Vec<TimedModifier>;

    fn do_regen(&mut self);

    fn _add_static_modifier(&mut self, amt: f32) {
        self.get_static_modifiers().push(amt);
    }

    fn add_temp_modifier(&mut self, amt: f32, durr: u32) {
        self.get_temp_modifiers()
            .push(TimedModifier::new(amt, durr));
    }

    fn tick_temp_modifiers(&mut self) {
        self.get_temp_modifiers().retain_mut(|m| m.tick() != 0);
    }

    fn get_regen(&mut self) -> f32 {
        let br = self.get_base_regen();
        let sm = self
            .get_static_modifiers()
            .iter()
            .fold(0.0, |acc, curr| acc + curr);
        let tm = self
            .get_temp_modifiers()
            .iter()
            .fold(0.0, |acc, curr| acc + curr.amt);
        br + sm + tm
    }
}

macro_rules! regenerator_impl {
    ($t:ty) => {
        impl Regenerator for $t {
            fn get_base_regen(&mut self) -> f32 {
                self.base_regen
            }

            fn get_static_modifiers(&mut self) -> &mut Vec<f32> {
                &mut self.static_modifiers
            }

            fn get_temp_modifiers(&mut self) -> &mut Vec<TimedModifier> {
                &mut self.temp_modifiers
            }

            fn do_regen(&mut self) {
                self.value = _min_max_or_betw(0.0, self.max_value, self.value + self.get_regen());
            }
        }
    };
}

macro_rules! modify_value {
    ($t:ty) => {
        impl $t {
            pub fn add(&mut self, amt: f32) -> f32 {
                let prev = self.value;
                self.value = _min_max_or_betw(0.0, self.max_value, self.value + amt);
                self.value - prev
            }

            pub fn subtract(&mut self, amt: f32) -> f32 {
                self.add(-amt)
            }
        }
    };
}

#[derive(Component)]
pub struct Health {
    pub value: f32,
    pub max_value: f32,
    base_regen: f32,
    static_modifiers: Vec<f32>,
    temp_modifiers: Vec<TimedModifier>,
}

regenerator_impl!(Health);
modify_value!(Health);

impl Health {
    fn new(value: f32, max_value: f32, base_regen: f32) -> Self {
        Self {
            value,
            max_value,
            base_regen,
            static_modifiers: Vec::new(),
            temp_modifiers: Vec::new(),
        }
    }
}

#[derive(Component)]
pub struct Stamina {
    pub value: f32,
    pub max_value: f32,
    base_regen: f32,
    static_modifiers: Vec<f32>,
    temp_modifiers: Vec<TimedModifier>,
}

regenerator_impl!(Stamina);
modify_value!(Stamina);

impl Stamina {
    fn new(value: f32, max_value: f32, base_regen: f32) -> Self {
        Self {
            value,
            max_value,
            base_regen,
            static_modifiers: Vec::new(),
            temp_modifiers: Vec::new(),
        }
    }
}

#[derive(Clone, EnumIter, Eq, Hash, PartialEq)]
pub enum DmgType {
    Blunt,
    Slash,
    Pierce,
    Fire,
    Ice,
    Poison,
    Stamina,
}

#[derive(Component)]
struct DmgResist {
    base_resists: HashMap<DmgType, Vec<f32>>,
    static_resists: HashMap<DmgType, Vec<f32>>,
    temp_resists: HashMap<DmgType, Vec<TimedModifier>>,
}

impl Default for DmgResist {
    fn default() -> Self {
        Self::new()
    }
}

impl DmgResist {
    fn new() -> Self {
        let hm: HashMap<DmgType, Vec<f32>> =
            DmgType::iter().fold(HashMap::new(), |mut acc, curr| {
                acc.insert(curr, Vec::new());
                return acc;
            });

        let temp_resists: HashMap<DmgType, Vec<TimedModifier>> =
            DmgType::iter().fold(HashMap::new(), |mut acc, curr| {
                acc.insert(curr, Vec::new());
                return acc;
            });

        Self {
            base_resists: hm.clone(),
            static_resists: hm.clone(),
            temp_resists,
        }
    }

    fn get_base_resist(&self, dmg_type: &DmgType) -> f32 {
        self.base_resists[&dmg_type]
            .iter()
            .fold(0.0, |acc, curr| acc + curr)
    }

    fn get_static_resist(&self, dmg_type: &DmgType) -> f32 {
        self.static_resists[&dmg_type]
            .iter()
            .fold(0.0, |acc, curr| acc + curr)
    }

    fn get_temp_resist(&self, dmg_type: &DmgType) -> f32 {
        self.temp_resists[&dmg_type]
            .iter()
            .fold(0.0, |acc, curr| acc + curr.amt)
    }

    fn get_resist(&self, dmg_type: &DmgType) -> f32 {
        self.get_base_resist(dmg_type)
            + self.get_static_resist(dmg_type)
            + self.get_temp_resist(dmg_type)
    }

    fn tick_temp_resists(&mut self) {
        self.temp_resists.iter_mut().for_each(|(_, v)| {
            v.retain_mut(|tm| tm.tick() != 0);
        });
    }
}

#[derive(Component)]
pub struct HealHealthModifier {
    // TODO: ...
}

#[derive(Component)]
pub struct HealStaminaModifier {
    // TODO: ...
}

#[derive(Clone, Copy)]
struct TimedModifier {
    amt: f32,
    counter: IncrCounter,
}

impl TimedModifier {
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
        Health::new(
            PLAYER_MAX_HEALTH,
            PLAYER_MAX_HEALTH,
            PLAYER_BASE_HEALTH_REGEN,
        ),
        Stamina::new(
            PLAYER_MAX_STAMINA,
            PLAYER_MAX_STAMINA,
            PLAYER_BASE_STAMINA_REGEN,
        ),
        DmgResist::new(),
        Speed(PLAYER_WALKING_SPEED),
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
    camera_query: Query<&Transform, (With<MainCamera>, Without<Player>)>,
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
    player_query: Query<&Stamina, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    player_state: Res<State<PlayerState>>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
) {
    // using just_pressed() here instead of pressed() because if
    // the player runs out of stamina, they are forced to release
    // ShiftLeft and press it again to resume sprinting
    if keys.just_pressed(KeyCode::ShiftLeft) && *player_state.get() == PlayerState::Walking {
        let player_stamina = player_query.get_single().unwrap();
        if player_stamina.value > player_stamina.max_value * 0.1 {
            next_player_state.set(PlayerState::Sprinting);
        }
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

fn temp_health_regen(mut health_query: Query<&mut Health>) {
    for mut health in health_query.iter_mut() {
        health.tick_temp_modifiers();
        health.do_regen();
    }
}

fn temp_stamina_regen(mut stamina_query: Query<&mut Stamina>) {
    for mut stamina in stamina_query.iter_mut() {
        stamina.tick_temp_modifiers();
        stamina.do_regen();
    }
}

fn temp_dmg_resists(mut dmg_resists_query: Query<&mut DmgResist>) {
    for mut dmg_resist in dmg_resists_query.iter_mut() {
        dmg_resist.tick_temp_resists();
    }
}

fn player_stamina_while_sprinting(
    mut player_query: Query<&mut Stamina, With<Player>>,
    player_state: Res<State<PlayerState>>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
) {
    if *player_state.get() != PlayerState::Sprinting {
        return;
    }

    let mut player_stamina = player_query.get_single_mut().unwrap();

    if player_stamina.value > 0.0 {
        player_stamina.value = _max(player_stamina.value - 1.0, 0.0);
        let regen = -player_stamina.get_regen();
        player_stamina.add_temp_modifier(regen, 1);
    } else {
        next_player_state.set(PlayerState::Walking);
        player_stamina.add_temp_modifier(-10_000.0, 90);
    }
}

fn handle_take_damage(
    mut event_reader: EventReader<TakeDamage>,
    mut query: Query<(
        Entity,
        Option<&mut Health>,
        Option<&mut Stamina>,
        Option<&DmgResist>,
    )>,
) {
    for event in event_reader.read() {
        if let Some((_, mut h, mut s, dr)) = query.iter_mut().find(|(e, _, _, _)| *e == event.1) {
            let dmg_resist = match dr {
                Some(d) => d,
                None => &DmgResist::new(),
            };

            for (dmg_type, amt) in &event.0 {
                match dmg_type {
                    DmgType::Blunt
                    | DmgType::Slash
                    | DmgType::Pierce
                    | DmgType::Fire
                    | DmgType::Ice
                    | DmgType::Poison => {
                        h.as_mut().map(|health| {
                            health.subtract(amt - &dmg_resist.get_resist(&dmg_type));
                        });
                    }
                    DmgType::Stamina => {
                        s.as_mut().map(|stamina| {
                            stamina.subtract(*amt - &dmg_resist.get_resist(&dmg_type));
                        });
                    }
                }
            }
        } else {
            should_not_happen!(
                "received TakeDamage event on entity that does not exist: {}",
                event.1,
            );
        }
    }
}

fn handle_heal_health(
    mut event_reader: EventReader<HealHealth>,
    mut health_query: Query<(Entity, &mut Health, Option<&HealHealthModifier>)>,
) {
    for event in event_reader.read() {
        if let Some((_, mut health, h)) = health_query.iter_mut().find(|(e, _, _)| *e == event.1) {
            health.add(event.0);
        } else {
            should_not_happen!(
                "received HealHealth event on entity that does not exist: {}",
                event.1,
            );
        }
    }
}

fn handle_heal_stamina(
    mut event_reader: EventReader<HealStamina>,
    mut stamina_query: Query<(Entity, &mut Stamina, Option<&HealStaminaModifier>)>,
) {
    for event in event_reader.read() {
        if let Some((_, mut stamina, s)) = stamina_query.iter_mut().find(|(e, _, _)| *e == event.1)
        {
            stamina.add(event.0);
        } else {
            should_not_happen!(
                "received HealStamina event on entity that does not exist: {}",
                event.1,
            );
        }
    }
}
