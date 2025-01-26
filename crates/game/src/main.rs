mod plugins;

use crate::plugins::{
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
use crate::plugins::debug::DebugPlugin;

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
