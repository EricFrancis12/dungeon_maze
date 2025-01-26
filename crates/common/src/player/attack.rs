use crate::utils::IncrCounter;
use bevy::prelude::{Component, Entity, Resource};

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

    pub fn tick(&mut self) -> i32 {
        self.attack_hand.map_or(0, |_| self.counter.tick())
    }

    pub fn is_charging_hand(&self, attack_hand: &AttackHand) -> bool {
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

    pub fn reset_to(&mut self, attack_hand: AttackHand) {
        self.reset();
        self.attack_hand = Some(attack_hand);
    }

    pub fn release(&mut self) -> AttackType {
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
