pub mod attack;

use crate::utils::{IncrCounter, _min_max_or_betw};
use attack::{AttackHand, AttackType};
use bevy::{
    prelude::{Component, Entity, Event, States},
    reflect::Reflect,
};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Component)]
pub struct Player;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum PlayerState {
    #[default]
    Walking,
    Sprinting,
    Attacking(AttackType, AttackHand),
}

impl PlayerState {
    pub fn is_ground_movement(&self) -> bool {
        *self == Self::Walking || *self == Self::Sprinting
    }
}

#[derive(Debug, Event)]
pub struct TakeDamage(pub Vec<(DmgType, f32)>, pub Entity);

#[derive(Event)]
pub struct HealHealth(pub f32, pub Entity);

#[derive(Event)]
pub struct HealStamina(pub f32, pub Entity);

#[derive(Component, Reflect)]
pub struct Speed(pub f32);

// TODO: create derive macro for Regenerator
pub trait Regenerator {
    fn get_base_regen(&mut self) -> f32;
    fn get_static_modifiers(&mut self) -> &mut Vec<f32>;
    fn get_temp_modifiers(&mut self) -> &mut Vec<TempAmt>;
    fn do_regen(&mut self);

    fn _add_static_modifier(&mut self, amt: f32) {
        self.get_static_modifiers().push(amt);
    }

    fn add_temp_modifier(&mut self, amt: f32, durr: u32) {
        self.get_temp_modifiers().push(TempAmt::new(amt, durr));
    }

    fn tick_temp_modifiers(&mut self) {
        self.get_temp_modifiers().retain_mut(|tm| tm.tick() != 0);
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
                &mut self.static_regen_modifiers
            }

            fn get_temp_modifiers(&mut self) -> &mut Vec<TempAmt> {
                &mut self.temp_regen_modifiers
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

pub trait HealModifier {
    fn new() -> Self;
    fn get_base_total(&self) -> f32;
    fn get_static_total(&self) -> f32;
    fn get_temp_total(&self) -> f32;
    fn get_total(&self) -> f32;
    fn tick_temp_modifiers(&mut self);
}

macro_rules! heal_modifier_impl {
    ($t:ty) => {
        impl HealModifier for $t {
            fn new() -> Self {
                Self {
                    base_modifiers: Vec::new(),
                    static_modifiers: Vec::new(),
                    temp_modifiers: Vec::new(),
                }
            }

            fn get_base_total(&self) -> f32 {
                self.base_modifiers.iter().fold(0.0, |acc, curr| acc + curr)
            }

            fn get_static_total(&self) -> f32 {
                self.static_modifiers
                    .iter()
                    .fold(0.0, |acc, curr| acc + curr)
            }

            fn get_temp_total(&self) -> f32 {
                self.temp_modifiers
                    .iter()
                    .fold(0.0, |acc, curr| acc + curr.amt)
            }

            fn get_total(&self) -> f32 {
                self.get_base_total() + self.get_static_total() + self.get_temp_total()
            }

            fn tick_temp_modifiers(&mut self) {
                self.temp_modifiers.retain_mut(|tm| tm.tick() != 0);
            }
        }
    };
}

pub struct HealHealthModifier {
    base_modifiers: Vec<f32>,
    static_modifiers: Vec<f32>,
    temp_modifiers: Vec<TempAmt>,
}

heal_modifier_impl!(HealHealthModifier);

pub struct HealStaminaModifier {
    base_modifiers: Vec<f32>,
    static_modifiers: Vec<f32>,
    temp_modifiers: Vec<TempAmt>,
}

heal_modifier_impl!(HealStaminaModifier);

#[derive(Component)]
pub struct Health {
    pub value: f32,
    pub max_value: f32,
    base_regen: f32,
    static_regen_modifiers: Vec<f32>,
    temp_regen_modifiers: Vec<TempAmt>,
    pub heal_modifier: HealHealthModifier,
}

regenerator_impl!(Health);
modify_value!(Health);

impl Health {
    pub fn new(value: f32, max_value: f32, base_regen: f32) -> Self {
        Self {
            value,
            max_value,
            base_regen,
            static_regen_modifiers: Vec::new(),
            temp_regen_modifiers: Vec::new(),
            heal_modifier: HealHealthModifier::new(),
        }
    }
}

#[derive(Component)]
pub struct Stamina {
    pub value: f32,
    pub max_value: f32,
    base_regen: f32,
    static_regen_modifiers: Vec<f32>,
    temp_regen_modifiers: Vec<TempAmt>,
    pub heal_modifier: HealStaminaModifier,
}

regenerator_impl!(Stamina);
modify_value!(Stamina);

impl Stamina {
    pub fn new(value: f32, max_value: f32, base_regen: f32) -> Self {
        Self {
            value,
            max_value,
            base_regen,
            static_regen_modifiers: Vec::new(),
            temp_regen_modifiers: Vec::new(),
            heal_modifier: HealStaminaModifier::new(),
        }
    }
}

#[derive(Component)]
pub struct DmgTarget;

#[derive(Component)]
pub struct Killable;

#[derive(Clone, Debug, EnumIter, Eq, Hash, PartialEq)]
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
pub struct DmgResist {
    base_resists: HashMap<DmgType, Vec<f32>>,
    static_resists: HashMap<DmgType, Vec<f32>>,
    temp_resists: HashMap<DmgType, Vec<TempAmt>>,
}

impl Default for DmgResist {
    fn default() -> Self {
        Self::new()
    }
}

impl DmgResist {
    pub fn new() -> Self {
        let hm: HashMap<DmgType, Vec<f32>> =
            DmgType::iter().fold(HashMap::new(), |mut acc, curr| {
                acc.insert(curr, Vec::new());
                return acc;
            });

        let temp_resists: HashMap<DmgType, Vec<TempAmt>> =
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

    pub fn _add_static_resist(&mut self, dmg_type: &DmgType, amt: f32) {
        self.static_resists.get_mut(dmg_type).unwrap().push(amt);
    }

    pub fn _add_temp_resist(&mut self, dmg_type: &DmgType, amt: TempAmt) {
        self.temp_resists.get_mut(dmg_type).unwrap().push(amt);
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

    pub fn get_resist(&self, dmg_type: &DmgType) -> f32 {
        self.get_base_resist(dmg_type)
            + self.get_static_resist(dmg_type)
            + self.get_temp_resist(dmg_type)
    }

    pub fn tick_temp_resists(&mut self) {
        self.temp_resists.iter_mut().for_each(|(_, v)| {
            v.retain_mut(|tm| tm.tick() != 0);
        });
    }
}

#[derive(Component)]
pub struct DmgImmune {
    counter: Option<IncrCounter>,
}

impl DmgImmune {
    fn _new(frames: Option<u32>) -> Self {
        Self {
            counter: frames.map(|f| IncrCounter::new(f as i32, -1)),
        }
    }

    pub fn tick(&mut self) {
        self.counter.map(|mut c| c.tick());
    }
}

#[derive(Clone, Copy)]
pub struct TempAmt {
    amt: f32,
    counter: IncrCounter,
}

impl TempAmt {
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
