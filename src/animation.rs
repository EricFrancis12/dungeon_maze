use bevy::{animation::animate_targets, prelude::*};
use std::time::Duration;

use crate::interaction::{Interactable, PendingInteractionExecuted};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PlayerAnimation>()
            .add_systems(Startup, setup_animations)
            .add_systems(
                Update,
                (
                    play_continuous_animations.before(animate_targets),
                    handle_cyclic_interaction_animations,
                    change_player_animation,
                ),
            );
    }
}

#[derive(Resource)]
pub struct AnimationLib {
    nodes: Vec<AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum PlayerAnimation {
    #[default]
    Idle,
    Jogging,
}

impl PlayerAnimation {
    pub fn index(&self) -> usize {
        match self {
            Self::Idle => 0,
            Self::Jogging => 1,
        }
    }
}

#[derive(Component)]
pub struct CyclicAnimation {
    curr: u32,
    max: u32,
}

impl CyclicAnimation {
    pub fn new(curr: u32, max: u32) -> Self {
        Self { curr, max }
    }

    pub fn cycle(&mut self) -> u32 {
        let c = self.curr;
        if self.curr == self.max {
            self.curr = 0;
        } else {
            self.curr += 1;
        }
        c
    }
}

fn setup_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Build the animation graph
    let mut graph = AnimationGraph::new();
    let nodes = graph
        .add_clips(
            [
                GltfAssetLabel::Animation(PlayerAnimation::Idle.index())
                    .from_asset("models/Man.glb#Animation1"),
                GltfAssetLabel::Animation(PlayerAnimation::Jogging.index())
                    .from_asset("models/Man.glb#Animation2"),
                GltfAssetLabel::Animation(0).from_asset("models/Treasure_Chest.glb#Animation0"),
            ]
            .into_iter()
            .map(|path| asset_server.load(path)),
            1.0,
            graph.root,
        )
        .collect();

    // Insert a resource with the current scene information
    commands.insert_resource(AnimationLib {
        nodes,
        graph: graphs.add(graph),
    });
}

fn play_continuous_animations(
    mut commands: Commands,
    mut animation_player_query: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    animation_lib: Res<AnimationLib>,
) {
    for (entity, mut animation_player) in &mut animation_player_query {
        let mut transitions = AnimationTransitions::new();

        transitions
            .play(
                &mut animation_player,
                animation_lib.nodes[0],
                Duration::ZERO,
            )
            .repeat();

        commands
            .entity(entity)
            .insert(animation_lib.graph.clone())
            .insert(transitions);
    }
}

fn handle_cyclic_interaction_animations(
    mut event_reader: EventReader<PendingInteractionExecuted>,
    mut cyclic_animation_query: Query<(Entity, &mut CyclicAnimation), With<Interactable>>,
) {
    for event in event_reader.read() {
        for (entity, mut cyclic_animation) in cyclic_animation_query.iter_mut() {
            if entity.index() == event.0 {
                let c = cyclic_animation.cycle();
                // TODO: animate entity based on what "c" value it is on
                println!("animating entity {} at cycle: {}", entity.index(), c);
                break;
            }
        }
    }
}

fn change_player_animation(
    mut animation_player_query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    player_animation: Res<State<PlayerAnimation>>,
    mut next_player_animation: ResMut<NextState<PlayerAnimation>>,
    animation_lib: Res<AnimationLib>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (mut animation_player, mut transitions) in animation_player_query.iter_mut() {
        let is_moving =
            keys.any_pressed([KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD]);
        let animation = *player_animation.get();

        let i = if is_moving && animation != PlayerAnimation::Jogging {
            next_player_animation.set(PlayerAnimation::Jogging);
            PlayerAnimation::Jogging.index()
        } else if !is_moving && animation != PlayerAnimation::Idle {
            next_player_animation.set(PlayerAnimation::Idle);
            PlayerAnimation::Idle.index()
        } else {
            continue;
        };

        transitions
            .play(
                &mut animation_player,
                animation_lib.nodes[i],
                Duration::from_millis(250),
            )
            .repeat();
    }
}
