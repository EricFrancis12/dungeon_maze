pub mod cell;
pub mod chunk;
pub mod door;
pub mod special;
pub mod wall;

use bevy::prelude::*;

pub const WALL_THICKNESS: f32 = 0.1;
const DOOR_CLOSE_FRAMES: usize = 24;

const DOOR_SCALE: Vec3 = Vec3 {
    x: 0.8,
    y: 0.88,
    z: 1.0,
};
const WALL_WITH_DOOR_SCALE: Vec3 = Vec3 {
    x: 2.0,
    y: WALL_THICKNESS * 2.0,
    z: 2.0,
};

const CHAIR_COLLIDER_HX: f32 = 0.2;
const CHAIR_COLLIDER_HY: f32 = 0.25;
const CHAIR_COLLIDER_HZ: f32 = 0.2;

const TREASURE_CHEST_COLLIDER_HX: f32 = 0.5;
const TREASURE_CHEST_COLLIDER_HY: f32 = 0.3;
const TREASURE_CHEST_COLLIDER_HZ: f32 = 0.3;
