use crate::{
    interaction::{Interactable, PendingInteractionExecuted},
    menu::{DragState, Dragging, Menu},
    player::attack::{AttackHand, AttackType},
    should_not_happen,
    utils::entity::get_n_parent,
    world::{bundle::special::OCItemContainer, ChunkCellMarker},
};

use bevy::{prelude::*, ui::RelativeCursorPosition};
use bevy_text_popup::{TextPopupEvent, TextPopupLocation, TextPopupTimeout};
use rand::{rngs::StdRng, Rng};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

const ITEM_INTERACTABLE_RANGE: f32 = 1.8;
const INVENTORY_MAX_SIZE: usize = 16;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Inventory>()
            .add_event::<InventoryChanged>()
            .add_event::<ItemUsed>()
            .add_event::<PlayerDroppedItem>()
            .add_event::<ItemRemovedFromOCItemContainer>()
            .add_systems(Update, (pick_up_items, drop_dragged_item));
    }
}

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

    pub fn ui_image(&self, asset_server: &Res<AssetServer>) -> UiImage {
        UiImage {
            texture: match self {
                Self::Coal => asset_server.load("images/coal.png"),
                Self::Cotton => asset_server.load("images/cotton.png"),
                Self::Flint => asset_server.load("images/flint.png"),
                Self::HealthPotion => asset_server.load("images/health_potion.png"),
                Self::StaminaPotion => asset_server.load("images/stamina_potion.png"),
                Self::HealthRegenPotion => asset_server.load("images/health_regen_potion.png"),
                Self::StaminaRegenPotion => asset_server.load("images/stamina_regen_potion.png"),
                Self::HealthPoison => asset_server.load("images/health_poison.png"),
                Self::StaminaPoison => asset_server.load("images/stamina_poison.png"),
                Self::HealthRegenPoison => asset_server.load("images/health_regen_poison.png"),
                Self::StaminaRegenPoison => asset_server.load("images/stamina_regen_poison.png"),
                Self::Broadsword => asset_server.load("images/broadsword.png"),
                Self::Katana => asset_server.load("images/katana.png"),
            },
            ..default()
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

    pub fn animation_frames(&self, attack_type: &AttackType, attack_hand: &AttackHand) -> u32 {
        match self.name.item_type() {
            ItemType::Weapon => match (&self.name, attack_type, attack_hand) {
                // TODO: ...
                (ItemName::Broadsword, _, _) => 50,
                (ItemName::Katana, _, _) => 50,
                _ => {
                    should_not_happen!("unhandled weapon ItemName: {:?}", self.name);
                    0
                }
            },
            _ => {
                should_not_happen!(
                    "expected ItemType::Weapon when calling animation_frames(), but got: {:?}",
                    self.name.item_type()
                );
                0
            }
        }
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
}

#[derive(Clone, Copy, Debug, Deserialize, Display, EnumIter, Eq, Hash, Serialize, PartialEq)]
pub enum EquipmentSlotName {
    LeftHand,
    RightHand,
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

    fn at_mut(&mut self, name: &EquipmentSlotName) -> &mut Option<Item> {
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

#[derive(Clone, Debug, Default, Deserialize, Resource, Serialize)]
pub struct Inventory {
    pub slots: [Option<Item>; INVENTORY_MAX_SIZE],
    pub equipment: Equipment,
}

impl Inventory {
    fn insert(&mut self, item: Item) -> Option<Item> {
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

fn pick_up_items(
    mut commands: Commands,
    mut event_reader: EventReader<PendingInteractionExecuted>,
    mut inv_event_writer: EventWriter<InventoryChanged>,
    mut irm_event_writer: EventWriter<ItemRemovedFromOCItemContainer>,
    mut popup_event_writer: EventWriter<TextPopupEvent>,
    mut item_query: Query<(Entity, &mut Item), With<Interactable>>,
    parent_query: Query<&Parent>,
    container_query: Query<&GlobalTransform, With<OCItemContainer>>,
    mut inventory: ResMut<Inventory>,
) {
    for event in event_reader.read() {
        for (entity, mut item) in item_query.iter_mut() {
            if entity == event.0 {
                let content = format!("Picked up ({}) {}", item.amt, item.name);
                let send_events = || {
                    inv_event_writer.send(InventoryChanged);
                    popup_event_writer.send(TextPopupEvent {
                        content,
                        location: TextPopupLocation::BottomLeft,
                        timeout: TextPopupTimeout::Seconds(4),
                        ..default()
                    });
                };

                match inventory.insert(item.clone()) {
                    Some(rem_item) => {
                        *item = rem_item;
                        send_events();
                    }
                    None => {
                        // Check if item was inside of a container
                        let parent_entity = get_n_parent(entity, &parent_query, 1);
                        if let Ok(gt) = container_query.get(parent_entity) {
                            irm_event_writer.send(ItemRemovedFromOCItemContainer {
                                ccm: ChunkCellMarker::from_global_transform(gt),
                                _item: item.clone(),
                                _entity: parent_entity,
                            });
                        }

                        item.amt = 0;
                        send_events();

                        commands.entity(entity).despawn_recursive();
                    }
                }

                break;
            }
        }
    }
}

fn drop_dragged_item(
    mut event_reader: EventReader<StateTransitionEvent<DragState>>,
    mut inv_event_writer: EventWriter<InventoryChanged>,
    mut pdi_event_writer: EventWriter<PlayerDroppedItem>,
    menu_query: Query<&RelativeCursorPosition, With<Menu>>,
    mut inventory: ResMut<Inventory>,
) {
    for event in event_reader.read() {
        if let Ok(rel_cursor_position) = menu_query.get_single() {
            if rel_cursor_position.mouse_over() {
                continue;
            }

            if let Some(prev_dragging_slot) = &event.exited {
                match prev_dragging_slot.0 {
                    Dragging::InventorySlot(i) => {
                        if let Some(slot) = inventory.slots.get_mut(i) {
                            if let Some(item) = slot {
                                pdi_event_writer.send(PlayerDroppedItem(item.clone()));
                                inv_event_writer.send(InventoryChanged);
                                *slot = None;
                            }
                        }
                    }
                    Dragging::EquipmentSlot(name) => {
                        let slot = inventory.equipment.at_mut(&name);
                        if let Some(item) = slot {
                            pdi_event_writer.send(PlayerDroppedItem(item.clone()));
                            inv_event_writer.send(InventoryChanged);
                            *slot = None;
                        }
                    }
                    Dragging::None => {}
                }
            }
        }

        break;
    }
}
