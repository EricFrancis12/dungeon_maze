use bevy::prelude::*;
use dungeon_maze_common::{
    error::Error,
    inventory::{Inventory, InventoryChanged},
    save::{GameSave, GameSaveRead, WorldDataChanged},
    settings::GameSettings,
    world::data::WorldData,
};
use platform_dirs::AppDirs;
use std::{
    fs::{self, File},
    path::PathBuf,
};

const DATA_DIR_NAME: &str = "dungeon_maze";
const SAVE_DIR_NAME: &str = "saves";
const SAVE_FILE_NAME: &str = "dungeon_maze_save.json";

pub struct GameSavePlugin;

impl Plugin for GameSavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldData>()
            .add_event::<WorldDataChanged>()
            .add_systems(Startup, load_save_data)
            .add_systems(Update, save_game_automatically);
    }
}

fn load_save_data(mut commands: Commands, mut next_game_settings: ResMut<NextState<GameSettings>>) {
    let game_save = read_game_save().unwrap_or_default();
    next_game_settings.set(game_save.game_settings);
    commands.insert_resource(game_save.inventory);
    commands.insert_resource(game_save.world_data);
}

fn save_game_automatically(
    gs_event_reader: EventReader<StateTransitionEvent<GameSettings>>,
    inv_event_reader: EventReader<InventoryChanged>,
    wd_event_reader: EventReader<WorldDataChanged>,
    game_settings: Res<State<GameSettings>>,
    inventory: Res<Inventory>,
    world_data: Res<WorldData>,
) {
    if !gs_event_reader.is_empty() || !inv_event_reader.is_empty() || !wd_event_reader.is_empty() {
        write_game_save(GameSave {
            game_settings: game_settings.clone(),
            inventory: inventory.clone(),
            world_data: world_data.clone(),
        })
        .unwrap();
    }
}

fn read_game_save() -> Result<GameSave, Error> {
    let save_file_path = get_save_file_path(SAVE_FILE_NAME);
    if !fs::exists(get_data_dir_path())? || !fs::exists(&save_file_path)? {
        return Ok(GameSave::default());
    }

    let file = fs::File::open(save_file_path)?;

    match serde_json::from_reader::<File, GameSaveRead>(file) {
        Ok(r) => Ok(GameSave {
            game_settings: r.game_settings.unwrap_or_default(),
            inventory: r.inventory.unwrap_or_default(),
            world_data: r.world_data.unwrap_or_default(),
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
