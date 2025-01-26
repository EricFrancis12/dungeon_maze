pub mod equipment;
pub mod item;

#[cfg(test)]
mod inventory_test;

use crate::{
    inventory::{
        equipment::{Equipment, EquipmentSlotName},
        item::Item,
    },
    should_not_happen,
    world::ChunkCellMarker,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

const INVENTORY_MAX_SIZE: usize = 16;

#[derive(Clone, Debug, Default, Deserialize, Resource, Serialize)]
pub struct Inventory {
    pub slots: [Option<Item>; INVENTORY_MAX_SIZE],
    pub equipment: Equipment,
}

impl Inventory {
    pub fn insert(&mut self, item: Item) -> Option<Item> {
        let mut temp_item = item.clone();

        for slot in self.slots.iter_mut() {
            if let Some(i) = slot {
                if i.name == item.name {
                    let rem_item = i.merge(temp_item);
                    if let Some(ri) = rem_item {
                        temp_item = ri;
                        continue;
                    } else {
                        return None;
                    }
                }
            }

            if slot.is_none() {
                let mut new_item = item.clone_with_amt(0);
                let rem_item = new_item.merge(temp_item);
                *slot = Some(new_item);
                if let Some(ri) = rem_item {
                    temp_item = ri;
                    continue;
                } else {
                    return None;
                }
            }
        }

        Some(temp_item)
    }

    pub fn merge_swap_at(&mut self, a: usize, b: usize) {
        let slot_a_clone = self.slots[a].clone();

        match (slot_a_clone, &mut self.slots[b]) {
            (Some(item_a), Some(item_b)) => {
                // Check if the two stacks have the same ItemName, and if so merge them.
                if item_a.name == item_b.name {
                    let rem_items = item_b.merge(item_a);
                    self.slots[a] = rem_items.to_owned();
                    return;
                }
            }
            _ => (),
        };

        self.slots.swap(a, b)
    }

    pub fn use_at(&mut self, i: usize) -> (Option<Item>, bool) {
        if let Some(slot) = self.slots.get_mut(i) {
            if let Some(item) = slot {
                let result = item._use();
                if item.amt == 0 {
                    *slot = None;
                }
                return result;
            }
        } else {
            should_not_happen!("indexing inventory out of bounds: {}", i);
        }
        (None, false)
    }

    pub fn is_equipable_at(&self, i: usize, name: &EquipmentSlotName) -> bool {
        match self.slots.get(i) {
            Some(slot) => slot.is_none() || slot.as_ref().unwrap().is_equipable_at(&name),
            None => {
                should_not_happen!("indexing inventory out of bounds: {}", i);
                false
            }
        }
    }

    pub fn equip_at(&mut self, i: usize, name: &EquipmentSlotName) -> bool {
        if let Some(slot) = self.slots.get_mut(i) {
            let equipment_slot = self.equipment.at_mut(&name);
            let equipment_slot_clone = equipment_slot.clone();

            match slot {
                Some(item) => {
                    if item.is_equipable_at(&name) {
                        *equipment_slot = Some(item.clone());
                        *slot = equipment_slot_clone;
                        return true;
                    }
                }
                None => {
                    *equipment_slot = None;
                    *slot = equipment_slot_clone;
                    return true;
                }
            }
        } else {
            should_not_happen!("indexing inventory out of bounds: {}", i);
        }
        false
    }
}

#[derive(Event)]
pub struct InventoryChanged;

#[derive(Event)]
pub struct ItemUsed(pub Item, pub Entity);

#[derive(Event)]
pub struct PlayerDroppedItem(pub Item);

#[derive(Event)]
pub struct ItemRemovedFromOCItemContainer {
    pub ccm: ChunkCellMarker,
    pub _item: Item,
    pub _entity: Entity,
}
