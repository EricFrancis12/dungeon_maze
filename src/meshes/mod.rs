pub mod obj;

use crate::mesh_from_obj;
use bevy::prelude::Mesh;

pub fn new_staircase_mesh() -> Mesh {
    mesh_from_obj!("../../assets/meshes/staircase.obj")
}

pub fn new_stairs_mesh() -> Mesh {
    mesh_from_obj!("../../assets/meshes/stairs.obj")
}

pub fn new_wall_with_door_gap_mesh() -> Mesh {
    mesh_from_obj!("../../assets/meshes/wall_with_door_gap.obj")
}

pub fn new_wall_with_window_gap_mesh() -> Mesh {
    mesh_from_obj!("../../assets/meshes/wall_with_window_gap.obj")
}
