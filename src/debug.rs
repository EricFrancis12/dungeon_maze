use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use std::env;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        let args: Vec<String> = env::args().collect();

        if args.contains(&String::from("world")) {
            app.add_plugins(WorldInspectorPlugin::new());
        }

        if args.contains(&String::from("rapier")) {
            app.add_plugins(RapierDebugRenderPlugin {
                enabled: true,
                mode: DebugRenderMode::all(),
                ..default()
            });
        }
    }
}
