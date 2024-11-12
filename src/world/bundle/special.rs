use crate::{animation::CyclicAnimation, interaction::Interactable};

use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, RigidBody};

const CHAIR_COLLIDER_HX: f32 = 0.2;
const CHAIR_COLLIDER_HY: f32 = 0.25;
const CHAIR_COLLIDER_HZ: f32 = 0.2;

const TREASURE_CHEST_COLLIDER_HX: f32 = 0.5;
const TREASURE_CHEST_COLLIDER_HY: f32 = 0.3;
const TREASURE_CHEST_COLLIDER_HZ: f32 = 0.3;

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

pub fn spawn_treasure_chest_bundle(
    child_builder: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
) {
    child_builder
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(0.0, TREASURE_CHEST_COLLIDER_HY, 0.0),
                ..default()
            },
            Collider::cuboid(
                TREASURE_CHEST_COLLIDER_HX,
                TREASURE_CHEST_COLLIDER_HY,
                TREASURE_CHEST_COLLIDER_HZ,
            ),
            Interactable { range: 2.0 },
            CyclicAnimation::new(2, 3),
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
