use crate::{
    inventory::item::Item,
    player::attack::{AttackHand, AttackType},
    utils::CyclicCounter,
};
use bevy::{
    asset::Handle,
    prelude::{AnimationGraph, AnimationNodeIndex, Component, Resource, States},
};

#[derive(Resource)]
pub struct AnimationLib {
    pub nodes: Vec<AnimationNodeIndex>,
    pub graph: Handle<AnimationGraph>,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum PlayerAnimation {
    #[default]
    Idle,
    Jogging,
    Running,

    // unarmed attacks
    UnarmedLeftLightAttack,
    UnarmedLeftHeavyAttack,
    UnarmedRightLightAttack,
    UnarmedRightHeavyAttack,

    // one handed weapon attacks
    OneHandedSlashLeftHeavyAttack,
    OneHandedSlashLeftLightAttack,
    OneHandedSlashRightHeavyAttack,
    OneHandedSlashRightLightAttack,
}

impl PlayerAnimation {
    pub fn index(&self) -> usize {
        match self {
            // TODO: fix glb animation ordering when exporting from Blender
            Self::Idle => 0,
            Self::Jogging => 1,
            Self::OneHandedSlashRightLightAttack => 2,
            Self::Running => 3,
            Self::UnarmedLeftHeavyAttack => 4,
            Self::UnarmedLeftLightAttack => 5,
            Self::UnarmedRightHeavyAttack => 6,
            Self::UnarmedRightLightAttack => 7,
            Self::OneHandedSlashLeftHeavyAttack => 7, // TODO
            Self::OneHandedSlashLeftLightAttack => 7, // TODO
            Self::OneHandedSlashRightHeavyAttack => 7, // TODO
        }
    }

    pub fn new_attack_animation(
        attack_type: &AttackType,
        attack_hand: &AttackHand,
        slot: &Option<Item>,
    ) -> Self {
        match slot {
            None => match (attack_type, attack_hand) {
                // unarmed attacks
                (AttackType::Light, AttackHand::Left) => Self::UnarmedLeftLightAttack,
                (AttackType::Light, AttackHand::Right) => Self::UnarmedRightLightAttack,
                (AttackType::Heavy, AttackHand::Left) => Self::UnarmedLeftHeavyAttack,
                (AttackType::Heavy, AttackHand::Right) => Self::UnarmedRightHeavyAttack,
            },
            Some(item) => item.name.player_attack_animation(attack_type, attack_hand),
        }
    }

    pub fn is_attack_animation(&self) -> bool {
        match self {
            Self::UnarmedLeftLightAttack
            | Self::UnarmedLeftHeavyAttack
            | Self::UnarmedRightLightAttack
            | Self::UnarmedRightHeavyAttack
            | Self::OneHandedSlashLeftHeavyAttack
            | Self::OneHandedSlashLeftLightAttack
            | Self::OneHandedSlashRightHeavyAttack
            | Self::OneHandedSlashRightLightAttack => true,
            Self::Idle | Self::Jogging | Self::Running => false,
        }
    }

    pub fn is_matching_attack_animation(
        &self,
        attack_type: &AttackType,
        attack_hand: &AttackHand,
        slot: &Option<Item>,
    ) -> bool {
        match self.is_attack_animation() {
            true => *self == Self::new_attack_animation(attack_type, attack_hand, slot),
            false => false,
        }
    }
}

#[derive(Component)]
pub struct ContinuousAnimation;

#[derive(Component)]
pub struct CyclicAnimation {
    counter: CyclicCounter,
}

impl CyclicAnimation {
    pub fn new(min: u32, max: u32) -> Self {
        Self {
            counter: CyclicCounter::new(min, max),
        }
    }

    fn _value(&self) -> u32 {
        self.counter.value()
    }

    pub fn cycle(&mut self) -> u32 {
        self.counter.cycle()
    }
}
