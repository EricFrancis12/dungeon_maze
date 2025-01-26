use crate::{
    plugins::world::{
        bundle::cell::spawn_cell_bundle,
        {chunk_from_xyz_seed, CELL_SIZE, CHUNK_SIZE},
    },
    SEED,
};
use bevy::prelude::*;
use dungeon_maze_common::world::{data::WorldData, ChunkCellMarker, ChunkMarker, EntitySpawner};

pub fn spawn_chunk_bundle(
    (chunk_x, chunk_y, chunk_z): (i64, i64, i64),
    entity_spawner: &mut impl EntitySpawner,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    world_data: &Res<WorldData>,
) {
    let chunk_bundle = (
        SpatialBundle {
            transform: Transform::from_xyz(
                chunk_x as f32 * CHUNK_SIZE,
                chunk_y as f32 * CELL_SIZE,
                chunk_z as f32 * CHUNK_SIZE,
            ),
            ..default()
        },
        ChunkMarker((chunk_x, chunk_y, chunk_z)),
        Name::new(format!("Chunk_({},{},{})", chunk_x, chunk_y, chunk_z)),
    );

    entity_spawner.spawn(chunk_bundle).with_children(|parent| {
        let chunk = chunk_from_xyz_seed(SEED, chunk_x, chunk_y, chunk_z);

        for (z, row) in chunk.cells.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let ccm = ChunkCellMarker {
                    chunk_x,
                    chunk_y,
                    chunk_z,
                    x,
                    z,
                };

                spawn_cell_bundle(
                    cell,
                    ccm,
                    parent,
                    asset_server,
                    meshes,
                    materials,
                    world_data,
                );
            }
        }
    });
}
