use bevy::prelude::Mesh;
use bevy_mesh_obj::mesh_from_obj;

pub fn new_staircase_mesh() -> Mesh {
    mesh_from_obj!("../../../assets/meshes/staircase.obj")
}

pub fn new_stairs_mesh() -> Mesh {
    mesh_from_obj!("../../../assets/meshes/stairs.obj")
}

pub fn new_wall_with_door_gap_mesh() -> Mesh {
    mesh_from_obj!("../../../assets/meshes/wall_with_door_gap.obj")
}

pub fn new_wall_with_window_gap_mesh() -> Mesh {
    mesh_from_obj!("../../../assets/meshes/wall_with_window_gap.obj")
}
