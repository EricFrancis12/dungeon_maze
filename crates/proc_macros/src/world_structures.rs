use dungeon_maze_common::world::{
    world_structure::{WorldStructure, WorldStructureName},
    Chunk,
};
use proc_macro::TokenStream;
use std::{
    collections::HashMap,
    fs::{exists, read_to_string},
    path::Path,
};
use strum::IntoEnumIterator;

const WORLD_STRUCTURES_DIR_PATH: &str = "assets/world_structures";

fn make_chunk_str(chunk: &Chunk) -> String {
    format!(
        r#"
            dungeon_maze_common::world::Chunk {{
                x: {},
                y: {},
                z: {},
                cells: vec![{}],
                world_structure: dungeon_maze_common::world::world_structure::WorldStructureName::{},
            }}
        "#,
        chunk.x,
        chunk.y,
        chunk.z,
        chunk
            .cells
            .iter()
            .map(|row| {
                let s = row
                    .iter()
                    .map(|c| {
                        format!(
                            r#"
                                dungeon_maze_common::world::Cell {{
                                    wall_top: dungeon_maze_common::world::CellWall::{},
                                    wall_bottom: dungeon_maze_common::world::CellWall::{},
                                    wall_left: dungeon_maze_common::world::CellWall::{},
                                    wall_right:  dungeon_maze_common::world::CellWall::{},
                                    floor: dungeon_maze_common::world::CellWall::{},
                                    ceiling: dungeon_maze_common::world::CellWall::{},
                                    door_top: {},
                                    door_bottom: {},
                                    door_left: {},
                                    door_right: {},
                                    window_top: {},
                                    window_bottom: {},
                                    window_left: {},
                                    window_right: {},
                                    special: dungeon_maze_common::world::CellSpecial::{},
                                }}
                            "#,
                            c.wall_top,
                            c.wall_bottom,
                            c.wall_left,
                            c.wall_right,
                            c.floor,
                            c.ceiling,
                            c.door_top,
                            c.door_bottom,
                            c.door_left,
                            c.door_right,
                            c.window_top,
                            c.window_bottom,
                            c.window_left,
                            c.window_right,
                            c.special,
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(",");

                format!("vec![{}]", s)
            })
            .collect::<Vec<String>>()
            .join(","),
        chunk.world_structure,
    )
}

pub fn parse_world_structures(_: TokenStream) -> TokenStream {
    assert_eq!(exists(WORLD_STRUCTURES_DIR_PATH).unwrap(), true);

    let mut world_structure_strs: HashMap<WorldStructureName, String> = HashMap::new();
    let mut chunk_strs: HashMap<WorldStructureName, String> = HashMap::new();

    for wsn in WorldStructureName::iter() {
        let path = format!("{}/{}.json", WORLD_STRUCTURES_DIR_PATH, wsn);

        if !exists(&path).unwrap() {
            let do_alt_insert = |hm: &mut HashMap<WorldStructureName, String>| {
                hm.insert(
                    wsn.clone(),
                    String::from(format!(
                        r#"panic!("WorldStructureName::{} was not defined at `{}` during compilation")"#,
                        wsn, path
                    )),
                );
            };

            do_alt_insert(&mut world_structure_strs);
            do_alt_insert(&mut chunk_strs);
            continue;
        }

        let ws = serde_json::from_str::<WorldStructure>(&read_to_string(&path).unwrap()).unwrap();

        let file_name = Path::new(&path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap()
            .to_string();

        let origin_chunk = dungeon_maze_common::utils::must_find_exactly_one(&ws.chunks, |ch| {
            ch.world_structure.to_string() == file_name
        });

        let s = ws
            .chunks
            .iter()
            .map(make_chunk_str)
            .collect::<Vec<String>>()
            .join(",");

        world_structure_strs.insert(wsn.clone(), format!("vec![{}]", s));
        chunk_strs.insert(wsn, make_chunk_str(&origin_chunk));
    }

    let make_match_arms = |hm: &HashMap<WorldStructureName, String>| -> String {
        hm.iter()
            .map(|(n, s)| {
                format!(
                    "dungeon_maze_common::world::world_structure::WorldStructureName::{} => {}",
                    n, s
                )
            })
            .collect::<Vec<String>>()
            .join(",")
    };

    format!(
        r#"
            fn gen_chunks(
                wsn: &dungeon_maze_common::world::world_structure::WorldStructureName,
                x: i64,
                y: i64,
                z: i64,
            ) -> Vec<dungeon_maze_common::world::Chunk> {{
                let mut chunks = match wsn {{
                    {}
                }};
                for chunk in chunks.iter_mut() {{
                    chunk.x += x;
                    chunk.y += y;
                    chunk.z += z;
                }}
                chunks
            }}

            fn gen_origin_chunk(
                wsn: &dungeon_maze_common::world::world_structure::WorldStructureName,
                x: i64,
                y: i64,
                z: i64,
            ) -> dungeon_maze_common::world::Chunk {{
                let mut chunk = match wsn {{
                    {}
                }};
                chunk.x += x;
                chunk.y += y;
                chunk.z += z;
                chunk
            }}
        "#,
        make_match_arms(&world_structure_strs),
        make_match_arms(&chunk_strs),
    )
    .parse()
    .unwrap()
}
