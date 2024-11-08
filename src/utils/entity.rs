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
