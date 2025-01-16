mod animation;
mod camera;
mod cursor;
mod error;
mod hud;
mod interaction;
mod inventory;
mod menu;
mod meshes;
mod player;
mod save;
mod settings;
mod utils;
mod world;

#[cfg(test)]
mod chunk_test;
#[cfg(test)]
mod inventory_test;

#[cfg(debug_assertions)]
mod debug;

use crate::{
    animation::AnimationPlugin,
    camera::CameraPlugin,
    cursor::CursorPlugin,
    hud::HudPlugin,
    interaction::InteractionPlugin,
    inventory::InventoryPlugin,
    menu::MenuPlugin,
    player::PlayerPlugin,
    save::GameSavePlugin,
    settings::SettingsPlugin,
    world::{WorldPlugin, CELL_SIZE, CHUNK_SIZE},
};
use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_rapier3d::prelude::*;
use bevy_text_popup::TextPopupPlugin;

#[cfg(debug_assertions)]
use debug::DebugPlugin;

pub const SEED: u32 = 123456;

fn main() {
    assert_eq!(
        CHUNK_SIZE % CELL_SIZE,
        0.0,
        "expected chunk size ({}) to be divisible by cell size ({})",
        CHUNK_SIZE,
        CELL_SIZE,
    );

    let mut app = App::new();

    app.add_plugins((EmbeddedAssetPlugin::default(), DefaultPlugins));

    app.add_plugins((
        RapierPhysicsPlugin::<NoUserData>::default(),
        CursorPlugin,
        TextPopupPlugin,
        MenuPlugin,
        InventoryPlugin,
        PlayerPlugin,
        AnimationPlugin,
        InteractionPlugin,
        SettingsPlugin,
        CameraPlugin,
        GameSavePlugin,
        WorldPlugin,
        HudPlugin,
        #[cfg(debug_assertions)]
        DebugPlugin,
    ));

    app.run();
}
