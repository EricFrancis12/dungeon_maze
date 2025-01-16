use crate::player::{Health, Player, Stamina};
use bevy::prelude::*;

const HEALTH_BAR_MAX_WIDTH: f32 = 300.0;
const STAMINA_BAR_MAX_WIDTH: f32 = 300.0;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, (update_health_bar, update_stamina_bar));
    }
}

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct StaminaBar;

fn spawn_hud(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                column_gap: Val::Px(10.0),
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                HealthBar,
                NodeBundle {
                    style: Style {
                        height: Val::Px(30.0),
                        width: Val::Px(HEALTH_BAR_MAX_WIDTH),
                        margin: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    background_color: Color::linear_rgb(0.6, 0.2, 0.2).into(),
                    ..default()
                },
            ));

            parent.spawn((
                StaminaBar,
                NodeBundle {
                    style: Style {
                        height: Val::Px(30.0),
                        width: Val::Px(STAMINA_BAR_MAX_WIDTH),
                        margin: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    background_color: Color::linear_rgb(0.2, 0.6, 0.2).into(),
                    ..default()
                },
            ));
        });
}

fn update_health_bar(
    player_health_query: Query<&Health, With<Player>>,
    mut health_bar_query: Query<&mut Style, With<HealthBar>>,
) {
    for health in player_health_query.iter() {
        for mut style in health_bar_query.iter_mut() {
            style.width = Val::Px(health.value / health.max_value * HEALTH_BAR_MAX_WIDTH);
        }
    }
}

fn update_stamina_bar(
    player_stamina_query: Query<&Stamina, With<Player>>,
    mut stamina_bar_query: Query<&mut Style, With<StaminaBar>>,
) {
    for stamina in player_stamina_query.iter() {
        for mut style in stamina_bar_query.iter_mut() {
            style.width = Val::Px(stamina.value / stamina.max_value * STAMINA_BAR_MAX_WIDTH);
        }
    }
}
