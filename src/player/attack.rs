use bevy::prelude::*;

use crate::{
    inventory::{EquipmentSlotName, Inventory},
    utils::IncrCounter,
};

use super::PlayerState;

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
    fn to_equipment_slot_name(&self) -> EquipmentSlotName {
        match self {
            Self::Left => EquipmentSlotName::LeftHand,
            Self::Right => EquipmentSlotName::RightHand,
        }
    }

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

pub fn start_player_attack(
    mouse: Res<ButtonInput<MouseButton>>,
    player_state: Res<State<PlayerState>>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
    inventory: Res<Inventory>,
) {
    if *player_state.get() == PlayerState::Walking {
        // TODO: Handle AttackType::Heavy when user "charges up" attack
        // by holding down left or right mouse button
        let attack_type = AttackType::Light;

        let num_frames = |attack_hand: &AttackHand| match inventory
            .equipment
            .at(&attack_hand.to_equipment_slot_name())
        {
            Some(item) => item.animation_frames(&attack_type, &attack_hand) as i32,
            None => attack_hand.animation_frames(&attack_type) as i32, // no item in slot means unarmed attack
        };

        if mouse.just_pressed(MouseButton::Left) {
            let attack_hand = AttackHand::Left;
            next_player_state.set(PlayerState::Attacking(
                attack_type,
                attack_hand,
                IncrCounter::new(num_frames(&attack_hand), -1),
            ));
        } else if mouse.just_pressed(MouseButton::Right) {
            let attack_hand = AttackHand::Right;
            next_player_state.set(PlayerState::Attacking(
                attack_type,
                attack_hand,
                IncrCounter::new(num_frames(&attack_hand), -1),
            ));
        }
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
