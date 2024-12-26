use bevy::prelude::*;

use crate::utils::IncrCounter;

use super::PlayerState;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum AttackType {
    #[default]
    Normal,
    Heavy,
}

pub fn start_player_attack(
    mouse: Res<ButtonInput<MouseButton>>,
    player_state: Res<State<PlayerState>>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
) {
    if *player_state.get() != PlayerState::Walking {
        return;
    }

    if mouse.just_pressed(MouseButton::Left) {
        next_player_state.set(PlayerState::Attacking(
            AttackType::Normal,
            IncrCounter::new(10, -1),
        ));
    }
}

pub fn tick_player_attack(
    player_state: Res<State<PlayerState>>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
) {
    match player_state.get() {
        PlayerState::Attacking(attack_type, mut counter) => {
            let i = counter.tick();
            println!("attacking: {}", i);
            if i == 0 {
                next_player_state.set(PlayerState::Walking);
            } else {
                next_player_state.set(PlayerState::Attacking(*attack_type, counter));
            }
        }
        _ => {}
    }
}
