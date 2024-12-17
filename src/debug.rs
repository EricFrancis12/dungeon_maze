use crate::{
    camera::MainCamera,
    player::{Player, PlayerState},
    utils::contains_any,
    world::ChunkCellMarker,
};

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use std::{env, f32::consts::PI};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        let args: Vec<String> = env::args().collect();

        let specified = |s: String| contains_any(&args, &[s, String::from("a")]);

        if specified(String::from("world")) {
            app.add_plugins(WorldInspectorPlugin::new());
        }

        if specified(String::from("rapier")) {
            app.add_plugins(RapierDebugRenderPlugin {
                enabled: true,
                mode: DebugRenderMode::all(),
                ..default()
            });
        }

        if specified(String::from("fly")) {
            app.add_systems(Update, player_flight_movement);
        }

        let position_arg = specified(String::from("position"));
        let compass_arg = specified(String::from("compass"));

        if position_arg || compass_arg {
            app.add_systems(Startup, spawn_ui_overlay);
        }

        if position_arg {
            app.add_systems(Startup, spawn_player_position_ui.after(spawn_ui_overlay))
                .add_systems(Update, update_player_position_ui);
        }

        if compass_arg {
            app.add_systems(Startup, spawn_compass_ui.after(spawn_ui_overlay))
                .add_systems(Update, update_compass_ui);
        }
    }
}

#[derive(Component)]
struct PositionMenu;

#[derive(Component)]
struct UIOverlay;

#[derive(Component)]
struct PositionMenuText;

#[derive(Component)]
struct Compass;

#[derive(Component)]
struct CompassArm;

#[derive(Component)]
struct CompassHand(f32);

fn player_flight_movement(
    camera_query: Query<&Transform, With<MainCamera>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    player_state: Res<State<PlayerState>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if *player_state.get() != PlayerState::Walking {
        return;
    }

    for mut player_transform in player_query.iter_mut() {
        let camera_transform = match camera_query.get_single() {
            Ok(ct) => ct,
            Err(err) => Err(format!("Error retrieving camera: {}", err)).unwrap(),
        };

        let mut direction = Vec3::ZERO;

        // Up
        if keys.pressed(KeyCode::KeyO) {
            direction += *camera_transform.up();
        }
        // Down
        if keys.pressed(KeyCode::KeyL) {
            direction += *camera_transform.down();
        }

        let movement = direction.normalize_or_zero() * 10.0 * time.delta_seconds();

        if direction.length_squared() > 0.0 {
            player_transform.translation.y += movement.y;
        }
    }
}

fn spawn_ui_overlay(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::End,
                row_gap: Val::Px(20.0),
                height: Val::Percent(100.0),
                width: Val::Percent(100.0),
                padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(16.0), Val::Px(16.0)),
                ..default()
            },
            ..default()
        },
        UIOverlay,
        Name::new("UI Overlay"),
    ));
}

fn spawn_player_position_ui(
    mut commands: Commands,
    ui_overlay_query: Query<Entity, With<UIOverlay>>,
) {
    let entity = ui_overlay_query.get_single().unwrap();

    commands.entity(entity).with_children(|parent| {
        parent
            .spawn((
                NodeBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::new(
                            Val::Px(8.0),
                            Val::Px(8.0),
                            Val::Px(8.0),
                            Val::Px(8.0),
                        ),
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
    player_query: Query<&GlobalTransform, With<Player>>,
    mut position_menu_text_query: Query<&mut Text, With<PositionMenuText>>,
) {
    let gt = player_query.get_single().unwrap();
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

fn spawn_compass_ui(mut commands: Commands, ui_overlay_query: Query<Entity, With<UIOverlay>>) {
    let entity = ui_overlay_query.get_single().unwrap();

    commands.entity(entity).with_children(|parent| {
        parent
            .spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Relative,
                        height: Val::Px(120.0),
                        width: Val::Px(120.0),
                        ..default()
                    },
                    border_radius: BorderRadius {
                        top_left: Val::Percent(50.0),
                        top_right: Val::Percent(50.0),
                        bottom_left: Val::Percent(50.0),
                        bottom_right: Val::Percent(50.0),
                    },
                    background_color: Color::WHITE.into(),
                    ..default()
                },
                Compass,
                Name::new("Compass"),
            ))
            .with_children(|parent| {
                for (sections, angle, color) in [
                    (
                        [["X", "-"], ["X", "+"]],
                        0.0,
                        Color::linear_rgba(160.0, 0.0, 0.0, 1.0),
                    ),
                    (
                        [["Z", "-"], ["Z", "+"]],
                        PI / 2.0,
                        Color::linear_rgba(0.0, 0.0, 160.0, 1.0),
                    ),
                ] {
                    parent
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    display: Display::Flex,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    top: Val::Percent(1.0 / 3.0 * 100.0),
                                    left: Val::Percent(0.0),
                                    height: Val::Percent(1.0 / 3.0 * 100.0),
                                    width: Val::Percent(100.0),
                                    ..default()
                                },
                                transform: Transform::from_rotation(Quat::from_rotation_z(angle)),
                                ..default()
                            },
                            CompassArm,
                            Name::new(format!("Compass Arm {}", sections[0][0])),
                        ))
                        .with_children(|grandparent| {
                            for section in sections {
                                grandparent.spawn((
                                    TextBundle {
                                        text: Text {
                                            sections: section
                                                .map(|s| {
                                                    TextSection::new(
                                                        s,
                                                        TextStyle {
                                                            font_size: 20.0,
                                                            color,
                                                            ..default()
                                                        },
                                                    )
                                                })
                                                .to_vec(),
                                            ..default()
                                        },
                                        style: Style {
                                            margin: UiRect {
                                                left: Val::Px(6.0),
                                                right: Val::Px(6.0),
                                                ..default()
                                            },
                                            ..default()
                                        },
                                        transform: Transform::from_rotation(Quat::from_rotation_z(
                                            angle / 2.0,
                                        )),
                                        ..default()
                                    },
                                    CompassHand(angle),
                                    Name::new(format!("Compass Hand {}", section.join(""))),
                                ));
                            }
                        });
                }
            });
    });
}

fn update_compass_ui(
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    player_query: Query<&GlobalTransform, With<Player>>,
    mut compass_query: Query<&mut Transform, With<Compass>>,
    mut compass_hands_query: Query<(&CompassHand, &mut Transform), Without<Compass>>,
) {
    let camera_gl_transform = camera_query.get_single().unwrap();
    let player_gl_transform = player_query.get_single().unwrap();
    let mut compass_transform = compass_query.get_single_mut().unwrap();

    let diff = player_gl_transform.translation() - camera_gl_transform.translation();
    let angle = diff.x.atan2(diff.z); // atan2 gives angle in the range (-PI, PI)

    compass_transform.rotation = Quat::from_rotation_z(angle);

    for (compass_hand, mut compass_hand_transform) in compass_hands_query.iter_mut() {
        compass_hand_transform.rotation = Quat::from_rotation_z(-angle - compass_hand.0);
    }
}
