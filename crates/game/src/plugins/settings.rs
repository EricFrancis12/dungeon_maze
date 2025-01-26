use bevy::prelude::*;
use dungeon_maze_common::settings::*;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RenderDistChanged>()
            .init_state::<GameSettings>();
    }
}
