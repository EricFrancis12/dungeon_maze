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
use bevy_third_person_camera::*;

pub const SEED: u32 = 1234;

fn main() {
    assert_eq!(
        CHUNK_SIZE % CELL_SIZE,
        0.0,
        "expected chunk size ({}) to be divisible by cell size ({})",
        CHUNK_SIZE,
        CELL_SIZE,
    );

    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
            WorldInspectorPlugin::new(),
            ThirdPersonCameraPlugin,
            CameraPlugin,
            PlayerPlugin,
            AnimationPlugin,
            WorldPlugin,
            InteractionPligin,
            GameSavePlugin,
            SettingsPlugin,
        ))
        .run();
}
