pub mod bundle;
pub mod chunk_generator;

use crate::plugins::world::{
    bundle::{chunk::spawn_chunk_bundle_from_xyz_seed, item::spawn_item_bundle},
    chunk_generator::ChunkGenerator,
};
use bevy::prelude::*;
use dungeon_maze_common::{
    interaction::{Interactable, PendingInteractionExecuted},
    inventory::{item::Item, ItemRemovedFromOCItemContainer, PlayerDroppedItem},
    player::Player,
    save::WorldDataChanged,
    settings::{GameSettings, RenderDistChanged},
    utils::{
        maze::maze_from_rng,
        rng::{rng_from_str, rng_from_xyz_seed},
    },
    world::{
        data::WorldData, world_structure::WorldStructureName, ActiveChunk, CellSpecial, CellWall,
        Chunk, ChunkCellMarker, ChunkMarker, CyclicTransform, OCItemContainer,
    },
};
use rand::{rngs::StdRng, Rng};
use std::collections::HashSet;
use strum::IntoEnumIterator;

pub const CELL_SIZE: f32 = 4.0;
pub const CHUNK_SIZE: f32 = 16.0;
pub const GRID_SIZE: usize = (CHUNK_SIZE / CELL_SIZE) as usize;

const WALL_BREAK_PROB: f64 = 0.2;
const WORLD_STRUCTURE_GEN_PROB: f64 = 0.18;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ActiveChunk>()
            .add_systems(Startup, spawn_initial_chunks)
            .add_systems(
                Update,
                (
                    manage_active_chunk,
                    update_spawned_chunks,
                    advance_cyclic_transforms,
                    handle_cyclic_transform_interactions.after(advance_cyclic_transforms),
                    activate_items_inside_containers.after(advance_cyclic_transforms),
                    remove_item_from_oc_item_containers,
                    spawn_dropped_item,
                ),
            );
    }
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
        spawn_chunk_bundle_from_xyz_seed(
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
    let (x, y, z) = ChunkCellMarker::from_global_transform(gt, CHUNK_SIZE, CELL_SIZE).chunk_xyz();

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
                spawn_chunk_bundle_from_xyz_seed(
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

pub fn spawn_dropped_item(
    mut commands: Commands,
    mut event_reader: EventReader<PlayerDroppedItem>,
    player_query: Query<&GlobalTransform, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for event in event_reader.read() {
        let player_gl_transform = player_query.get_single().unwrap();
        spawn_item_bundle(
            event.0.clone(),
            &mut commands,
            &mut meshes,
            Some(Transform::from_translation(
                player_gl_transform.translation(),
            )),
            true,
            true,
            true,
        );
        break;
    }
}

pub fn remove_item_from_oc_item_containers(
    mut commands: Commands,
    mut event_reader: EventReader<ItemRemovedFromOCItemContainer>,
    mut event_writer: EventWriter<WorldDataChanged>,
    world_data: ResMut<WorldData>,
) {
    for event in event_reader.read() {
        let mut new_world_data = world_data.clone();
        let cell_data =
            new_world_data.at_cell_or_create_mut(event.ccm.chunk_xyz(), event.ccm.cell_xz());
        cell_data.treasure_chest_data.item = None;
        commands.insert_resource(new_world_data);
        event_writer.send(WorldDataChanged);
    }
}

pub fn chunk_from_xyz_seed(seed: u32, x: i64, y: i64, z: i64) -> Chunk {
    let mut rng = rng_from_xyz_seed(seed, x, y, z);

    if chunk_has_world_structure(seed, x, y, z) {
        return WorldStructureName::choose(&mut rng).gen_origin_chunk(x, y, z);
    }

    let mut cells = maze_from_rng(&mut rng, GRID_SIZE, GRID_SIZE);

    let h = GRID_SIZE / 2;
    let w = GRID_SIZE / 2;

    // left and right walls
    cells[h][0].wall_left = CellWall::None;
    cells[h][GRID_SIZE - 1].wall_right = CellWall::None;

    // top and bottom walls
    cells[0][w].wall_top = CellWall::None;
    cells[GRID_SIZE - 1][w].wall_bottom = CellWall::None;

    // ceiling and floor (y axis)
    for h in 0..GRID_SIZE {
        for w in 0..GRID_SIZE {
            let mut y_minus_1_rng = rng_from_str(seed_str_from_neis(
                seed,
                (x, y - 1, z, w, h),
                (x, y, z, w, h),
            ));
            if y_minus_1_rng.gen_bool(WALL_BREAK_PROB) {
                cells[h][w].floor = CellWall::None;
            }

            let mut y_plus_1_rng = rng_from_str(seed_str_from_neis(
                seed,
                (x, y, z, w, h),
                (x, y + 1, z, w, h),
            ));
            if y_plus_1_rng.gen_bool(WALL_BREAK_PROB) {
                cells[h][w].ceiling = CellWall::None;
            }
        }
    }

    let mut floored_cells: Vec<(usize, usize)> = Vec::new();
    for h in 0..GRID_SIZE {
        for w in 0..GRID_SIZE {
            if cells[h][w].floor == CellWall::Solid {
                floored_cells.push((w, h));
            }
        }
    }

    let rand_floored_cell = |r: &mut StdRng, fc: &mut Vec<(usize, usize)>| {
        let i = r.gen_range(0..fc.len());
        let (w, h) = fc[i];
        fc.splice(i..i + 1, []);
        (w, h)
    };

    for spec in CellSpecial::iter() {
        if floored_cells.is_empty() {
            break;
        }

        if rng.gen_bool(spec.spawn_prob()) {
            let (w, h) = rand_floored_cell(&mut rng, &mut floored_cells);
            cells[h][w].special = spec;
        }
    }

    let search_radius = WorldStructureName::max_radius() as i64 - 1;
    if search_radius > 0 {
        // Reach out on all sides equal to max world structure radius
        // to see if any surrounding chunks have world structures.
        // The number of chunks to check in any one direction
        // is equal to the max world structure radius minus 1.

        let x_min = x - search_radius;
        let x_max = x + search_radius;
        let y_min = y - search_radius;
        let y_max = y + search_radius;
        let z_min = z - search_radius;
        let z_max = z + search_radius;

        for _x in x_min..=x_max {
            for _y in y_min..=y_max {
                for _z in z_min..=z_max {
                    if _x == x && _y == y && _z == z {
                        continue;
                    }

                    if chunk_has_world_structure(seed, _x, _y, _z) {
                        let ws_chunk = chunk_from_xyz_seed(seed, _x, _y, _z);
                        let ws_chunks = ws_chunk.world_structure.gen_chunks(_x, _y, _z);

                        if let Some(ch) =
                            ws_chunks.iter().find(|c| c.x == x && c.y == y && c.z == z)
                        {
                            return ch.clone();
                        }
                    }
                }
            }
        }
    }

    Chunk {
        x,
        y,
        z,
        cells,
        world_structure: WorldStructureName::None,
    }
}

pub fn make_nei_chunks_xyz(
    chunk: (i64, i64, i64),
    x_rend_dist: u32,
    y_rend_dist: u32,
    z_rend_dist: u32,
) -> Vec<(i64, i64, i64)> {
    if x_rend_dist == 0 || y_rend_dist == 0 || z_rend_dist == 0 {
        return Vec::new();
    }

    let (x, y, z) = chunk;

    let x_r = x_rend_dist as i64 - 1;
    let y_r = y_rend_dist as i64 - 1;
    let z_r = z_rend_dist as i64 - 1;

    (x - x_r..=x + x_r)
        .flat_map(|i| {
            (y - y_r..=y + y_r).flat_map(move |j| (z - z_r..=z + z_r).map(move |k| (i, j, k)))
        })
        .collect()
}

fn chunk_has_world_structure(seed: u32, x: i64, y: i64, z: i64) -> bool {
    let mut rng = rng_from_xyz_seed(seed, x, y, z);
    rng.gen_bool(WORLD_STRUCTURE_GEN_PROB)
}

fn seed_str_from_neis(
    seed: u32,
    greater_nei: (i64, i64, i64, usize, usize),
    less_nei: (i64, i64, i64, usize, usize),
) -> String {
    let (g_chunk_x, g_chunk_y, g_chunk_z, g_x, g_z) = greater_nei;
    let (l_chunk_x, l_chunk_y, l_chunk_z, l_x, l_z) = less_nei;
    format!(
        "{}-{}_{}_{}_{}_{}-{}_{}_{}_{}_{}",
        seed, g_chunk_x, g_chunk_y, g_chunk_z, g_x, g_z, l_chunk_x, l_chunk_y, l_chunk_z, l_x, l_z,
    )
}
