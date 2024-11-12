use bevy::prelude::*;

pub fn get_n_parent(mut curr_entity: Entity, parent_query: &Query<&Parent>, mut n: u32) -> Entity {
    loop {
        if n == 0 {
            break;
        }
        if let Ok(parent) = parent_query.get(curr_entity) {
            curr_entity = parent.get();
            n -= 1;
        } else {
            break;
        }
    }
    curr_entity
}

pub fn incr_betw_transforms(t1: Transform, t2: Transform, frames_betw: usize) -> Vec<Transform> {
    let mut transforms = vec![t1];
    for i in 0..frames_betw {
        let translation = Vec3::lerp(
            t1.translation,
            t2.translation,
            i as f32 / frames_betw as f32,
        );
        let rotation = Quat::slerp(t1.rotation, t2.rotation, i as f32 / frames_betw as f32);
        let scale = Vec3::lerp(t1.scale, t2.scale, i as f32 / frames_betw as f32);

        transforms.push(Transform {
            translation,
            rotation,
            scale,
        });
    }
    transforms.push(t2);
    transforms
}
