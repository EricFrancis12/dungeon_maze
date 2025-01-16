use crate::{inventory::item::Item, player::attack::AttackHand};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

#[derive(
    Clone, Component, Copy, Debug, Deserialize, Display, EnumIter, Eq, Hash, Serialize, PartialEq,
)]
pub enum EquipmentSlotName {
    LeftHand,
    RightHand,
}

impl EquipmentSlotName {
    pub fn matches_target(&self, name: &Name) -> bool {
        name.to_string()
            == match self {
                Self::LeftHand => String::from("Left_Hand_Grip_Target"),
                Self::RightHand => String::from("Right_Hand_Grip_Target"),
            }
    }

    pub fn _matches_direction(&self, name: &Name) -> bool {
        name.to_string()
            == match self {
                Self::LeftHand => String::from("Left_Hand_Grip_Direction"),
                Self::RightHand => String::from("Right_Hand_Grip_Direction"),
            }
    }

    pub fn query_target<'a>(
        &self,
        query: impl IntoIterator<Item = (Entity, &'a Name)>,
    ) -> Option<Entity> {
        query
            .into_iter()
            .find(|(_, name)| self.matches_target(name))
            .map(|(e, _)| e)
    }

    pub fn _query_direction<'a>(
        &self,
        query: impl IntoIterator<Item = (Entity, &'a Name)>,
    ) -> Option<Entity> {
        query
            .into_iter()
            .find(|(_, name)| self._matches_direction(name))
            .map(|(e, _)| e)
    }
}

impl From<&AttackHand> for EquipmentSlotName {
    fn from(value: &AttackHand) -> Self {
        match value {
            AttackHand::Left => Self::LeftHand,
            AttackHand::Right => Self::RightHand,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Equipment {
    left_hand: Option<Item>,
    right_hand: Option<Item>,
}

impl Equipment {
    pub fn at(&self, name: &EquipmentSlotName) -> &Option<Item> {
        match name {
            EquipmentSlotName::LeftHand => &self.left_hand,
            EquipmentSlotName::RightHand => &self.right_hand,
        }
    }

    pub fn at_mut(&mut self, name: &EquipmentSlotName) -> &mut Option<Item> {
        match name {
            EquipmentSlotName::LeftHand => &mut self.left_hand,
            EquipmentSlotName::RightHand => &mut self.right_hand,
        }
    }

    pub fn swap(&mut self, a: &EquipmentSlotName, b: &EquipmentSlotName) {
        if a == b {
            return;
        }

        let slot_a_clone = self.at_mut(a).clone();
        let slot_b_clone = self.at_mut(b).clone();

        let slot_a = self.at_mut(a);
        *slot_a = slot_b_clone;

        let slot_b = self.at_mut(b);
        *slot_b = slot_a_clone;
    }
}
