use crate::{
    utils::noise::noise_from_xyz_seed,
    world::{AssetLib, Cell, CellSpecial, CellWall, ChunkCellMarker},
    SEED,
};

use super::{
    super::{Side, CELL_SIZE, GRID_SIZE},
    door::*,
    special::*,
    wall::*,
    WALL_THICKNESS,
};
use bevy::prelude::*;

pub fn spawn_new_cell_bundle(
    cell: &Cell,
    ccm: ChunkCellMarker,
    child_builder: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    asset_lib: &Res<AssetLib>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let cell_bundle = (
        SpatialBundle {
            transform: Transform::from_xyz(calc_floor_pos(ccm.x), 0.0, calc_floor_pos(ccm.z)),
            ..default()
        },
        cell.clone(),
        ccm.clone(),
        Name::new(format!("Cell_({},{})", ccm.x, ccm.z)),
    );

    child_builder.spawn(cell_bundle).with_children(|parent| {
        let mesh = meshes.add(
            Cuboid::from_size(Vec3 {
                x: CELL_SIZE,
                y: WALL_THICKNESS,
                z: CELL_SIZE,
            })
            .mesh(),
        );

        // Floor
        if cell.floor == CellWall::Solid {
            spawn_new_wall_bundle(
                Side::Down,
                parent,
                &mesh,
                &materials.add(Color::linear_rgba(0.55, 0.0, 0.0, 1.0)),
            );
        }

        // Ceiling
        if cell.ceiling == CellWall::Solid {
            spawn_new_wall_bundle(
                Side::Up,
                parent,
                &mesh,
                &materials.add(Color::linear_rgba(0.0, 0.2, 0.4, 1.0)),
            );
        }

        let noise_xyz = noise_from_xyz_seed(SEED, ccm.chunk_x, ccm.chunk_y, ccm.chunk_z);

        let path = if noise_xyz < -0.2 {
            "images/wall-1.png"
        } else if noise_xyz < 0.0 {
            "images/wall-2.png"
        } else if noise_xyz < 0.2 {
            "images/wall-3.png"
        } else {
            "images/wall-4.png"
        };

        let wall_texture_handle = asset_server.load(path);
        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(wall_texture_handle),
            ..Default::default()
        });

        // Top wall
        if cell.wall_top == CellWall::Solid {
            spawn_new_wall_bundle(Side::Top, parent, &mesh, &material);
        }

        // Top wall with door
        if cell.wall_top == CellWall::SolidWithDoorGap {
            spawn_new_wall_with_door_gap_bundle(Side::Top, parent, asset_lib, meshes, &material);
        }

        // Top door
        if cell.door_top {
            spawn_new_door_bundle(Side::Top, parent, &asset_server);
        }

        // Bottom wall
        if cell.wall_bottom == CellWall::Solid {
            spawn_new_wall_bundle(Side::Bottom, parent, &mesh, &material);
        }

        // Bottom wall with door
        if cell.wall_bottom == CellWall::SolidWithDoorGap {
            spawn_new_wall_with_door_gap_bundle(Side::Bottom, parent, asset_lib, meshes, &material);
        }

        // Bottom door
        if cell.door_bottom {
            spawn_new_door_bundle(Side::Bottom, parent, &asset_server);
        }

        // Left wall
        if cell.wall_left == CellWall::Solid {
            spawn_new_wall_bundle(Side::Left, parent, &mesh, &material);
        }

        // Left wall with door
        if cell.wall_left == CellWall::SolidWithDoorGap {
            spawn_new_wall_with_door_gap_bundle(Side::Left, parent, asset_lib, meshes, &material);
        }

        // Left door
        if cell.door_left {
            spawn_new_door_bundle(Side::Left, parent, &asset_server);
        }

        // Right wall
        if cell.wall_right == CellWall::Solid {
            spawn_new_wall_bundle(Side::Right, parent, &mesh, &material);
        }

        // Right wall with door
        if cell.wall_right == CellWall::SolidWithDoorGap {
            spawn_new_wall_with_door_gap_bundle(Side::Right, parent, asset_lib, meshes, &material);
        }

        // Right door
        if cell.door_right {
            spawn_new_door_bundle(Side::Right, parent, &asset_server);
        }

        // Special
        match cell.special {
            CellSpecial::None => (),
            CellSpecial::Chair => {
                spawn_new_chair_bundle(parent, asset_server);
            }
            CellSpecial::TreasureChest => {
                spawn_new_treasure_chest_bundle(parent, asset_server);
            }
            CellSpecial::Staircase => spawn_new_staircase_bundle(parent, asset_server),
        }
    });
}

fn calc_floor_pos(index: usize) -> f32 {
    let mut positions = vec![CELL_SIZE / 2.0, -CELL_SIZE / 2.0];
    while positions.len() < GRID_SIZE {
        positions.insert(0, positions[0] + CELL_SIZE);
        positions.push(positions.last().unwrap() - CELL_SIZE);
    }
    positions.get(index).unwrap().to_owned()
}
