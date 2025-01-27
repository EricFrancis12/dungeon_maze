use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_third_person_camera::*;
use dungeon_maze_common::{
    animation::{ContinuousAnimation, PlayerAnimation},
    camera::MainCamera,
    inventory::{equipment::EquipmentSlotName, item::Item, Inventory, InventoryChanged},
    menu::MenuOpen,
    player::{
        attack::{AttackChargeUp, AttackHand, EntitiesHit},
        DmgImmune, DmgResist, DmgTarget, DmgType, HealHealth, HealModifier, HealStamina, Health,
        Killable, Player, PlayerState, Regenerator, Speed, Stamina, TakeDamage,
    },
    should_not_happen,
    utils::_max,
};
use std::f32::consts::PI;
use strum::IntoEnumIterator;

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
            .insert_resource(AttackChargeUp::new(10, 15, None))
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (
                    spawn_starting_equiped_items,
                    spawn_new_equiped_items,
                    toggle_player_sprinting,
                    player_ground_movement,
                    temp_health_regen,
                    temp_stamina_regen,
                    temp_dmg_resists,
                    temp_heal_health_modifiers,
                    temp_heal_stamina_modifiers,
                    tick_dmg_immune,
                    drain_stamina_while_sprinting.run_if(in_state(PlayerState::Sprinting)),
                    handle_take_damage,
                    handle_heal_health,
                    handle_heal_stamina,
                    despawn_dead_entities,
                    charge_up_and_release_attack.run_if(in_state(MenuOpen(false))),
                    equipment_attack_collisions,
                    reset_entities_hit,
                ),
            )
            .add_systems(OnEnter(PlayerState::Walking), change_player_speed)
            .add_systems(OnEnter(PlayerState::Sprinting), change_player_speed);
    }
}

fn spawn_player(
    mut commands: Commands,
    name_query: Query<
        (Entity, &Name),
        (Without<Player>, Without<EquipmentSlotName>, Without<Item>),
    >,
    asset_server: Res<AssetServer>,
    inventory: Res<Inventory>,
) {
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
                        .from_asset("embedded://models/man.glb"),
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

    // TODO: Refactor to run this logic once player model has been spawned:
    for slot_name in EquipmentSlotName::iter() {
        if let Some(item) = inventory.equipment.at(&slot_name) {
            spawn_equipment_model_bundle(
                &slot_name,
                item,
                &mut commands,
                &name_query,
                &asset_server,
            );
        }
    }
}

fn update_equiped_items(
    mut commands: &mut Commands,
    name_query: &Query<
        (Entity, &Name),
        (Without<Player>, Without<EquipmentSlotName>, Without<Item>),
    >,
    slot_name_query: &Query<(Entity, &EquipmentSlotName, &Item)>,
    asset_server: &Res<AssetServer>,
    inventory: &Res<Inventory>,
) {
    for slot_name in EquipmentSlotName::iter() {
        if let Some(item) = inventory.equipment.at(&slot_name) {
            handle_equipped_item(
                &slot_name,
                item,
                &mut commands,
                &name_query,
                &slot_name_query,
                &asset_server,
            );
        } else {
            handle_unequipped_item(&slot_name, &mut commands, &slot_name_query);
        }
    }
}

fn spawn_starting_equiped_items(
    mut commands: Commands,
    name_query: Query<
        (Entity, &Name),
        (Without<Player>, Without<EquipmentSlotName>, Without<Item>),
    >,
    added_name_query: Query<&Name, Added<Name>>,
    slot_name_query: Query<(Entity, &EquipmentSlotName, &Item)>,
    asset_server: Res<AssetServer>,
    inventory: Res<Inventory>,
) {
    for name in added_name_query.iter() {
        EquipmentSlotName::iter()
            .filter(|sn| sn.matches_target(name))
            .for_each(|_| {
                update_equiped_items(
                    &mut commands,
                    &name_query,
                    &slot_name_query,
                    &asset_server,
                    &inventory,
                );
            });
    }
}

fn spawn_new_equiped_items(
    mut commands: Commands,
    mut event_reader: EventReader<InventoryChanged>,
    name_query: Query<
        (Entity, &Name),
        (Without<Player>, Without<EquipmentSlotName>, Without<Item>),
    >,
    slot_name_query: Query<(Entity, &EquipmentSlotName, &Item)>,
    asset_server: Res<AssetServer>,
    inventory: Res<Inventory>,
) {
    for _ in event_reader.read() {
        update_equiped_items(
            &mut commands,
            &name_query,
            &slot_name_query,
            &asset_server,
            &inventory,
        );
    }
}

fn handle_equipped_item(
    slot_name: &EquipmentSlotName,
    item: &Item,
    commands: &mut Commands,
    name_query: &Query<
        (Entity, &Name),
        (Without<Player>, Without<EquipmentSlotName>, Without<Item>),
    >,
    slot_name_query: &Query<(Entity, &EquipmentSlotName, &Item)>,
    asset_server: &Res<AssetServer>,
) {
    if let Some((entity, _, _item)) = slot_name_query.iter().find(|(_, n, _)| *n == slot_name) {
        if item == _item {
            return;
        }

        // Despawn existing item model
        commands.entity(entity).despawn_recursive();
    }

    // Spawn new item model
    spawn_equipment_model_bundle(slot_name, item, commands, name_query, asset_server);
}

fn handle_unequipped_item(
    slot_name: &EquipmentSlotName,
    commands: &mut Commands,
    slot_name_query: &Query<(Entity, &EquipmentSlotName, &Item)>,
) {
    if let Some((entity, _, _item)) = slot_name_query.iter().find(|(_, n, _)| *n == slot_name) {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_equipment_model_bundle(
    slot_name: &EquipmentSlotName,
    item: &Item,
    commands: &mut Commands,
    name_query: &Query<
        (Entity, &Name),
        (Without<Player>, Without<EquipmentSlotName>, Without<Item>),
    >,
    asset_server: &Res<AssetServer>,
) {
    let target = slot_name.query_target(name_query);

    if let (Some(target_entity), Some(path)) = (target, item.model_path()) {
        // TODO: fix models not being spawned in the correct orientation:
        commands.entity(target_entity).with_children(|parent| {
            parent.spawn((
                slot_name.clone(),
                item.clone(),
                Sensor,
                // TODO: impliment colliders in the shape of the held weapon
                Collider::cuboid(0.1, 0.1, 0.1),
                SceneBundle {
                    scene: asset_server.load(path),
                    ..default()
                },
                Name::new(format!("{} Equipment Model", slot_name)),
            ));
        });
    }
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
    // Using just_pressed() here instead of pressed() because if
    // the player runs out of stamina, they are forced to release
    // ShiftLeft and press it again to resume sprinting.
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
        match *player_state.get() {
            PlayerState::Walking => *player_speed = Speed(PLAYER_WALKING_SPEED),
            PlayerState::Sprinting => *player_speed = Speed(PLAYER_SPRINTING_SPEED),
            PlayerState::Attacking(..) => {}
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

fn temp_heal_health_modifiers(mut health_query: Query<&mut Health>) {
    for mut health in health_query.iter_mut() {
        health.heal_modifier.tick_temp_modifiers();
    }
}

fn temp_heal_stamina_modifiers(mut stamina_query: Query<&mut Stamina>) {
    for mut stamina in stamina_query.iter_mut() {
        stamina.heal_modifier.tick_temp_modifiers();
    }
}

fn tick_dmg_immune(mut dmg_immune_query: Query<&mut DmgImmune>) {
    for mut dmg_immune in dmg_immune_query.iter_mut() {
        dmg_immune.tick();
    }
}

fn drain_stamina_while_sprinting(
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
        Option<&DmgImmune>,
    )>,
) {
    for event in event_reader.read() {
        if let Some((_, mut h, mut s, dr, di)) =
            query.iter_mut().find(|(e, _, _, _, _)| *e == event.1)
        {
            if di.is_some() {
                continue;
            }

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
                            // TODO: have dmg_resist affect a percentage of amt instead subtracting a flat value?
                            health.subtract(amt - &dmg_resist.get_resist(&dmg_type));
                        });
                    }
                    DmgType::Stamina => {
                        s.as_mut().map(|stamina| {
                            // TODO: have dmg_resist affect a percentage of amt instead subtracting a flat value?
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
    mut health_query: Query<(Entity, &mut Health)>,
) {
    for event in event_reader.read() {
        if let Some((_, mut health)) = health_query.iter_mut().find(|(e, _)| *e == event.1) {
            let total_modifier = health.heal_modifier.get_total();
            // TODO: have total_modifier affect a percentage of event.0 instead adding a flat value?
            health.add(event.0 + total_modifier);
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
    mut stamina_query: Query<(Entity, &mut Stamina)>,
) {
    for event in event_reader.read() {
        if let Some((_, mut stamina)) = stamina_query.iter_mut().find(|(e, _)| *e == event.1) {
            let total_modifier = stamina.heal_modifier.get_total();
            // TODO: have total_modifier affect a percentage of event.0 instead adding a flat value?
            stamina.add(event.0 + total_modifier);
        } else {
            should_not_happen!(
                "received HealStamina event on entity that does not exist: {}",
                event.1,
            );
        }
    }
}

fn despawn_dead_entities(
    mut commands: Commands,
    killable_query: Query<(Entity, &Health), With<Killable>>,
) {
    for (entity, health) in killable_query.iter() {
        if health.value <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn charge_up_and_release_attack(
    mut next_player_state: ResMut<NextState<PlayerState>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut attack_charge_up: ResMut<AttackChargeUp>,
) {
    for (mouse_button, attack_hand) in [
        (MouseButton::Left, AttackHand::Left),
        (MouseButton::Right, AttackHand::Right),
    ] {
        if mouse.pressed(mouse_button) {
            if mouse.just_pressed(mouse_button) {
                attack_charge_up.reset_to(attack_hand);
            } else if attack_charge_up.is_charging_hand(&attack_hand) {
                attack_charge_up.tick();
            }
            break;
        }

        if attack_charge_up.is_charging_hand(&attack_hand) {
            let attack_type = attack_charge_up.release();
            next_player_state.set(PlayerState::Attacking(attack_type, attack_hand));
            break;
        }
    }
}

pub fn equipment_attack_collisions(
    mut commands: Commands,
    mut event_writer: EventWriter<TakeDamage>,
    mut item_query: Query<
        (Entity, &EquipmentSlotName, &Item, Option<&mut EntitiesHit>),
        (With<Collider>, Without<Player>),
    >,
    dmg_target_query: Query<
        Entity,
        (
            With<DmgTarget>,
            With<Collider>,
            Without<Player>,
            Without<EntitiesHit>,
            Without<EquipmentSlotName>,
            Without<Item>,
        ),
    >,
    rapier_context: Res<RapierContext>,
    player_state: Res<State<PlayerState>>,
) {
    if let PlayerState::Attacking(attack_type, attack_hand) = *player_state.get() {
        for (item_entity, slot_name, item, mut eh) in item_query.iter_mut() {
            if *slot_name != EquipmentSlotName::from(&attack_hand) {
                continue;
            }

            for entity in dmg_target_query.iter() {
                if rapier_context
                    .intersection_pair(entity, item_entity)
                    .unwrap_or(false)
                {
                    // TODO: account for damage modifiers before dealing damage

                    if let Some(entities_hit) = eh.as_mut() {
                        if entities_hit.0.contains(&entity) {
                            return;
                        }
                        entities_hit.0.push(entity);
                    } else {
                        commands
                            .entity(item_entity)
                            .insert(EntitiesHit::new(vec![entity]));
                    }

                    event_writer.send(TakeDamage(item.calc_dmg(&attack_type), entity));
                }
            }
        }
    }
}

pub fn reset_entities_hit(
    mut commands: Commands,
    mut event_reader: EventReader<StateTransitionEvent<PlayerState>>,
    entities_hit_query: Query<Entity, (With<EntitiesHit>, With<EquipmentSlotName>)>,
) {
    for event in event_reader.read() {
        if let Some(PlayerState::Attacking(..)) = &event.exited {
            for entity in entities_hit_query.iter() {
                commands.entity(entity).remove::<EntitiesHit>();
            }
        }
    }
}
