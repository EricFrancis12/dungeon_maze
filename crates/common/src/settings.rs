use bevy::prelude::{Event, States};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize, States)]
pub struct GameSettings {
    pub chunk_render_dist: ChunkRenderDist,
}

#[derive(Event)]
pub struct RenderDistChanged;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ChunkRenderDist(pub u32, pub u32, pub u32);

impl Default for ChunkRenderDist {
    fn default() -> Self {
        Self(1, 1, 1)
    }
}
