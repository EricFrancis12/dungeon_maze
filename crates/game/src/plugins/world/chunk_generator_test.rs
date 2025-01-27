use crate::plugins::world::chunk_generator::ChunkGenerator;
use dungeon_maze_common::world::world_structure::WorldStructureName;
use strum::IntoEnumIterator;

#[test]
fn test_world_structure_gen_origin_chunk_no_panic() {
    for x in -20..20 {
        for y in -20..20 {
            for z in -20..20 {
                for ws in WorldStructureName::iter() {
                    ws.gen_origin_chunk(x, y, z);
                }
            }
        }
    }
}

#[test]
fn test_world_structure_gen_chunks_no_panic() {
    for x in -20..20 {
        for y in -20..20 {
            for z in -20..20 {
                for ws in WorldStructureName::iter() {
                    ws.gen_chunks(x, y, z);
                }
            }
        }
    }
}
