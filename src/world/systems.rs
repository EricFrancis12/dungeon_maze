use super::{
    bundle::chunk::spawn_chunk_bundle, make_nei_chunks_xyz, ActiveChunk, ActiveChunkChangeRequest,
    AssetLib, ChunkDespawnQueue, ChunkMarker, ChunkSpawnQueue, CyclicTransform, CELL_SIZE,
    CHUNK_SIZE,
};
use crate::{
    interaction::{Interactable, PendingInteractionExecuted},
    player::Player,
    settings::GameSettings,
};

use bevy::prelude::*;
use std::collections::HashSet;

pub fn preload_assets(asset_server: Res<AssetServer>, mut asset_lib: ResMut<AssetLib>) {
    let mut meshes: Vec<Handle<Mesh>> = vec![];
    for mesh_path in [
        "meshes/wall_with_door_gap.glb#Mesh0/Primitive0",
        "meshes/wall_with_window_gap.glb#Mesh0/Primitive0",
    ] {
        let h: Handle<Mesh> = asset_server.load(mesh_path);
        meshes.push(h);
    }
    asset_lib.meshes = meshes;
}

pub fn spawn_initial_chunks(
    active_chunk: Res<State<ActiveChunk>>,
    game_settings: Res<State<GameSettings>>,
    mut next_chunk_spawn_queue: ResMut<NextState<ChunkSpawnQueue>>,
) {
    let render_dist = game_settings.chunk_render_dist;
    let chunks = make_nei_chunks_xyz(
        (active_chunk.0, active_chunk.1, active_chunk.2),
        render_dist.0,
        render_dist.1,
        render_dist.2,
    );
    next_chunk_spawn_queue.set(ChunkSpawnQueue(chunks));
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
    mut event_reader: EventReader<ActiveChunkChangeRequest>,
    chunks_query: Query<(Entity, &ChunkMarker)>,
    game_settings: Res<State<GameSettings>>,
    mut next_active_chunk: ResMut<NextState<ActiveChunk>>,
    mut next_chunk_spawn_queue: ResMut<NextState<ChunkSpawnQueue>>,
    mut next_chunk_despawn_queue: ResMut<NextState<ChunkDespawnQueue>>,
) {
    for event in event_reader.read() {
        let chunk_xyz = event.value.to_tuple();
        next_active_chunk.set(event.value);

        let rend_dist = game_settings.chunk_render_dist;
        let new_chunks = make_nei_chunks_xyz(chunk_xyz, rend_dist.0, rend_dist.1, rend_dist.2);

        let mut existing_chunks: HashSet<(i64, i64, i64)> = HashSet::new();

        // Despawn chunks that are not among new chunks
        let mut new_chunk_despawn_queue = HashSet::new();
        for (chunk_entity, chunk_marker) in chunks_query.iter() {
            if !new_chunks.contains(&chunk_marker.0) {
                // commands.entity(chunk_entity).despawn_recursive(); // TODO: ...
                new_chunk_despawn_queue.insert(chunk_entity);
            }
            existing_chunks.insert(chunk_marker.0);
        }

        let v: Vec<Entity> = new_chunk_despawn_queue.iter().map(|e| *e).collect();
        next_chunk_despawn_queue.set(ChunkDespawnQueue(v));

        // Spawn new chunks that do not currently exist
        let mut chunks = Vec::new();
        for (x, y, z) in new_chunks {
            if !existing_chunks.contains(&(x, y, z)) {
                chunks.push((x, y, z));
            }
        }
        next_chunk_spawn_queue.set(ChunkSpawnQueue(chunks));
    }
}

pub fn spawn_chunks_from_queue(
    mut commands: Commands,
    chunk_spawn_queue: Res<State<ChunkSpawnQueue>>,
    mut next_chunk_spawn_queue: ResMut<NextState<ChunkSpawnQueue>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if chunk_spawn_queue.get().0.len() > 0 {
        let mut new_chunk_spawn_queue = chunk_spawn_queue.clone();
        let chunk = new_chunk_spawn_queue.0.pop().unwrap();
        spawn_chunk_bundle(
            chunk,
            &mut commands,
            &asset_server,
            &mut meshes,
            &mut materials,
        );

        next_chunk_spawn_queue.set(new_chunk_spawn_queue);
    }
}

pub fn despawn_chunks_from_queue(
    mut commands: Commands,
    children_query: Query<&Children>,
    chunk_despawn_queue: Res<State<ChunkDespawnQueue>>,
    mut next_chunk_despawn_queue: ResMut<NextState<ChunkDespawnQueue>>,
) {
    if chunk_despawn_queue.get().0.len() > 0 {
        let mut new_chunk_despawn_queue = chunk_despawn_queue.clone();
        let chunk_entity = new_chunk_despawn_queue.0.pop().unwrap();

        if let Ok(children) = children_query.get(chunk_entity) {
            if children.len() > 0 {
                for i in [0, 1] {
                    if let Some(child_entity) = children.get(i) {
                        commands.entity(*child_entity).despawn_recursive();
                    }
                }
                return;
            }
        }

        commands.entity(chunk_entity).despawn_recursive();
        next_chunk_despawn_queue.set(new_chunk_despawn_queue);
    }
}

pub fn handle_cyclic_transform_interactions(
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
