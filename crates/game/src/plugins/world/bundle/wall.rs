use crate::plugins::world::{bundle::WALL_THICKNESS, CELL_SIZE};
use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape};
use dungeon_maze_common::{
    meshes::{new_wall_with_door_gap_mesh, new_wall_with_window_gap_mesh},
    world::{CellWall, EntitySpawner, Side},
};
use std::f32::consts::PI;

const WALL_SCALE: Vec3 = Vec3 {
    x: 1.0,
    y: 1.0,
    z: WALL_THICKNESS,
};

pub fn spawn_wall_bundle(
    side: Side,
    wall: &CellWall,
    entity_spawner: &mut impl EntitySpawner,
    meshes: &mut ResMut<Assets<Mesh>>,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
) {
    match wall {
        CellWall::Solid => spawn_solid_wall_bundle(side, entity_spawner, &mesh, &material),
        CellWall::SolidWithDoorGap => {
            spawn_wall_with_door_gap_bundle(side, entity_spawner, meshes, &material);
        }
        CellWall::SolidWithWindowGap => {
            spawn_wall_with_window_gap_bundle(side, entity_spawner, meshes, &material);
        }
        _ => (),
    }
}

pub fn spawn_solid_wall_bundle(
    side: Side,
    entity_spawner: &mut impl EntitySpawner,
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

    entity_spawner.spawn((
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

pub fn spawn_wall_with_door_gap_bundle(
    side: Side,
    entity_spawner: &mut impl EntitySpawner,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: &Handle<StandardMaterial>,
) {
    let (x, y, z, r) = wall_dims(&side);
    let mesh_handle = meshes.add(new_wall_with_door_gap_mesh());
    let mesh = meshes.get(&mesh_handle).unwrap();

    entity_spawner.spawn((
        PbrBundle {
            mesh: mesh_handle,
            material: material.clone(),
            transform: Transform::from_xyz(x, y, z)
                .with_scale(WALL_SCALE)
                .with_rotation(r),
            ..default()
        },
        Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap(),
        Name::new(format!("{} Wall With Door Gap", side)),
    ));
}

pub fn spawn_wall_with_window_gap_bundle(
    side: Side,
    entity_spawner: &mut impl EntitySpawner,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: &Handle<StandardMaterial>,
) {
    let (x, y, z, r) = wall_dims(&side);
    let mesh_handle = meshes.add(new_wall_with_window_gap_mesh());
    let mesh = meshes.get(&mesh_handle).unwrap();

    entity_spawner.spawn((
        PbrBundle {
            mesh: mesh_handle.clone(),
            material: material.clone(),
            transform: Transform::from_xyz(x, y, z)
                .with_scale(WALL_SCALE)
                .with_rotation(r),
            ..default()
        },
        Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap(),
        Name::new(format!("{} Wall With Window Gap", side)),
    ));
}

fn wall_dims(side: &Side) -> (f32, f32, f32, Quat) {
    match side {
        Side::Top => (
            CELL_SIZE / 2.0 - WALL_THICKNESS,
            CELL_SIZE / 2.0,
            0.0,
            Quat::from_rotation_y(PI / 2.0),
        ),
        Side::Bottom => (
            -CELL_SIZE / 2.0,
            CELL_SIZE / 2.0,
            0.0,
            Quat::from_rotation_y(PI / 2.0),
        ),
        Side::Left => (
            0.0,
            CELL_SIZE / 2.0,
            CELL_SIZE / 2.0,
            Quat::from_rotation_x(0.0),
        ),
        Side::Right => (
            0.0,
            CELL_SIZE / 2.0,
            -CELL_SIZE / 2.0,
            Quat::from_rotation_x(0.0),
        ),
        _ => panic!("unexpected side: {}", side),
    }
}
