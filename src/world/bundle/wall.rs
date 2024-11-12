use crate::world::{AssetLib, Side, CELL_SIZE};

use super::{WALL_THICKNESS, WALL_WITH_DOOR_SCALE};
use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape};
use std::f32::consts::PI;

pub fn spawn_new_wall_bundle(
    side: Side,
    child_builder: &mut ChildBuilder,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
) {
    let (x, y, z, r) = match side {
        Side::Top => (
            CELL_SIZE / 2.0 - WALL_THICKNESS / 2.0,
            CELL_SIZE / 2.0,
            0.0,
            Quat::from_rotation_z(PI / 2.0),
        ),
        Side::Bottom => (
            -CELL_SIZE / 2.0 + WALL_THICKNESS / 2.0,
            CELL_SIZE / 2.0,
            0.0,
            Quat::from_rotation_z(PI * 3.0 / 2.0),
        ),
        Side::Left => (
            0.0,
            CELL_SIZE / 2.0,
            CELL_SIZE / 2.0 - WALL_THICKNESS / 2.0,
            Quat::from_rotation_x(PI * 3.0 / 2.0),
        ),
        Side::Right => (
            0.0,
            CELL_SIZE / 2.0,
            -CELL_SIZE / 2.0 + WALL_THICKNESS / 2.0,
            Quat::from_rotation_x(PI / 2.0),
        ),
        Side::Up => (
            0.0,
            CELL_SIZE - WALL_THICKNESS / 2.0,
            0.0,
            Quat::from_rotation_x(PI),
        ),
        Side::Down => (0.0, WALL_THICKNESS / 2.0, 0.0, Quat::from_rotation_x(0.0)),
    };

    child_builder.spawn((
        PbrBundle {
            mesh: mesh.clone(),
            material: material.clone(),
            transform: Transform::from_xyz(x, y, z).with_rotation(r),
            ..default()
        },
        Collider::cuboid(CELL_SIZE / 2.0, WALL_THICKNESS / 2.0, CELL_SIZE / 2.0),
        Name::new(format!("{} Wall", side)),
    ));
}

pub fn spawn_new_wall_with_door_gap_bundle(
    side: Side,
    child_builder: &mut ChildBuilder,
    asset_lib: &Res<AssetLib>,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: &Handle<StandardMaterial>,
) {
    let (x, y, z, r) = match side {
        Side::Top => (
            CELL_SIZE / 2.0,
            CELL_SIZE / 2.0,
            0.0,
            Quat::from_rotation_x(PI / 2.0) * Quat::from_rotation_z(PI / 2.0),
        ),
        Side::Bottom => (
            -CELL_SIZE / 2.0 + WALL_THICKNESS,
            CELL_SIZE / 2.0,
            0.0,
            Quat::from_rotation_x(PI / 2.0) * Quat::from_rotation_z(PI / 2.0),
        ),
        Side::Left => (
            0.0,
            CELL_SIZE / 2.0,
            CELL_SIZE / 2.0 - WALL_THICKNESS,
            Quat::from_rotation_x(PI / 2.0),
        ),
        Side::Right => (
            0.0,
            CELL_SIZE / 2.0,
            -CELL_SIZE / 2.0,
            Quat::from_rotation_x(PI / 2.0),
        ),
        _ => panic!("unexpected side: {}", side),
    };

    let mesh_handle = &asset_lib.meshes[0];
    let mesh = meshes.get(mesh_handle).unwrap();

    child_builder.spawn((
        PbrBundle {
            mesh: mesh_handle.clone(),
            material: material.clone(),
            transform: Transform::from_xyz(x, y, z)
                .with_scale(WALL_WITH_DOOR_SCALE)
                .with_rotation(r),
            ..default()
        },
        Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap(),
        Name::new(format!("{} Wall With Door Gap", side)),
    ));
}
