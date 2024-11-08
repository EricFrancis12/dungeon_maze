use crate::world::{ActiveChunk, ActiveChunkChangeRequest};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameSettingsChangeRequest>()
            .add_event::<GameSettingsChanged>()
            .init_state::<GameSettings>()
            .add_systems(Update, (change_game_settings, handle_game_settings_change));
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize, States)]
pub struct GameSettings {
    pub chunk_render_dist: ChunkRenderDist,
}

#[derive(Event)]
struct GameSettingsChangeRequest {
    value: GameSettings,
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

fn change_game_settings(
    mut acc_event_writer: EventWriter<ActiveChunkChangeRequest>,
    mut gs_event_writer: EventWriter<GameSettingsChangeRequest>,
    active_chunk: Res<State<ActiveChunk>>,
    game_settings: Res<State<GameSettings>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let dist = if keys.just_released(KeyCode::Numpad0) {
        0
    } else if keys.just_released(KeyCode::Numpad1) {
        1
    } else if keys.just_released(KeyCode::Numpad2) {
        2
    } else if keys.just_released(KeyCode::Numpad3) {
        3
    } else {
        return;
    };

    let mut new_game_settings = game_settings.clone();
    new_game_settings.chunk_render_dist = ChunkRenderDist(dist, dist, dist);

    gs_event_writer.send(GameSettingsChangeRequest {
        value: new_game_settings,
    });

    acc_event_writer.send(ActiveChunkChangeRequest {
        value: active_chunk.clone(),
    });
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
