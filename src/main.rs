mod animation;
mod camera;
mod error;
mod interaction;
mod inventory;
mod menu;
mod player;
mod save;
mod settings;
mod utils;
mod world;

#[cfg(test)]
mod chunk_test;

#[cfg(debug_assertions)]
mod debug;

use animation::AnimationPlugin;
use camera::CameraPlugin;
use interaction::InteractionPlugin;
use inventory::InventoryPlugin;
use menu::MenuPlugin;
use player::PlayerPlugin;
use save::GameSavePlugin;
use settings::SettingsPlugin;
use world::{WorldPlugin, CELL_SIZE, CHUNK_SIZE};

#[cfg(debug_assertions)]
use debug::DebugPlugin;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_text_popup::TextPopupPlugin;

pub const SEED: u32 = 1234;

fn main() {
    assert_eq!(
        CHUNK_SIZE % CELL_SIZE,
        0.0,
        "expected chunk size ({}) to be divisible by cell size ({})",
        CHUNK_SIZE,
        CELL_SIZE,
    );

    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        RapierPhysicsPlugin::<NoUserData>::default(),
        TextPopupPlugin,
        MenuPlugin,
        PlayerPlugin,
        AnimationPlugin,
        InteractionPlugin,
        SettingsPlugin,
        CameraPlugin,
        InventoryPlugin,
        GameSavePlugin,
        WorldPlugin,
        #[cfg(debug_assertions)]
        DebugPlugin,
    ));

    app.run();
}
