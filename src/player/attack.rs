use crate::{
    inventory::{equipment::EquipmentSlotName, item::Item},
    player::{DmgTarget, Player, PlayerState, TakeDamage},
    utils::IncrCounter,
};
use bevy::prelude::*;
use bevy_rapier3d::{plugin::RapierContext, prelude::Collider};

#[derive(Component)]
pub struct EntitiesHit(pub Vec<Entity>);

impl EntitiesHit {
    pub fn new(entities: Vec<Entity>) -> Self {
        Self(entities)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Resource)]
pub struct AttackChargeUp {
    light_attack_frames: u32,
    heavy_attack_frames: u32,
    attack_hand: Option<AttackHand>,
    counter: IncrCounter,
}

impl AttackChargeUp {
    pub fn new(
        light_attack_frames: u32,
        heavy_attack_frames: u32,
        attack_hand: Option<AttackHand>,
    ) -> Self {
        Self {
            light_attack_frames,
            heavy_attack_frames,
            attack_hand,
            counter: IncrCounter::new((light_attack_frames + heavy_attack_frames) as i32, -1),
        }
    }

    fn tick(&mut self) -> i32 {
        self.attack_hand.map_or(0, |_| self.counter.tick())
    }

    fn is_charging_hand(&self, attack_hand: &AttackHand) -> bool {
        match self.attack_hand {
            Some(h) => &h == attack_hand,
            None => false,
        }
    }

    fn reset(&mut self) {
        self.attack_hand = None;
        self.counter = IncrCounter::new(
            (self.light_attack_frames + self.heavy_attack_frames) as i32,
            -1,
        )
    }

    fn reset_to(&mut self, attack_hand: AttackHand) {
        self.reset();
        self.attack_hand = Some(attack_hand);
    }

    fn release(&mut self) -> AttackType {
        self.attack_hand.map_or(AttackType::Light, |_| {
            let attack_type = if self.counter.get_value() <= self.heavy_attack_frames as i32 {
                AttackType::Heavy
            } else {
                AttackType::Light
            };

            self.reset();
            attack_type
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum AttackType {
    #[default]
    Light,
    Heavy,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum AttackHand {
    #[default]
    Left,
    Right,
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
