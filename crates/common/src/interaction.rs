use bevy::prelude::{Component, Entity, Event, States};

#[derive(Component)]
pub struct Interactable {
    pub range: f32,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub struct PendingInteraction(pub Option<Entity>);

#[derive(Event)]
pub struct PendingInteractionExecuted(pub Entity);
