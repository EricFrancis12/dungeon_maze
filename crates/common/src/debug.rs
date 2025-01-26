use bevy::prelude::Component;

#[derive(Component)]
pub struct PositionMenu;

#[derive(Component)]
pub struct UIOverlay;

#[derive(Component)]
pub struct PositionMenuText;

#[derive(Component)]
pub struct Compass;

#[derive(Component)]
pub struct CompassArm;

#[derive(Component)]
pub struct CompassHand(pub f32);
