use bevy::prelude::*;

use crate::player::Player;

pub struct InteractionPligin;

impl Plugin for InteractionPligin {
    fn build(&self, app: &mut App) {
        app.add_event::<PendingInteractionChanged>()
            .add_event::<PendingInteractionExecuted>()
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
struct PendingInteraction(Option<u32>);

#[derive(Event)]
struct PendingInteractionChanged;

#[derive(Event)]
pub struct PendingInteractionExecuted(pub u32);

fn update_pending_interaction(
    mut event_writer: EventWriter<PendingInteractionChanged>,
    interactables_query: Query<(Entity, &Interactable, &GlobalTransform)>,
    player_query: Query<&GlobalTransform, With<Player>>,
    pending_interaction: Res<State<PendingInteraction>>,
    mut next_pending_interaction: ResMut<NextState<PendingInteraction>>,
) {
    let player_gl_transform = player_query.get_single().expect("Error retrieving player");
    let curr_index = pending_interaction.get().0;

    // Check if player is in range of any interactables
    for (entity, interactable, ibl_gl_transform) in interactables_query.iter() {
        let dist = player_gl_transform
            .translation()
            .distance(ibl_gl_transform.translation());

        if dist <= interactable.range {
            next_pending_interaction.set(PendingInteraction(Some(entity.index())));

            if curr_index.is_none() || curr_index.unwrap() != entity.index() {
                event_writer.send(PendingInteractionChanged);
                println!("changed to Some: {}", entity.index());
            }
            return;
        }
    }

    if curr_index.is_some() {
        next_pending_interaction.set(PendingInteraction(None));
        event_writer.send(PendingInteractionChanged);
        println!("changed to None");
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

    if let Some(index) = pending_interaction.get().0 {
        println!("interaction executed for entity: {}", index);
        event_writer.send(PendingInteractionExecuted(index));
    }
}
