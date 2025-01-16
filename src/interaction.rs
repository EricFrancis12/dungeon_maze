use crate::player::Player;
use bevy::prelude::*;

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PendingInteractionExecuted>()
            .init_state::<PendingInteraction>()
            .add_systems(
                Update,
                (update_pending_interaction, execute_pending_interaction),
            );
    }
}

#[derive(Component)]
pub struct Interactable {
    pub range: f32,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
struct PendingInteraction(Option<Entity>);

#[derive(Event)]
pub struct PendingInteractionExecuted(pub Entity);

fn update_pending_interaction(
    interactables_query: Query<(Entity, &Interactable, &GlobalTransform)>,
    player_query: Query<&GlobalTransform, With<Player>>,
    pending_interaction: Res<State<PendingInteraction>>,
    mut next_pending_interaction: ResMut<NextState<PendingInteraction>>,
) {
    let player_gl_transform = player_query.get_single().expect("Error retrieving player");
    let curr_entity = pending_interaction.get().0;

    // Check if player is in range of any interactables
    let mut closest_entity: Option<(Entity, f32)> = None;

    for (entity, interactable, ibl_gl_transform) in interactables_query.iter() {
        let dist = player_gl_transform
            .translation()
            .distance(ibl_gl_transform.translation());

        if dist <= interactable.range
            && (closest_entity.is_none() || dist < closest_entity.unwrap().1)
        {
            closest_entity = Some((entity, dist));
        }
    }

    if let Some((entity, _)) = closest_entity {
        next_pending_interaction.set(PendingInteraction(Some(entity)));

        return;
    }

    // Change back to none if no interactables in range
    if curr_entity.is_some() {
        next_pending_interaction.set(PendingInteraction(None));
    }
}

fn execute_pending_interaction(
    mut event_writer: EventWriter<PendingInteractionExecuted>,
    pending_interaction: Res<State<PendingInteraction>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }

    if let Some(entity) = pending_interaction.get().0 {
        event_writer.send(PendingInteractionExecuted(entity));
    }
}
