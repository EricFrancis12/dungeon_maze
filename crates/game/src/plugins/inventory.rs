use crate::plugins::world::{CELL_SIZE, CHUNK_SIZE};
use bevy::{prelude::*, ui::RelativeCursorPosition};
use bevy_text_popup::{TextPopupEvent, TextPopupLocation, TextPopupTimeout};
use dungeon_maze_common::{
    interaction::{Interactable, PendingInteractionExecuted},
    inventory::{
        item::Item, Inventory, InventoryChanged, ItemRemovedFromOCItemContainer, ItemUsed,
        PlayerDroppedItem,
    },
    menu::{DragState, Dragging, Menu},
    utils::entity::get_n_parent,
    world::{ChunkCellMarker, OCItemContainer},
};

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

pub fn pick_up_items(
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
                                ccm: ChunkCellMarker::from_global_transform(
                                    gt, CHUNK_SIZE, CELL_SIZE,
                                ),
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

pub fn drop_dragged_item(
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
