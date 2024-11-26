use crate::{
    error::Error,
    interaction::{Interactable, PendingInteractionExecuted},
    world::bundle::special::Item,
};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

const INVENTORY_MAX_SIZE: usize = 16;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Inventory>()
            .add_event::<ItemPickedUp>()
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
pub struct ItemPickedUp;

fn pick_up_items(
    mut commands: Commands,
    mut event_reader: EventReader<PendingInteractionExecuted>,
    mut inv_event_writer: EventWriter<InventoryChanged>,
    mut ipu_event_writer: EventWriter<ItemPickedUp>,
    item_query: Query<(Entity, &Item), With<Interactable>>,
    mut inventory: ResMut<Inventory>,
) {
    for event in event_reader.read() {
        for (entity, item) in item_query.iter() {
            if entity == event.0 {
                match inventory.try_insert(item) {
                    Ok(_) => {
                        commands.entity(entity).despawn_recursive();
                        inv_event_writer.send(InventoryChanged);
                        ipu_event_writer.send(ItemPickedUp);
                    }
                    Err(err) => println!("error adding item to inventory: {}", err),
                }
                break;
            }
        }
    }
}
