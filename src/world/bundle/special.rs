use crate::{animation::CyclicAnimation, interaction::Interactable};

use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, RigidBody};
use serde::{Deserialize, Serialize};

const CHAIR_COLLIDER_HX: f32 = 0.2;
const CHAIR_COLLIDER_HY: f32 = 0.25;
const CHAIR_COLLIDER_HZ: f32 = 0.2;

const TREASURE_CHEST_COLLIDER_HX: f32 = 0.5;
const TREASURE_CHEST_COLLIDER_HY: f32 = 0.3;
const TREASURE_CHEST_COLLIDER_HZ: f32 = 0.3;
const TREASURE_CHEST_MIN_ANIMATION: u32 = 3;
const TREASURE_CHEST_MAX_ANIMATION: u32 = 4;
const TREASURE_CHEST_INTERACTABLE_RANGE: f32 = 2.0;

pub const ITEM_INTERACTABLE_RANGE: f32 = 1.8;

#[derive(Clone, Component, Debug, Deserialize, Serialize)]
pub struct Item;

impl Item {
    pub fn interactable() -> Interactable {
        Interactable {
            range: ITEM_INTERACTABLE_RANGE,
        }
    }
}

#[derive(Component)]
pub struct TreasureChest;

impl TreasureChest {
    pub fn is_open(ca: &CyclicAnimation) -> bool {
        match ca.value() {
            TREASURE_CHEST_MAX_ANIMATION => true,
            _ => false,
        }
    }

    fn cyclic_animation() -> CyclicAnimation {
        CyclicAnimation::new(TREASURE_CHEST_MIN_ANIMATION, TREASURE_CHEST_MAX_ANIMATION)
    }

    fn interactable() -> Interactable {
        Interactable {
            range: TREASURE_CHEST_INTERACTABLE_RANGE,
        }
    }
}

pub fn spawn_chair_bundle(child_builder: &mut ChildBuilder, asset_server: &Res<AssetServer>) {
    child_builder
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

fn spawn_item_bundle(
    item: Item,
    child_builder: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    transform: Option<Transform>,
    interactable: bool,
    collider: bool,
) {
    let mesh = meshes.add(
        Cuboid::from_size(Vec3 {
            x: 0.2,
            y: 0.2,
            z: 0.2,
        })
        .mesh(),
    );

    let mut commands = child_builder.spawn((
        item,
        PbrBundle {
            mesh,
            transform: transform.unwrap_or(Transform::default()),
            ..default()
        },
        Name::new("Item"),
    ));

    if interactable {
        commands.insert(Item::interactable());
    }

    if collider {
        commands.insert(Collider::cuboid(0.1, 0.1, 0.1));
    }
}

pub fn spawn_treasure_chest_bundle(
    child_builder: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    child_builder
        .spawn((
            TreasureChest,
            TreasureChest::cyclic_animation(),
            TreasureChest::interactable(),
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

            spawn_item_bundle(
                Item,
                parent,
                meshes,
                Some(Transform::from_xyz(0.0, 0.2, 0.0)),
                false,
                false,
            );
        });
}

pub fn spawn_staircase_bundle(child_builder: &mut ChildBuilder, asset_server: &Res<AssetServer>) {
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

    child_builder
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
