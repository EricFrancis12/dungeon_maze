use bevy::prelude::*;

pub fn get_top_parent(mut curr_entity: Entity, parent_query: &Query<&Parent>) -> Entity {
    //Loop up all the way to the top parent
    loop {
        if let Ok(parent) = parent_query.get(curr_entity) {
            curr_entity = parent.get();
        } else {
            break;
        }
    }
    curr_entity
}

pub fn get_n_parent(mut curr_entity: Entity, parent_query: &Query<&Parent>, n: u32) -> Entity {
    let mut count = n;
    loop {
        if count == 0 {
            break;
        }
        if let Ok(parent) = parent_query.get(curr_entity) {
            curr_entity = parent.get();
            count -= 1;
        } else {
            break;
        }
    }
    curr_entity
}
