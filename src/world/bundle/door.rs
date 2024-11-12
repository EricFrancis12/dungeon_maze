use super::{DOOR_CLOSE_FRAMES, DOOR_SCALE, WALL_THICKNESS};
use crate::{
    interaction::Interactable,
    utils::entity::incr_betw_transforms,
    world::{CyclicTransform, Side, CELL_SIZE},
};

use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;
use std::f32::consts::PI;

pub fn spawn_new_door_bundle(
    side: Side,
    child_builder: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
) {
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

    child_builder.spawn((
        SceneBundle {
            scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/Door.glb")),
            transform: transforms[0],
            ..default()
        },
        Collider::cuboid(CELL_SIZE / 8.0, CELL_SIZE / 4.0, WALL_THICKNESS / 2.0),
        Interactable { range: 2.0 },
        CyclicTransform::new_cycled(vec![transforms, clone]),
        Name::new(format!("{} Wall Door", side)),
    ));
}
