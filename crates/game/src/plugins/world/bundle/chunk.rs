use crate::{
    plugins::world::{
        bundle::cell::spawn_cell_bundle,
        {chunk_from_xyz_seed, CELL_SIZE, CHUNK_SIZE},
    },
    SEED,
};
use bevy::prelude::*;
use dungeon_maze_common::world::{
    data::WorldData, Chunk, ChunkCellMarker, ChunkMarker, EntitySpawner,
};

pub fn spawn_chunk_bundle(
    chunk: &Chunk,
    entity_spawner: &mut impl EntitySpawner,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    world_data: &Res<WorldData>,
) {
    let chunk_bundle = (
        SpatialBundle {
            transform: Transform::from_xyz(
                chunk.x as f32 * CHUNK_SIZE,
                chunk.y as f32 * CELL_SIZE,
                chunk.z as f32 * CHUNK_SIZE,
            ),
            ..default()
        },
        ChunkMarker((chunk.x, chunk.y, chunk.z)),
        Name::new(format!("Chunk_({},{},{})", chunk.x, chunk.y, chunk.z)),
    );

    entity_spawner.spawn(chunk_bundle).with_children(|parent| {
        for (z, row) in chunk.cells.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let ccm = ChunkCellMarker {
                    chunk_x: chunk.x,
                    chunk_y: chunk.y,
                    chunk_z: chunk.z,
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

pub fn spawn_chunk_bundle_from_xyz_seed(
    (chunk_x, chunk_y, chunk_z): (i64, i64, i64),
    entity_spawner: &mut impl EntitySpawner,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    world_data: &Res<WorldData>,
) {
    let chunk = chunk_from_xyz_seed(SEED, chunk_x, chunk_y, chunk_z);

    spawn_chunk_bundle(
        &chunk,
        entity_spawner,
        asset_server,
        meshes,
        materials,
        world_data,
    );
}
