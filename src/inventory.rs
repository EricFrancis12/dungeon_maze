use crate::{
    error::Error,
    interaction::{Interactable, PendingInteractionExecuted},
    utils::entity::get_n_parent,
    world::{
        bundle::special::{Item, TreasureChest},
        ChunkCellMarker,
    },
};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

const INVENTORY_MAX_SIZE: usize = 16;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Inventory>()
            .add_event::<InventoryChanged>()
            .add_event::<ItemRemovedFromTreasureChest>()
            .add_systems(Update, pick_up_items);
    }
}

#[derive(Clone, Debug, Default, Deserialize, Resource, Serialize)]
pub struct Inventory([Option<Item>; INVENTORY_MAX_SIZE]);

impl Inventory {
    fn try_insert(&mut self, item: &Item) -> Result<(), Error> {
        for slot in self.0.iter_mut() {
            if slot.is_none() {
                *slot = Some(item.clone());
                return Ok(());
            }
        }
        Err(Error::InventoryOverflow)
    }
}

#[derive(Event)]
pub struct InventoryChanged;

#[derive(Event)]
pub struct ItemRemovedFromTreasureChest {
    pub ccm: ChunkCellMarker,
    pub _item: Item,
    pub _entity: Entity,
}

fn pick_up_items(
    mut commands: Commands,
    mut event_reader: EventReader<PendingInteractionExecuted>,
    mut inv_event_writer: EventWriter<InventoryChanged>,
    mut irm_event_writer: EventWriter<ItemRemovedFromTreasureChest>,
    item_query: Query<(Entity, &Item), With<Interactable>>,
    parent_query: Query<&Parent>,
    treasure_chest_query: Query<&GlobalTransform, With<TreasureChest>>,
    mut inventory: ResMut<Inventory>,
) {
    for event in event_reader.read() {
        for (entity, item) in item_query.iter() {
            if entity == event.0 {
                match inventory.try_insert(item) {
                    Ok(_) => {
                        // Check if item was inside of a treasure chest
                        let parent_entity = get_n_parent(entity, &parent_query, 1);
                        if let Ok(gt) = treasure_chest_query.get(parent_entity) {
                            irm_event_writer.send(ItemRemovedFromTreasureChest {
                                ccm: ChunkCellMarker::from_global_transform(gt),
                                _item: item.clone(),
                                _entity: parent_entity,
                            });
                        }

                        commands.entity(entity).despawn_recursive();
                        inv_event_writer.send(InventoryChanged);
                    }
                    Err(err) => println!("error adding item to inventory: {}", err),
                }
                break;
            }
        }
    }
}
