use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameSettingsChangeRequest>()
            .add_event::<GameSettingsChanged>()
            .init_state::<GameSettings>()
            .add_systems(Update, handle_game_settings_change);
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize, States)]
pub struct GameSettings {
    pub chunk_render_dist: ChunkRenderDist,
}

#[derive(Event)]
pub struct GameSettingsChangeRequest {
    pub value: GameSettings,
}

#[derive(Event)]
pub struct GameSettingsChanged;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ChunkRenderDist(pub u32, pub u32, pub u32);

impl Default for ChunkRenderDist {
    fn default() -> Self {
        Self(1, 1, 1)
    }
}

fn handle_game_settings_change(
    mut event_reader: EventReader<GameSettingsChangeRequest>,
    mut event_writer: EventWriter<GameSettingsChanged>,
    mut next_game_settings: ResMut<NextState<GameSettings>>,
) {
    for event in event_reader.read() {
        next_game_settings.set(event.value);
        event_writer.send(GameSettingsChanged);
    }
}
