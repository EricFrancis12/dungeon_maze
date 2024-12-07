use crate::inventory::Item;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::EntitySpawner;

pub fn spawn_item_bundle(
    item: Item,
    entity_spawner: &mut impl EntitySpawner,
    meshes: &mut ResMut<Assets<Mesh>>,
    transform: Option<Transform>,
    interactable: bool,
    collider: bool,
    rigid_body: bool,
) {
    let mesh = meshes.add(
        Cuboid::from_size(Vec3 {
            x: 0.2,
            y: 0.2,
            z: 0.2,
        })
        .mesh(),
    );

    let mut entity_commands = entity_spawner.spawn((
        item,
        PbrBundle {
            mesh,
            transform: transform.unwrap_or_default(),
            ..default()
        },
        Name::new("Item"),
    ));

    if interactable {
        entity_commands.insert(Item::interactable());
    }
    if collider {
        entity_commands.insert(Collider::cuboid(0.1, 0.1, 0.1));
    }
    if rigid_body {
        entity_commands.insert(RigidBody::Dynamic);
    }
}
