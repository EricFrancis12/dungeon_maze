use crate::{
    interaction::Interactable,
    utils::entity::incr_betw_transforms,
    world::{
        bundle::{EntitySpawner, WALL_THICKNESS},
        CyclicTransform, Side, CELL_SIZE,
    },
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;
use std::f32::consts::PI;

const DOOR_CLOSE_FRAMES: usize = 24;

const DOOR_SCALE: Vec3 = Vec3 {
    x: 0.8,
    y: 0.88,
    z: 1.0,
};

pub fn spawn_door_bundle(
    side: Side,
    entity_spawner: &mut impl EntitySpawner,
    asset_server: &Res<AssetServer>,
) {
    // TODO: refine door open/close start & end positions for animation:
    let (sx, sy, sz, sr, ex, ey, ez, er) = match side {
        Side::Top => (1.95, 1.0, 0.015, -PI / 2.0, 1.5, 1.0, 0.5, 0.0),
        Side::Bottom => (-1.95, 1.0, -0.015, PI / 2.0, -1.5, 1.0, -0.5, PI),
        Side::Left => (-0.015, 1.0, 1.95, PI, -0.5, 1.0, 1.5, -PI / 2.0),
        Side::Right => (-0.015, 1.0, -1.95, 0.0, 0.5, 1.0, -1.5, PI / 2.0),
        _ => panic!("unexpected side: {}", side),
    };

    let start = Transform::from_xyz(sx, sy, sz)
        .with_scale(DOOR_SCALE)
        .with_rotation(Quat::from_rotation_y(sr));
    let end = Transform::from_xyz(ex, ey, ez)
        .with_scale(DOOR_SCALE)
        .with_rotation(Quat::from_rotation_y(er));

    let transforms = incr_betw_transforms(start, end, DOOR_CLOSE_FRAMES);
    let mut clone = transforms.clone();
    clone.reverse();

    entity_spawner.spawn((
        SceneBundle {
            scene: asset_server
                .load(GltfAssetLabel::Scene(0).from_asset("embedded://models/door.glb")),
            transform: transforms[0],
            ..default()
        },
        Collider::cuboid(CELL_SIZE / 8.0, CELL_SIZE / 4.0, WALL_THICKNESS / 2.0),
        Interactable { range: 2.0 },
        CyclicTransform::new_cycled(vec![transforms, clone]),
        Name::new(format!("{} Wall Door", side)),
    ));
}
