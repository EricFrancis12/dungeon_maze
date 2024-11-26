use crate::{
    error::Error,
    inventory::{Inventory, InventoryChanged},
    settings::{GameSettings, GameSettingsChanged},
};

use bevy::prelude::*;
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    path::PathBuf,
};

const DATA_DIR_NAME: &str = "backrooms-maze";
const SAVE_DIR_NAME: &str = "saves";
const SAVE_FILE_NAME: &str = "backrooms-maze-save.json";

pub struct GameSavePlugin;

impl Plugin for GameSavePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_save_data)
            .add_systems(Update, save_game_automatically);
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct GameSave {
    pub game_settings: GameSettings,
    pub inventory: Inventory,
}

#[derive(Default, Deserialize, Serialize)]
pub struct GameSaveRead {
    pub game_settings: Option<GameSettings>,
    pub inventory: Option<Inventory>,
}

fn load_save_data(mut commands: Commands, mut next_game_settings: ResMut<NextState<GameSettings>>) {
    let game_save = read_game_save().unwrap();
    next_game_settings.set(game_save.game_settings);
    commands.insert_resource(game_save.inventory);
}

macro_rules! save_once_and_return {
    ($event_reader:ident, $do_save:expr) => {
        for _ in $event_reader.read() {
            $do_save();
            $event_reader.clear();
            return;
        }
    };
}

fn save_game_automatically(
    mut gs_event_reader: EventReader<GameSettingsChanged>,
    mut inv_event_reader: EventReader<InventoryChanged>,
    game_settings: Res<State<GameSettings>>,
    inventory: Res<Inventory>,
) {
    let do_save = || {
        write_game_save(GameSave {
            game_settings: game_settings.clone(),
            inventory: inventory.clone(),
        })
        .unwrap();
    };

    save_once_and_return!(gs_event_reader, do_save);
    save_once_and_return!(inv_event_reader, do_save);
}

fn read_game_save() -> Result<GameSave, Error> {
    let save_file_path = get_save_file_path(SAVE_FILE_NAME);
    if !fs::exists(get_data_dir_path())? || !fs::exists(&save_file_path)? {
        return Ok(GameSave::default());
    }

    let file = fs::File::open(save_file_path)?;

    match serde_json::from_reader::<File, GameSaveRead>(file) {
        Ok(g) => Ok(GameSave {
            game_settings: g.game_settings.unwrap_or_default(),
            inventory: g.inventory.unwrap_or_default(),
        }),
        Err(err) => return Err(Error::loading(err)),
    }
}

fn write_game_save(game_save: GameSave) -> Result<(), Error> {
    let data_dir_path = get_data_dir_path();
    if !fs::exists(&data_dir_path)? {
        fs::create_dir(data_dir_path)?;
    }

    let save_dir_path = get_save_dir_path();
    if !fs::exists(&save_dir_path)? {
        fs::create_dir(save_dir_path)?;
    }

    let file = fs::File::create(get_save_file_path(SAVE_FILE_NAME))?;
    match serde_json::to_writer(file, &game_save) {
        Ok(_) => Ok(()),
        Err(err) => Err(Error::saving(err)),
    }
}

fn get_data_dir_path() -> PathBuf {
    AppDirs::new(Some(DATA_DIR_NAME), true).unwrap().data_dir
}

fn get_save_dir_path() -> PathBuf {
    get_data_dir_path().join(SAVE_DIR_NAME)
}

fn get_save_file_path(save_file_name: impl Into<String>) -> PathBuf {
    get_save_dir_path().join(save_file_name.into())
}
