use crate::{
    interaction::{Interactable, PendingInteractionExecuted},
    menu::{DraggingInventorySlot, Menu},
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
    Misc,
    RawMaterial,
    Weapon,
}

#[derive(Clone, Component, Debug, Deserialize, Display, EnumIter, Eq, PartialEq, Serialize)]
pub enum ItemName {
    Coal,
    Cotton,
    Flint,
    HealthPotion,
    StaminaPotion,
    HealthPoison,
    StaminaPoison,
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
            Self::HealthPotion | Self::StaminaPotion | Self::HealthPoison | Self::StaminaPoison => {
                ItemType::Consumable
            }
        }
    }

    pub fn max_amt(&self) -> u16 {
        match self.item_type() {
            ItemType::Consumable | ItemType::Misc | ItemType::RawMaterial => 64,
            ItemType::Weapon => 1,
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
                Self::HealthPoison => asset_server.load("images/health_poison.png"),
                Self::StaminaPoison => asset_server.load("images/stamina_poison.png"),
            },
            ..default()
        }
    }
}

#[derive(Clone, Component, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
}

#[derive(Clone, Debug, Default, Deserialize, Resource, Serialize)]
pub struct Inventory(pub [Option<Item>; INVENTORY_MAX_SIZE]);

impl Inventory {
    fn insert(&mut self, item: Item) -> Option<Item> {
        let mut temp_item = item.clone();

        for slot in self.0.iter_mut() {
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

    pub fn use_at(&mut self, i: usize) -> (Option<Item>, bool) {
        if let Some(slot) = self.0.get_mut(i) {
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
    mut event_reader: EventReader<StateTransitionEvent<DraggingInventorySlot>>,
    mut inv_event_writer: EventWriter<InventoryChanged>,
    mut pdi_event_writer: EventWriter<PlayerDroppedItem>,
    menu_query: Query<&RelativeCursorPosition, With<Menu>>,
    mut inventory: ResMut<Inventory>,
) {
    for event in event_reader.read() {
        if let Ok(rel_cursor_position) = menu_query.get_single() {
            if !rel_cursor_position.mouse_over() {
                if let Some(prev_dragging_inventory_slot) = &event.exited {
                    if let Some(i) = prev_dragging_inventory_slot.0 {
                        if let Some(slot) = inventory.0.get_mut(i) {
                            if let Some(item) = slot {
                                pdi_event_writer.send(PlayerDroppedItem(item.clone()));
                                inv_event_writer.send(InventoryChanged);
                                *slot = None;
                            }
                        }
                    }
                }
            }
        }

        break;
    }
}
