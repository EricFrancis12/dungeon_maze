use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use std::env;

use crate::{
    player::Player,
    settings::{ChunkRenderDist, GameSettings, GameSettingsChangeRequest},
    world::{ActiveChunk, ActiveChunkChangeRequest, ChunkCellMarker},
};

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

        if args.contains(&String::from("render")) {
            app.add_systems(Update, change_game_settings_render_dist);
        }

        if args.contains(&String::from("position")) {
            app.add_systems(Startup, spawn_player_position_ui)
                .add_systems(Update, update_player_position_ui);
        }
    }
}

#[derive(Component)]
struct PositionMenu;

#[derive(Component)]
struct PositionMenuOverlay;

#[derive(Component)]
struct PositionMenuText;

fn change_game_settings_render_dist(
    mut acc_event_writer: EventWriter<ActiveChunkChangeRequest>,
    mut gs_event_writer: EventWriter<GameSettingsChangeRequest>,
    active_chunk: Res<State<ActiveChunk>>,
    game_settings: Res<State<GameSettings>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let pressed = keys.get_pressed().collect::<Vec<&KeyCode>>();
    if pressed.is_empty() {
        return;
    }

    let dist = match pressed[0] {
        KeyCode::Numpad0 => 0,
        KeyCode::Numpad1 => 1,
        KeyCode::Numpad2 => 2,
        KeyCode::Numpad3 => 3,
        KeyCode::Numpad4 => 4,
        KeyCode::Numpad5 => 5,
        _ => return,
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

fn spawn_player_position_ui(mut commands: Commands) {
    let rect = UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(8.0), Val::Px(8.0));

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    justify_content: JustifyContent::End,
                    align_items: AlignItems::Start,
                    column_gap: Val::Px(10.0),
                    height: Val::Percent(100.0),
                    width: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
            PositionMenuOverlay,
            Name::new("Position Menu Overlay"),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: rect.clone(),
                            margin: rect,
                            ..default()
                        },
                        background_color: Color::WHITE.into(),
                        ..default()
                    },
                    PositionMenu,
                    Name::new("Position Menu"),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle {
                            text: Text {
                                sections: vec![TextSection::new(
                                    "",
                                    TextStyle {
                                        font_size: 20.0,
                                        color: Color::BLACK,
                                        ..default()
                                    },
                                )],
                                ..default()
                            },
                            ..default()
                        },
                        PositionMenuText,
                        Name::new("Position Menu Text"),
                    ));
                });
        });
}

fn update_player_position_ui(
    player_gl_transform_query: Query<&GlobalTransform, With<Player>>,
    mut position_menu_text_query: Query<&mut Text, With<PositionMenuText>>,
) {
    let gt = player_gl_transform_query.get_single().unwrap();
    let ccm = ChunkCellMarker::from_global_transform(gt);

    for mut text in position_menu_text_query.iter_mut() {
        for section in text.sections.iter_mut() {
            section.value = format!(
                "Chunk: ({},{},{})   Cell: ({},{})",
                ccm.chunk_x, ccm.chunk_y, ccm.chunk_z, ccm.x, ccm.z
            );
        }
    }
}
