pub mod cell;
pub mod chunk;
pub mod door;
pub mod item;
pub mod special;
pub mod wall;
pub mod window;

use bevy::{ecs::system::EntityCommands, prelude::*};

pub const WALL_THICKNESS: f32 = 0.1;

pub trait EntitySpawner {
    fn spawn(&mut self, entity: impl Bundle) -> EntityCommands;
}

impl EntitySpawner for Commands<'_, '_> {
    fn spawn(&mut self, entity: impl Bundle) -> EntityCommands {
        self.spawn(entity)
    }
}

impl EntitySpawner for ChildBuilder<'_> {
    fn spawn(&mut self, entity: impl Bundle) -> EntityCommands {
        self.spawn(entity)
    }
}
