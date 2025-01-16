use crate::{
    animation::PlayerAnimation,
    interaction::Interactable,
    inventory::equipment::EquipmentSlotName,
    player::{
        attack::{AttackHand, AttackType},
        DmgType,
    },
    should_not_happen,
};
use bevy::{asset::AssetPath, prelude::*};
use rand::{rngs::StdRng, Rng};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

const ITEM_INTERACTABLE_RANGE: f32 = 1.8;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ItemType {
    Consumable,
    RawMaterial,
    Weapon,
}

#[derive(
    Clone, Component, Copy, Debug, Deserialize, Display, EnumIter, Eq, PartialEq, Serialize,
)]
pub enum ItemName {
    // Raw materials
    Coal,
    Cotton,
    Flint,

    // Consumables
    HealthPotion,
    StaminaPotion,
    HealthRegenPotion,
    StaminaRegenPotion,
    HealthPoison,
    StaminaPoison,
    HealthRegenPoison,
    StaminaRegenPoison,

    // Weapons
    Broadsword,
    Katana,
}

impl ItemName {
    pub fn choose(rng: &mut StdRng) -> Self {
        let item_names = Self::iter().collect::<Vec<ItemName>>();
        let i = rng.gen_range(0..item_names.len());
        item_names[i].to_owned()
    }

    pub fn item_type(&self) -> ItemType {
        match self {
            Self::Coal | Self::Cotton | Self::Flint => ItemType::RawMaterial,
            Self::HealthPotion
            | Self::StaminaPotion
            | Self::HealthRegenPotion
            | Self::StaminaRegenPotion
            | Self::HealthPoison
            | Self::StaminaPoison
            | Self::HealthRegenPoison
            | Self::StaminaRegenPoison => ItemType::Consumable,
            Self::Broadsword | Self::Katana => ItemType::Weapon,
        }
    }

    pub fn max_amt(&self) -> u16 {
        match self.item_type() {
            ItemType::Consumable | ItemType::RawMaterial => 64,
            ItemType::Weapon => 1,
        }
    }

    pub fn is_equipable_at(&self, _: &EquipmentSlotName) -> bool {
        match self.item_type() {
            ItemType::Weapon => true,
            ItemType::Consumable | ItemType::RawMaterial => false,
        }
    }

    pub fn base_dmg(&self) -> Vec<(DmgType, f32)> {
        match self {
            Self::Broadsword => vec![(DmgType::Slash, 30.0)],
            Self::Katana => vec![(DmgType::Slash, 40.0)],
            _ => {
                should_not_happen!("ItemName {} does not deal damage", self);
                Vec::new()
            }
        }
    }

    pub fn calc_dmg(&self, _: &AttackType) -> Vec<(DmgType, f32)> {
        // TODO: ...
        self.base_dmg()
    }

    pub fn ui_image(&self, asset_server: &Res<AssetServer>) -> UiImage {
        UiImage {
            texture: match self {
                Self::Coal => asset_server.load("embedded://images/coal.png"),
                Self::Cotton => asset_server.load("embedded://images/cotton.png"),
                Self::Flint => asset_server.load("embedded://images/flint.png"),
                Self::HealthPotion => asset_server.load("embedded://images/health_potion.png"),
                Self::StaminaPotion => asset_server.load("embedded://images/stamina_potion.png"),
                Self::HealthRegenPotion => {
                    asset_server.load("embedded://images/health_regen_potion.png")
                }
                Self::StaminaRegenPotion => {
                    asset_server.load("embedded://images/stamina_regen_potion.png")
                }
                Self::HealthPoison => asset_server.load("embedded://images/health_poison.png"),
                Self::StaminaPoison => asset_server.load("embedded://images/stamina_poison.png"),
                Self::HealthRegenPoison => {
                    asset_server.load("embedded://images/health_regen_poison.png")
                }
                Self::StaminaRegenPoison => {
                    asset_server.load("embedded://images/stamina_regen_poison.png")
                }
                Self::Broadsword => asset_server.load("embedded://images/broadsword.png"),
                Self::Katana => asset_server.load("embedded://images/katana.png"),
            },
            ..default()
        }
    }

    pub fn model_path(&self) -> Option<impl Into<AssetPath>> {
        match self {
            Self::Broadsword => {
                Some(GltfAssetLabel::Scene(0).from_asset("embedded://models/broadsword.glb"))
            }
            Self::Katana => {
                Some(GltfAssetLabel::Scene(0).from_asset("embedded://models/katana.glb"))
            }
            _ => {
                should_not_happen!("expected ItemName with a 3d model, but got: {}", self);
                None
            }
        }
    }

    pub fn player_attack_animation(
        &self,
        attack_type: &AttackType,
        attack_hand: &AttackHand,
    ) -> PlayerAnimation {
        match self {
            Self::Broadsword | &Self::Katana => match (attack_type, attack_hand) {
                (AttackType::Light, AttackHand::Left) => {
                    PlayerAnimation::OneHandedSlashLeftLightAttack
                }
                (AttackType::Light, AttackHand::Right) => {
                    PlayerAnimation::OneHandedSlashRightLightAttack
                }
                (AttackType::Heavy, AttackHand::Left) => {
                    PlayerAnimation::OneHandedSlashLeftHeavyAttack
                }
                (AttackType::Heavy, AttackHand::Right) => {
                    PlayerAnimation::OneHandedSlashRightHeavyAttack
                }
            },
            Self::Coal
            | Self::Cotton
            | Self::Flint
            | Self::HealthPotion
            | Self::StaminaPotion
            | Self::HealthRegenPotion
            | Self::StaminaRegenPotion
            | Self::HealthPoison
            | Self::StaminaPoison
            | Self::HealthRegenPoison
            | Self::StaminaRegenPoison => {
                should_not_happen!(
                    "{:?} is not a weapon, and therefore does not have an attack animation",
                    self
                );
                PlayerAnimation::default()
            }
        }
    }
}

#[derive(Clone, Copy, Component, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Item {
    pub name: ItemName,
    pub amt: u16,
}

impl Item {
    pub fn new(name: ItemName, amt: u16) -> Self {
        Self { name, amt }
    }

    pub fn clone_with_amt(&self, amt: u16) -> Self {
        Self::new(self.name.clone(), amt)
    }

    pub fn choose(rng: &mut StdRng, amt: u16) -> Self {
        Self {
            name: ItemName::choose(rng),
            amt,
        }
    }

    pub fn max_amt(&self) -> u16 {
        self.name.max_amt()
    }

    pub fn merge(&mut self, item: Item) -> Option<Self> {
        if self.name != item.name {
            should_not_happen!("attempting to merge 2 items with different ItemNames");
            return Some(item);
        }

        let total = self.amt.wrapping_add(item.amt);
        let ma = self.max_amt();

        if total > ma {
            self.amt = ma;
            return Some(self.clone_with_amt(total.wrapping_sub(ma)));
        }

        self.amt = total;
        None
    }

    pub fn interactable() -> Interactable {
        Interactable {
            range: ITEM_INTERACTABLE_RANGE,
        }
    }

    pub fn ui_image(&self, asset_server: &Res<AssetServer>) -> UiImage {
        self.name.ui_image(asset_server)
    }

    pub fn model_path(&self) -> Option<impl Into<AssetPath>> {
        self.name.model_path()
    }

    // _use returns a tuple with 2 values:
    // The first value is an optional Item, which represents the byproduct (output) of the original item being used.
    // The second value is a bool that indicates whether or not the original item was mutated.
    pub fn _use(&mut self) -> (Option<Self>, bool) {
        match self.name.item_type() {
            ItemType::Consumable => {
                if self.amt == 0 {
                    return (None, false);
                }
                self.amt -= 1;
                return (Some(self.clone_with_amt(1)), true);
            }
            _ => (None, false),
        }
    }

    pub fn is_equipable_at(&self, name: &EquipmentSlotName) -> bool {
        self.name.is_equipable_at(name)
    }

    fn _base_dmg(&self) -> Vec<(DmgType, f32)> {
        self.name.base_dmg()
    }

    pub fn calc_dmg(&self, attack_type: &AttackType) -> Vec<(DmgType, f32)> {
        self.name.calc_dmg(attack_type)
    }
}
