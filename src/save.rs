use crate::error::Error;
use crate::GameSave;

use platform_dirs::AppDirs;
use std::{fs, path::PathBuf};

const DATA_DIR_NAME: &str = "backrooms-maze";
const SAVE_DIR_NAME: &str = "saves";
const SAVE_FILE_NAME: &str = "backrooms-maze-save.json";

pub fn read_game_save() -> Result<GameSave, Error> {
    let save_file_path = get_save_file_path(SAVE_FILE_NAME);
    if !fs::exists(get_data_dir_path())? || !fs::exists(&save_file_path)? {
        return Ok(GameSave::default());
    }

    let file = fs::File::open(save_file_path)?;

    let game_save: GameSave = match serde_json::from_reader(file) {
        Ok(gs) => gs,
        Err(err) => return Err(Error::loading(err)),
    };

    Ok(game_save)
}

pub fn write_game_save(game_save: GameSave) -> Result<(), Error> {
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
