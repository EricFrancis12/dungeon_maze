use bevy::prelude::{Component, Resource, Vec2};

#[derive(Default, Resource)]
pub struct CursorPosition(pub Vec2);

#[derive(Component)]
pub struct CursorFollower;
