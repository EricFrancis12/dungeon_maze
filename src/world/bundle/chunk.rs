use super::{
    super::{chunk_from_xyz_seed, AssetLib, ChunkCellMarker, ChunkMarker, CELL_SIZE, CHUNK_SIZE},
    cell::spawn_cell_bundle,
};
use crate::SEED;

use bevy::prelude::*;

pub fn spawn_chunk_bundle(
    (chunk_x, chunk_y, chunk_z): (i64, i64, i64),
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    asset_lib: &Res<AssetLib>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
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

    commands.spawn(chunk_bundle).with_children(|parent| {
        let chunk = chunk_from_xyz_seed(SEED, chunk_x, chunk_y, chunk_z);

        for (x, row) in chunk.cells.iter().enumerate() {
            for (z, cell) in row.iter().enumerate() {
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
                    asset_lib,
                    meshes,
                    materials,
                );
            }
        }
    });
}
