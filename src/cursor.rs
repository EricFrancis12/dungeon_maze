use bevy::prelude::*;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPosition>()
            .add_systems(Update, (update_cursor_position, cursor_follower_movement));
    }
}

#[derive(Default, Resource)]
pub struct CursorPosition(pub Vec2);

#[derive(Component)]
pub struct CursorFollower;

fn update_cursor_position(
    mut event_reader: EventReader<CursorMoved>,
    mut cursor_position: ResMut<CursorPosition>,
) {
    for event in event_reader.read() {
        cursor_position.0 = event.position;
    }
}

fn cursor_follower_movement(
    mut cursor_follower_query: Query<&mut Style, With<CursorFollower>>,
    cursor_position: Res<CursorPosition>,
) {
    for mut style in cursor_follower_query.iter_mut() {
        style.left = Val::Px(cursor_position.0.x);
        style.top = Val::Px(cursor_position.0.y);
    }
}
