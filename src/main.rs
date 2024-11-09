mod animation;
mod camera;
mod chunk_test;
mod error;
mod interaction;
mod maze;
mod player;
mod save;
mod settings;
mod utils;
mod world;

use std::env;

use animation::AnimationPlugin;
use camera::CameraPlugin;
use interaction::InteractionPligin;
use player::PlayerPlugin;
use save::GameSavePlugin;
use settings::SettingsPlugin;
use world::{WorldPlugin, CELL_SIZE, CHUNK_SIZE};

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;

pub const SEED: u32 = 1234;

fn main() {
    assert_eq!(
        CHUNK_SIZE % CELL_SIZE,
        0.0,
        "expected chunk size ({}) to be divisible by cell size ({})",
        CHUNK_SIZE,
        CELL_SIZE,
    );

    let args: Vec<String> = env::args().collect();

    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        RapierPhysicsPlugin::<NoUserData>::default(),
        CameraPlugin,
        PlayerPlugin,
        AnimationPlugin,
        WorldPlugin,
        InteractionPligin,
        GameSavePlugin,
        SettingsPlugin,
    ));

    if args.contains(&String::from("dev")) {
        app.add_plugins((
            WorldInspectorPlugin::new(),
            RapierDebugRenderPlugin {
                enabled: true,
                mode: DebugRenderMode::all(),
                ..default()
            },
        ));
    }

    app.run();
}
