use crate::{
    animation::CyclicAnimation,
    interaction::Interactable,
    inventory::Item,
    meshes::{new_staircase_mesh, new_stairs_mesh},
    world::{data::WorldData, ChunkCellMarker},
};

use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape, RigidBody};
use rand::Rng;

use super::{item::spawn_item_bundle, EntitySpawner};

const CHAIR_COLLIDER_HX: f32 = 0.2;
const CHAIR_COLLIDER_HY: f32 = 0.25;
const CHAIR_COLLIDER_HZ: f32 = 0.2;

const TREASURE_CHEST_COLLIDER_HX: f32 = 0.5;
const TREASURE_CHEST_COLLIDER_HY: f32 = 0.3;
const TREASURE_CHEST_COLLIDER_HZ: f32 = 0.3;
const TREASURE_CHEST_MIN_ANIMATION: u32 = 8; // TODO: refactor
const TREASURE_CHEST_MAX_ANIMATION: u32 = 9; // TODO: refactor
const TREASURE_CHEST_INTERACTABLE_RANGE: f32 = 2.0;

#[derive(Component)]
pub struct OCItemContainer;

pub fn spawn_chair_bundle(
    entity_spawner: &mut impl EntitySpawner,
    asset_server: &Res<AssetServer>,
) {
    entity_spawner
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(0.0, CHAIR_COLLIDER_HY * 2.0, 0.0),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(CHAIR_COLLIDER_HX, CHAIR_COLLIDER_HY, CHAIR_COLLIDER_HZ),
            Name::new("Chair"),
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneBundle {
                    scene: asset_server
                        .load(GltfAssetLabel::Scene(0).from_asset("embedded://models/chair.glb")),
                    transform: Transform::from_xyz(0.0, -CHAIR_COLLIDER_HY, 0.0),
                    ..default()
                },
                Name::new("Chair Model"),
            ));
        });
}

pub fn spawn_treasure_chest_bundle(
    entity_spawner: &mut impl EntitySpawner,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    world_data: &Res<WorldData>,
    ccm: &ChunkCellMarker,
) {
    entity_spawner
        .spawn((
            OCItemContainer,
            CyclicAnimation::new(TREASURE_CHEST_MIN_ANIMATION, TREASURE_CHEST_MAX_ANIMATION),
            Interactable {
                range: TREASURE_CHEST_INTERACTABLE_RANGE,
            },
            SpatialBundle {
                transform: Transform::from_xyz(0.0, TREASURE_CHEST_COLLIDER_HY, 0.0),
                ..default()
            },
            Collider::cuboid(
                TREASURE_CHEST_COLLIDER_HX,
                TREASURE_CHEST_COLLIDER_HY,
                TREASURE_CHEST_COLLIDER_HZ,
            ),
            Name::new("Treasure Chest"),
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneBundle {
                    scene: asset_server.load(
                        GltfAssetLabel::Scene(0).from_asset("embedded://models/treasure_chest.glb"),
                    ),
                    transform: Transform::from_xyz(0.0, -TREASURE_CHEST_COLLIDER_HY, 0.0),
                    ..default()
                },
                Name::new("Treasure Chest Model"),
            ));

            let item = if let Some(cell_data) = world_data.at_cell(ccm.chunk_xyz(), ccm.cell_xz()) {
                match &cell_data.treasure_chest_data.item {
                    Some(i) => i.clone(),
                    None => return,
                }
            } else {
                let mut rng = ccm.to_rng();
                // TODO: items with a max stack size of 1
                // should only be able to spawn with an amt of 1
                let amt = rng.gen_range(1..=3);
                Item::choose(&mut rng, amt)
            };

            spawn_item_bundle(
                item,
                parent,
                meshes,
                Some(Transform::from_xyz(0.0, 0.2, 0.0)),
                false,
                false,
                false,
            );
        });
}

pub fn spawn_staircase_bundle(
    entity_spawner: &mut impl EntitySpawner,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    let mesh = new_staircase_mesh();

    entity_spawner.spawn((
        Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).unwrap(),
        PbrBundle {
            mesh: meshes.add(mesh),
            transform: Transform {
                translation: Vec3 {
                    x: 0.0,
                    y: 2.0,
                    z: -2.0,
                },
                scale: Vec3 {
                    x: 2.0,
                    y: 2.0,
                    z: 2.0,
                },
                ..default()
            },
            ..default()
        },
        Name::new("Stairs"),
    ));
}

pub fn spawn_stairs_bundle(
    entity_spawner: &mut impl EntitySpawner,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    let mesh = new_stairs_mesh();

    entity_spawner.spawn((
        Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).unwrap(),
        PbrBundle {
            mesh: meshes.add(mesh),
            transform: Transform {
                translation: Vec3 {
                    x: 0.0,
                    y: 2.0,
                    z: -2.0,
                },
                scale: Vec3 {
                    x: 2.0,
                    y: 2.0,
                    z: 2.0,
                },
                ..default()
            },
            ..default()
        },
        Name::new("Stairs"),
    ));
}
