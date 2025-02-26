use crate::{
    plugins::world::{
        bundle::{
            door::spawn_door_bundle,
            special::{
                spawn_chair_bundle, spawn_staircase_bundle, spawn_stairs_bundle,
                spawn_treasure_chest_bundle,
            },
            wall::{spawn_solid_wall_bundle, spawn_wall_bundle},
            window::spawn_window_bundle,
            WALL_THICKNESS,
        },
        CELL_SIZE, CHUNK_SIZE, GRID_SIZE,
    },
    SEED,
};
use bevy::prelude::*;
use dungeon_maze_common::{
    utils::noise::noise_from_xyz_seed,
    world::{data::WorldData, Cell, CellSpecial, CellWall, ChunkCellMarker, EntitySpawner, Side},
};

pub fn spawn_cell_bundle(
    cell: &Cell,
    ccm: ChunkCellMarker,
    entity_spawner: &mut impl EntitySpawner,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    world_data: &Res<WorldData>,
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

    entity_spawner.spawn(cell_bundle).with_children(|parent| {
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
            spawn_solid_wall_bundle(
                Side::Down,
                parent,
                &mesh,
                &materials.add(Color::linear_rgba(0.55, 0.0, 0.0, 1.0)),
            );
        }

        // Ceiling
        if cell.ceiling == CellWall::Solid {
            spawn_solid_wall_bundle(
                Side::Up,
                parent,
                &mesh,
                &materials.add(Color::linear_rgba(0.0, 0.2, 0.4, 1.0)),
            );
        }

        let noise_xyz = noise_from_xyz_seed(
            SEED,
            ccm.chunk_x,
            ccm.chunk_y,
            ccm.chunk_z,
            CHUNK_SIZE,
            CELL_SIZE,
        );

        let path = if noise_xyz < -0.2 {
            "embedded://images/wall-1.png"
        } else if noise_xyz < 0.0 {
            "embedded://images/wall-2.png"
        } else if noise_xyz < 0.2 {
            "embedded://images/wall-3.png"
        } else {
            "embedded://images/wall-4.png"
        };

        let wall_texture_handle = asset_server.load(path);
        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(wall_texture_handle),
            ..Default::default()
        });

        // Walls
        for (side, wall) in [
            (Side::Top, &cell.wall_top),
            (Side::Bottom, &cell.wall_bottom),
            (Side::Left, &cell.wall_left),
            (Side::Right, &cell.wall_right),
        ] {
            spawn_wall_bundle(side, wall, parent, meshes, &mesh, &material);
        }

        // Doors
        for (side, door) in [
            (Side::Top, cell.door_top),
            (Side::Bottom, cell.door_bottom),
            (Side::Left, cell.door_left),
            (Side::Right, cell.door_right),
        ] {
            if door {
                spawn_door_bundle(side, parent, &asset_server);
            }
        }

        // Windows
        for (side, window) in [
            (Side::Top, cell.window_top),
            (Side::Bottom, cell.window_bottom),
            (Side::Left, cell.window_left),
            (Side::Right, cell.window_right),
        ] {
            if window {
                spawn_window_bundle(side, parent, &asset_server);
            }
        }

        // Special
        match cell.special {
            CellSpecial::None => (),
            CellSpecial::Chair => {
                spawn_chair_bundle(parent, asset_server);
            }
            CellSpecial::TreasureChest => {
                spawn_treasure_chest_bundle(parent, asset_server, meshes, world_data, &ccm);
            }
            CellSpecial::Staircase => spawn_staircase_bundle(parent, meshes),
            CellSpecial::Stairs => spawn_stairs_bundle(parent, meshes),
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
