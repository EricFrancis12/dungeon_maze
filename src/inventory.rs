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
            .add_event::<InventoryChanged>()
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

fn pick_up_items(
    mut commands: Commands,
    mut event_reader: EventReader<PendingInteractionExecuted>,
    mut event_writer: EventWriter<InventoryChanged>,
    item_query: Query<(Entity, &Item), With<Interactable>>,
    mut inventory: ResMut<Inventory>,
) {
    for event in event_reader.read() {
        for (entity, item) in item_query.iter() {
            if entity == event.0 {
                match inventory.try_insert(item) {
                    Ok(_) => {
                        commands.entity(entity).despawn_recursive();
                        event_writer.send(InventoryChanged);
                    }
                    Err(err) => println!("error adding item to inventory: {}", err),
                }
                break;
            }
        }
    }
}
