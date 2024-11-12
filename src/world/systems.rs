use super::{
    bundle::chunk::spawn_new_chunk_bundle, make_nei_chunks_xyz, ActiveChunk,
    ActiveChunkChangeRequest, AssetLib, ChunkMarker, CyclicTransform, CELL_SIZE, CHUNK_SIZE,
};
use crate::{
    interaction::{Interactable, PendingInteractionExecuted},
    player::Player,
    settings::GameSettings,
};

use bevy::prelude::*;
use std::collections::HashSet;

pub fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let asset_lib = AssetLib {
        meshes: vec![asset_server.load("meshes/wall_with_door_gap.glb#Mesh0/Primitive0")],
        models: vec![
            // TODO: ...
        ],
    };
    commands.insert_resource(asset_lib);
}

pub fn spawn_initial_chunks(
    mut commands: Commands,
    active_chunk: Res<State<ActiveChunk>>,
    game_settings: Res<State<GameSettings>>,
    asset_server: Res<AssetServer>,
    asset_lib: Res<AssetLib>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let render_dist = game_settings.chunk_render_dist;
    let chunks = make_nei_chunks_xyz(
        (active_chunk.0, active_chunk.1, active_chunk.2),
        render_dist.0,
        render_dist.1,
        render_dist.2,
    );
    for xyz in chunks {
        spawn_new_chunk_bundle(
            xyz,
            &mut commands,
            &asset_server,
            &asset_lib,
            &mut meshes,
            &mut materials,
        );
    }
}

pub fn manage_active_chunk(
    mut event_writer: EventWriter<ActiveChunkChangeRequest>,
    player_query: Query<&GlobalTransform, With<Player>>,
    active_chunk: Res<State<ActiveChunk>>,
) {
    let player_gl_transform = player_query.get_single().expect("Error retrieving player");
    let player_gl_translation = player_gl_transform.translation();

    let mut chunk = active_chunk.clone();
    let half_chunk_size = CHUNK_SIZE / 2.0;
    let half_cell_size = CELL_SIZE / 2.0;

    // x
    let x_chunk_size = active_chunk.0 as f32 * CHUNK_SIZE;
    let x_min_crossed = player_gl_translation.x < x_chunk_size - half_chunk_size;
    let x_max_crossed = player_gl_translation.x > x_chunk_size + half_chunk_size;

    if x_min_crossed {
        chunk.0 -= 1;
    } else if x_max_crossed {
        chunk.0 += 1;
    }

    // y
    let y_chunk_size = active_chunk.1 as f32 * CELL_SIZE;
    let y_min_crossed = player_gl_translation.y < y_chunk_size - half_cell_size;
    let y_max_crossed = player_gl_translation.y > y_chunk_size + half_cell_size;

    if y_min_crossed {
        chunk.1 -= 1;
    } else if y_max_crossed {
        chunk.1 += 1;
    }

    // z
    let z_chunk_size = active_chunk.2 as f32 * CHUNK_SIZE;
    let z_min_crossed = player_gl_translation.z < z_chunk_size - half_chunk_size;
    let z_max_crossed = player_gl_translation.z > z_chunk_size + half_chunk_size;

    if z_min_crossed {
        chunk.2 -= 1;
    } else if z_max_crossed {
        chunk.2 += 1;
    }

    if x_min_crossed
        || x_max_crossed
        || y_min_crossed
        || y_max_crossed
        || z_min_crossed
        || z_max_crossed
    {
        event_writer.send(ActiveChunkChangeRequest { value: chunk });
    }
}

pub fn handle_active_chunk_change(
    mut commands: Commands,
    mut event_reader: EventReader<ActiveChunkChangeRequest>,
    chunks_query: Query<(Entity, &ChunkMarker)>,
    game_settings: Res<State<GameSettings>>,
    mut next_active_chunk: ResMut<NextState<ActiveChunk>>,
    asset_server: Res<AssetServer>,
    asset_lib: Res<AssetLib>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in event_reader.read() {
        let chunk_xyz = event.value.to_tuple();
        next_active_chunk.set(event.value);

        let rend_dist = game_settings.chunk_render_dist;
        let new_chunks = make_nei_chunks_xyz(chunk_xyz, rend_dist.0, rend_dist.1, rend_dist.2);

        let mut existing_chunks: HashSet<(i64, i64, i64)> = HashSet::new();

        // Despawn chunks that are not in the new chunks
        for (chunk_entity, chunk_marker) in chunks_query.iter() {
            if !new_chunks.contains(&chunk_marker.0) {
                commands.entity(chunk_entity).despawn_recursive();
            }
            existing_chunks.insert(chunk_marker.0);
        }

        // Spawn new chunks that are not currently existing
        for (x, y, z) in new_chunks {
            if !existing_chunks.contains(&(x, y, z)) {
                spawn_new_chunk_bundle(
                    (x, y, z),
                    &mut commands,
                    &asset_server,
                    &asset_lib,
                    &mut meshes,
                    &mut materials,
                );
            }
        }
    }
}

pub fn handle_cyclic_interaction_transforms(
    mut event_reader: EventReader<PendingInteractionExecuted>,
    mut cyclic_transforms_query: Query<(Entity, &mut CyclicTransform), With<Interactable>>,
) {
    for event in event_reader.read() {
        for (entity, mut cyclic_transform) in cyclic_transforms_query.iter_mut() {
            if entity == event.0 {
                cyclic_transform.cycle();
            }
        }
    }
}

pub fn advance_cyclic_transforms(
    mut cyclic_transforms_query: Query<(&mut CyclicTransform, &mut Transform)>,
) {
    for (mut ct, mut transform) in cyclic_transforms_query.iter_mut() {
        if let Some(t) = ct.tick() {
            *transform = t.clone();
        }
    }
}
