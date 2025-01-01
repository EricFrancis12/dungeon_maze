use crate::{
    animation::CyclicAnimation,
    interaction::Interactable,
    inventory::Item,
    world::{data::WorldData, ChunkCellMarker},
};

use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, RigidBody};
use rand::Rng;

use super::{item::spawn_item_bundle, EntitySpawner};

const CHAIR_COLLIDER_HX: f32 = 0.2;
const CHAIR_COLLIDER_HY: f32 = 0.25;
const CHAIR_COLLIDER_HZ: f32 = 0.2;

const TREASURE_CHEST_COLLIDER_HX: f32 = 0.5;
const TREASURE_CHEST_COLLIDER_HY: f32 = 0.3;
const TREASURE_CHEST_COLLIDER_HZ: f32 = 0.3;
const TREASURE_CHEST_MIN_ANIMATION: u32 = 12; // TODO: refactor
const TREASURE_CHEST_MAX_ANIMATION: u32 = 13; // TODO: refactor
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
                        .load(GltfAssetLabel::Scene(0).from_asset("models/Chair.glb")),
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
                    scene: asset_server
                        .load(GltfAssetLabel::Scene(0).from_asset("models/Treasure_Chest.glb")),
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
    asset_server: &Res<AssetServer>,
) {
    let mut shapes: Vec<(Vec3, Quat, Collider)> = Vec::new();

    let step_collider = Collider::cuboid(0.2, 0.1, 0.82);
    let flat_collider = Collider::cuboid(0.5, 0.01, 2.0);

    // lower steps
    for i in 0..7 {
        shapes.push((
            Vec3 {
                x: -0.9 + (i as f32 * 0.3),
                y: 0.1 + (i as f32 * 0.3),
                z: -1.18,
            },
            Quat::default(),
            step_collider.clone(),
        ));
    }

    // upper steps
    for j in 0..5 {
        shapes.push((
            Vec3 {
                x: 0.3 - (j as f32 * 0.3),
                y: 2.5 + (j as f32 * 0.3),
                z: 1.18,
            },
            Quat::default(),
            step_collider.clone(),
        ));
    }

    // lower flat section
    shapes.push((
        Vec3 {
            x: 1.5,
            y: 2.2,
            z: 0.0,
        },
        Quat::default(),
        flat_collider.clone(),
    ));

    // upper flat section
    shapes.push((
        Vec3 {
            x: -1.5,
            y: 4.0,
            z: 0.0,
        },
        Quat::default(),
        flat_collider,
    ));

    entity_spawner
        .spawn((
            SpatialBundle::default(),
            RigidBody::Fixed,
            Collider::compound(shapes),
            Name::new("Staircase"),
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneBundle {
                    scene: asset_server
                        .load(GltfAssetLabel::Scene(0).from_asset("models/Staircase.glb")),
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
                Name::new("Staircase Model"),
            ));
        });
}
