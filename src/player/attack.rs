use bevy::prelude::*;

use crate::{
    inventory::{EquipmentSlotName, Inventory},
    utils::IncrCounter,
};

use super::PlayerState;

#[derive(Event)]
pub struct PlayerAttacked(AttackType, AttackHand);

impl PlayerAttacked {
    fn to_tuple(&self) -> (AttackType, AttackHand) {
        (self.0, self.1)
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

impl AttackHand {
    // Returns the number of animation frames an unarmed attack takes
    fn animation_frames(self, attack_type: &AttackType) -> u32 {
        match (self, attack_type) {
            // TODO: measure animation frames:
            (Self::Left, &AttackType::Light) => 60,
            (Self::Left, &AttackType::Heavy) => 60,
            (Self::Right, &AttackType::Light) => 60,
            (Self::Right, &AttackType::Heavy) => 60,
        }
    }
}

impl Into<EquipmentSlotName> for AttackHand {
    fn into(self) -> EquipmentSlotName {
        match self {
            Self::Left => EquipmentSlotName::LeftHand,
            Self::Right => EquipmentSlotName::RightHand,
        }
    }
}

pub fn charge_up_and_release_attack(
    mut event_writer: EventWriter<PlayerAttacked>,
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
            event_writer.send(PlayerAttacked(attack_type, attack_hand));
            break;
        }
    }
}

pub fn start_player_attack(
    mut event_reader: EventReader<PlayerAttacked>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
    inventory: Res<Inventory>,
) {
    for (attack_type, attack_hand) in event_reader.read().map(|e| e.to_tuple()) {
        let frames = inventory
            .equipment
            .at(&attack_hand.into())
            .map_or(attack_hand.animation_frames(&attack_type), |item| {
                item.animation_frames(&attack_type, &attack_hand)
            }) as i32;

        next_player_state.set(PlayerState::Attacking(
            attack_type,
            attack_hand,
            IncrCounter::new(frames, -1),
        ));
    }
}

pub fn tick_player_attack(
    player_state: Res<State<PlayerState>>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
) {
    match player_state.get() {
        PlayerState::Attacking(attack_type, attack_hand, mut counter) => {
            let i = counter.tick();
            if i == 0 {
                next_player_state.set(PlayerState::Walking);
            } else {
                next_player_state.set(PlayerState::Attacking(*attack_type, *attack_hand, counter));
            }
        }
        _ => {}
    }
}
