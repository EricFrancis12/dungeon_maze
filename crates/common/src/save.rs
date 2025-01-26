use crate::{inventory::Inventory, settings::GameSettings, world::data::WorldData};
use bevy::prelude::Event;
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct GameSave {
    pub game_settings: GameSettings,
    pub inventory: Inventory,
    pub world_data: WorldData,
}

#[derive(Default, Deserialize, Serialize)]
pub struct GameSaveRead {
    pub game_settings: Option<GameSettings>,
    pub inventory: Option<Inventory>,
    pub world_data: Option<WorldData>,
}

#[derive(Event)]
pub struct WorldDataChanged;
