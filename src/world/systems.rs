use super::{
    bundle::{chunk::spawn_chunk_bundle, special::OCItemContainer},
    data::WorldData,
    make_nei_chunks_xyz, ActiveChunk, AssetLib, ChunkCellMarker, ChunkMarker, CyclicTransform,
};
use crate::{
    interaction::{Interactable, PendingInteractionExecuted},
    inventory::Item,
    player::Player,
    settings::{GameSettings, RenderDistChanged},
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
    mut commands: Commands,
    active_chunk: Res<State<ActiveChunk>>,
    game_settings: Res<State<GameSettings>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    world_data: Res<WorldData>,
) {
    let render_dist = game_settings.chunk_render_dist;
    let chunks = make_nei_chunks_xyz(
        (active_chunk.0, active_chunk.1, active_chunk.2),
        render_dist.0,
        render_dist.1,
        render_dist.2,
    );
    for xyz in chunks {
        spawn_chunk_bundle(
            xyz,
            &mut commands,
            &asset_server,
            &mut meshes,
            &mut materials,
            &world_data,
        );
    }
}

pub fn manage_active_chunk(
    player_query: Query<&GlobalTransform, With<Player>>,
    active_chunk: Res<State<ActiveChunk>>,
    mut next_active_chunk: ResMut<NextState<ActiveChunk>>,
) {
    let gt = player_query.get_single().expect("Error retrieving player");
    let (x, y, z) = ChunkCellMarker::from_global_transform(gt).chunk_xyz();

    if x != active_chunk.0 || y != active_chunk.1 || z != active_chunk.2 {
        next_active_chunk.set(ActiveChunk(x, y, z));
    }
}

pub fn update_spawned_chunks(
    mut commands: Commands,
    ac_event_reader: EventReader<StateTransitionEvent<ActiveChunk>>,
    rd_event_reader: EventReader<RenderDistChanged>,
    chunks_query: Query<(Entity, &ChunkMarker)>,
    active_chunk: Res<State<ActiveChunk>>,
    game_settings: Res<State<GameSettings>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    world_data: Res<WorldData>,
) {
    if !ac_event_reader.is_empty() || !rd_event_reader.is_empty() {
        let rend_dist = game_settings.chunk_render_dist;
        let new_chunks = make_nei_chunks_xyz(
            active_chunk.to_tuple(),
            rend_dist.0,
            rend_dist.1,
            rend_dist.2,
        );

        let mut existing_chunks: HashSet<(i64, i64, i64)> = HashSet::new();

        // Despawn chunks that are not among new chunks
        for (chunk_entity, chunk_marker) in chunks_query.iter() {
            if !new_chunks.contains(&chunk_marker.0) {
                commands.entity(chunk_entity).despawn_recursive();
            }
            existing_chunks.insert(chunk_marker.0);
        }

        // Spawn new chunks that do not currently exist
        for (x, y, z) in new_chunks {
            if !existing_chunks.contains(&(x, y, z)) {
                spawn_chunk_bundle(
                    (x, y, z),
                    &mut commands,
                    &asset_server,
                    &mut meshes,
                    &mut materials,
                    &world_data,
                );
            }
        }
    };
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

pub fn activate_items_inside_containers(
    mut commands: Commands,
    mut event_reader: EventReader<PendingInteractionExecuted>,
    containers_query: Query<(Entity, &Children), With<OCItemContainer>>,
    interactable_item_query: Query<&Item, With<Interactable>>,
    noninteractable_item_query: Query<&Item, Without<Interactable>>,
) {
    for event in event_reader.read() {
        for (treasure_chest_entity, children) in containers_query.iter() {
            if treasure_chest_entity == event.0 {
                for child in children.iter() {
                    if noninteractable_item_query.get(*child).is_ok() {
                        // If Interactable component is not present, insert one
                        commands.entity(*child).insert(Item::interactable());
                    } else if interactable_item_query.get(*child).is_ok() {
                        // If Interactable component is present, remove it
                        commands.entity(*child).remove::<Interactable>();
                    }
                }

                break;
            }
        }
    }
}
