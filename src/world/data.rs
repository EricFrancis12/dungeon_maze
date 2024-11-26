use super::bundle::special::Item;
use crate::{inventory::ItemRemovedFromTreasureChest, save::WorldDataChanged};

use bevy::prelude::*;
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{collections::HashMap, fmt};

macro_rules! serialize_impl {
    ($t:ty, $child_t:ty, $prop_name:ident, $closure:expr) => {
        impl Serialize for $t {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                // Convert the HashMap keys to strings
                let stringified: HashMap<String, &$child_t> =
                    self.$prop_name.iter().map($closure).collect();

                // Serialize the stringified HashMap
                stringified.serialize(serializer)
            }
        }
    };
}

macro_rules! deserialize_impl {
    ($t:ty, $child_t:ty, $prop_name:ident, $key_len:expr, $key_parser:expr) => {
        impl<'de> Deserialize<'de> for $t {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct DataVisitor;

                impl<'de> Visitor<'de> for DataVisitor {
                    type Value = $t;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str(concat!(
                            "a map with stringified tuple keys and ",
                            stringify!($child_t),
                            " values"
                        ))
                    }

                    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
                    where
                        M: MapAccess<'de>,
                    {
                        let mut $prop_name: HashMap<_, $child_t> = HashMap::new();

                        while let Some((key, value)) = map.next_entry::<String, $child_t>()? {
                            let parts: Vec<&str> = key.split(',').collect();
                            if parts.len() != $key_len {
                                return Err(de::Error::custom(format!(
                                    "Invalid key format: {}",
                                    key
                                )));
                            }

                            let parsed_key = $key_parser(&parts).map_err(de::Error::custom)?;
                            $prop_name.insert(parsed_key, value);
                        }

                        Ok(Self::Value { $prop_name })
                    }
                }

                deserializer.deserialize_map(DataVisitor)
            }
        }
    };
}

#[derive(Clone, Debug, Default, Resource)]
pub struct WorldData {
    chunks: HashMap<(i64, i64, i64), ChunkData>,
}

serialize_impl!(WorldData, ChunkData, chunks, |(&(x, y, z), value)| (
    format!("{},{},{}", x, y, z),
    value
));

deserialize_impl!(WorldData, ChunkData, chunks, 3, parse_3d_key);

fn parse_3d_key(parts: &[&str]) -> Result<(i64, i64, i64), String> {
    Ok((
        parts[0]
            .parse::<i64>()
            .map_err(|_| format!("Invalid x: {}", parts[0]))?,
        parts[1]
            .parse::<i64>()
            .map_err(|_| format!("Invalid y: {}", parts[1]))?,
        parts[2]
            .parse::<i64>()
            .map_err(|_| format!("Invalid z: {}", parts[2]))?,
    ))
}

impl WorldData {
    pub fn at_chunk(&self, xyz: (i64, i64, i64)) -> Option<&ChunkData> {
        self.chunks.get(&xyz)
    }

    fn _at_chunk_mut(&mut self, xyz: (i64, i64, i64)) -> Option<&mut ChunkData> {
        self.chunks.get_mut(&xyz)
    }

    pub fn at_cell(&self, xyz: (i64, i64, i64), xz: (usize, usize)) -> Option<&CellData> {
        if let Some(chunk_data) = self.at_chunk(xyz) {
            return chunk_data.at_cell(xz);
        }
        None
    }

    fn _at_cell_mut(&mut self, xyz: (i64, i64, i64), xz: (usize, usize)) -> Option<&mut CellData> {
        if let Some(chunk_data) = self._at_chunk_mut(xyz) {
            return chunk_data._at_cell_mut(xz);
        }
        None
    }

    pub fn at_chunk_or_create_mut(&mut self, xyz: (i64, i64, i64)) -> &mut ChunkData {
        if !self.chunks.contains_key(&xyz) {
            self.chunks.insert(xyz.clone(), ChunkData::default());
        }
        self.chunks.get_mut(&xyz).unwrap()
    }

    pub fn at_cell_or_create_mut(
        &mut self,
        xyz: (i64, i64, i64),
        xz: (usize, usize),
    ) -> &mut CellData {
        let chunk_data = self.at_chunk_or_create_mut(xyz);
        if !chunk_data.cells.contains_key(&xz) {
            chunk_data.cells.insert(xz.clone(), CellData::default());
        }
        chunk_data.cells.get_mut(&xz).unwrap()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ChunkData {
    cells: HashMap<(usize, usize), CellData>,
}

serialize_impl!(ChunkData, CellData, cells, |(&(x, z), value)| (
    format!("{},{}", x, z),
    value
));

deserialize_impl!(ChunkData, CellData, cells, 2, parse_2d_key);

fn parse_2d_key(parts: &[&str]) -> Result<(usize, usize), String> {
    Ok((
        parts[0]
            .parse::<usize>()
            .map_err(|_| format!("Invalid x: {}", parts[0]))?,
        parts[1]
            .parse::<usize>()
            .map_err(|_| format!("Invalid z: {}", parts[1]))?,
    ))
}

impl ChunkData {
    pub fn at_cell(&self, xz: (usize, usize)) -> Option<&CellData> {
        if let Some(cell_data) = self.cells.get(&xz) {
            return Some(cell_data);
        }
        None
    }

    fn _at_cell_mut(&mut self, xz: (usize, usize)) -> Option<&mut CellData> {
        if let Some(cell_data) = self.cells.get_mut(&xz) {
            return Some(cell_data);
        }
        None
    }

    fn _at_cell_or_create_mut(&mut self, xz: (usize, usize)) -> &mut CellData {
        if !self.cells.contains_key(&xz) {
            self.cells.insert(xz.clone(), CellData::default());
        }
        self.cells.get_mut(&xz).unwrap()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CellData {
    pub treasure_chest_data: TreasureChestData,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TreasureChestData {
    pub item: Option<Item>,
}

pub fn update_world_data_treasure_chests(
    mut commands: Commands,
    mut event_reader: EventReader<ItemRemovedFromTreasureChest>,
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
