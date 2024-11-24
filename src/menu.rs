use crate::{
    settings::{ChunkRenderDist, GameSettings, GameSettingsChangeRequest, GameSettingsChanged},
    world::{ActiveChunk, ActiveChunkChangeRequest},
};

use bevy::prelude::*;
use std::fmt::{Formatter, Result};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ActiveMenuTabChanged>()
            .init_state::<MenuOpen>()
            .init_state::<ActiveMenuTab>()
            .add_systems(Update, toggle_menu_open)
            .add_systems(Update, change_active_menu_tab)
            .add_systems(Update, spawn_menu_content_on_tab_change)
            .add_systems(Update, change_menu_tabs_background_color)
            .add_systems(Update, change_render_dist)
            .add_systems(Update, change_render_dist_buttons_background_color)
            .add_systems(OnEnter(MenuOpen(true)), spawn_menu)
            .add_systems(OnExit(MenuOpen(true)), despawn_menu);
    }
}

#[derive(Clone, Component, Debug, Default, Eq, Hash, PartialEq)]
enum MenuTab {
    #[default]
    Inventory,
    Settings,
}

impl std::fmt::Display for MenuTab {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Inventory => write!(f, "Inventory"),
            Self::Settings => write!(f, "Settings"),
        }
    }
}

#[derive(Component)]
struct Menu;

#[derive(Component)]
struct MenuContent;

#[derive(Component)]
struct RenderDistButton(u32);

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
struct MenuOpen(bool);

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
struct ActiveMenuTab(MenuTab);

#[derive(Event)]
struct ActiveMenuTabChanged;

fn toggle_menu_open(
    keys: Res<ButtonInput<KeyCode>>,
    menu_open: Res<State<MenuOpen>>,
    mut next_menu_open: ResMut<NextState<MenuOpen>>,
) {
    if keys.just_pressed(KeyCode::KeyM) {
        next_menu_open.set(MenuOpen(!menu_open.0));
    }
}

fn spawn_menu(
    mut commands: Commands,
    active_menu_tab: Res<State<ActiveMenuTab>>,
    game_settings: Res<State<GameSettings>>,
) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(0.0),
                    height: Val::Percent(80.0),
                    width: Val::Percent(20.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                background_color: Color::BLACK.into(),
                ..default()
            },
            Menu,
            Name::new("Menu"),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            height: Val::Percent(94.0),
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        background_color: Color::linear_rgba(0.0, 0.0, 0.7, 1.0).into(),
                        ..default()
                    },
                    MenuContent,
                ))
                .with_children(|grandparent| match active_menu_tab.get().0 {
                    MenuTab::Inventory => spawn_inventory_menu_content(grandparent),
                    MenuTab::Settings => spawn_settings_menu_content(grandparent, &game_settings),
                });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        height: Val::Percent(6.0),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: Color::linear_rgba(0.0, 0.7, 0.0, 1.0).into(),
                    ..default()
                })
                .with_children(|grandparent| {
                    for tab in [MenuTab::Inventory, MenuTab::Settings] {
                        grandparent.spawn((
                            ButtonBundle {
                                style: Style {
                                    height: Val::Percent(100.0),
                                    width: Val::Percent(20.0),
                                    ..default()
                                },
                                background_color: get_tab_background_color(
                                    &tab,
                                    active_menu_tab.get(),
                                ),
                                ..default()
                            },
                            Name::new(format!("Menu Tab {}", tab)),
                            tab,
                        ));
                    }
                });
        });
}

fn despawn_menu(mut commands: Commands, menu_query: Query<Entity, With<Menu>>) {
    let menu_entity = menu_query.get_single().unwrap();
    commands.entity(menu_entity).despawn_recursive();
}

fn change_active_menu_tab(
    mut event_writer: EventWriter<ActiveMenuTabChanged>,
    menu_tab_query: Query<(&MenuTab, &Interaction)>,
    active_menu_tab: Res<State<ActiveMenuTab>>,
    mut next_active_menu_tab: ResMut<NextState<ActiveMenuTab>>,
) {
    for (menu_tab, interaction) in menu_tab_query.iter() {
        if *interaction == Interaction::Pressed {
            if *menu_tab != active_menu_tab.get().0 {
                next_active_menu_tab.set(ActiveMenuTab(menu_tab.clone()));
                event_writer.send(ActiveMenuTabChanged);
                break;
            }
        }
    }
}

fn spawn_menu_content_on_tab_change(
    mut commands: Commands,
    mut event_reader: EventReader<ActiveMenuTabChanged>,
    menu_content_query: Query<Entity, With<MenuContent>>,
    active_menu_tab: Res<State<ActiveMenuTab>>,
    game_settings: Res<State<GameSettings>>,
) {
    for _ in event_reader.read() {
        let entity = menu_content_query.get_single().unwrap();

        let mut entity_commands = commands.entity(entity);
        entity_commands.despawn_descendants();

        entity_commands.with_children(|parent| match active_menu_tab.get().0 {
            MenuTab::Inventory => spawn_inventory_menu_content(parent),
            MenuTab::Settings => spawn_settings_menu_content(parent, &game_settings),
        });
    }
}

fn spawn_inventory_menu_content(child_builder: &mut ChildBuilder) {
    child_builder.spawn(TextBundle {
        text: Text {
            sections: vec![TextSection::new(
                "Inventory",
                TextStyle {
                    font_size: 20.0,
                    ..default()
                },
            )],
            ..default()
        },
        ..default()
    });
}

fn spawn_settings_menu_content(
    child_builder: &mut ChildBuilder,
    game_settings: &Res<State<GameSettings>>,
) {
    child_builder.spawn(TextBundle {
        text: Text {
            sections: vec![TextSection::new(
                "Settings",
                TextStyle {
                    font_size: 20.0,
                    ..default()
                },
            )],
            ..default()
        },
        ..default()
    });

    child_builder
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                justify_content: JustifyContent::SpaceEvenly,
                width: Val::Percent(90.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for i in 0..=5u32 {
                let background_color = if i == game_settings.get().chunk_render_dist.0 {
                    Color::linear_rgba(0.0, 0.0, 0.4, 1.0).into()
                } else {
                    Color::WHITE.into()
                };

                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                display: Display::Flex,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                height: Val::Px(20.0),
                                width: Val::Px(20.0),
                                ..default()
                            },
                            background_color,
                            ..default()
                        },
                        RenderDistButton(i),
                    ))
                    .with_children(|grandparent| {
                        grandparent.spawn(TextBundle {
                            text: Text {
                                sections: vec![TextSection::new(
                                    format!("{}", i),
                                    TextStyle {
                                        font_size: 20.0,
                                        color: Color::BLACK.into(),
                                        ..default()
                                    },
                                )],
                                ..default()
                            },
                            ..default()
                        });
                    });
            }
        });
}

fn change_menu_tabs_background_color(
    mut event_reader: EventReader<ActiveMenuTabChanged>,
    mut menu_tab_query: Query<(&MenuTab, &mut BackgroundColor)>,
    active_menu_tab: Res<State<ActiveMenuTab>>,
) {
    for _ in event_reader.read() {
        for (tab, mut background_color) in menu_tab_query.iter_mut() {
            *background_color = get_tab_background_color(tab, active_menu_tab.get());
        }
    }
}

fn get_tab_background_color(tab: &MenuTab, active_menu_tab: &ActiveMenuTab) -> BackgroundColor {
    let red = if *tab == active_menu_tab.0 { 0.3 } else { 0.7 };
    Color::linear_rgba(red, 0.0, 0.0, 1.0).into()
}

fn change_render_dist(
    mut acc_event_writer: EventWriter<ActiveChunkChangeRequest>,
    mut gs_event_writer: EventWriter<GameSettingsChangeRequest>,
    active_chunk: Res<State<ActiveChunk>>,
    button_query: Query<(&RenderDistButton, &Interaction)>,
    game_settings: Res<State<GameSettings>>,
) {
    for (button, interaction) in button_query.iter() {
        if *interaction == Interaction::Pressed {
            if button.0 != game_settings.get().chunk_render_dist.0 {
                println!("button: {}", button.0);
                let mut value = game_settings.clone();
                value.chunk_render_dist = ChunkRenderDist(button.0, button.0, button.0);

                gs_event_writer.send(GameSettingsChangeRequest { value });

                acc_event_writer.send(ActiveChunkChangeRequest {
                    value: active_chunk.clone(),
                });

                break;
            }
        }
    }
}

fn change_render_dist_buttons_background_color(
    mut event_reader: EventReader<GameSettingsChanged>,
    mut buttons_query: Query<(&RenderDistButton, &mut BackgroundColor)>,
    game_settings: Res<State<GameSettings>>,
) {
    for _ in event_reader.read() {
        for (render_dist_button, mut background_color) in buttons_query.iter_mut() {
            if render_dist_button.0 == game_settings.get().chunk_render_dist.0 {
                *background_color = Color::linear_rgba(0.0, 0.0, 0.4, 1.0).into();
            } else {
                *background_color = Color::WHITE.into();
            }
        }
    }
}
