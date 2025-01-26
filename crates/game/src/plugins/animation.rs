use bevy::{animation::animate_targets, prelude::*};
use dungeon_maze_common::{
    animation::{AnimationLib, ContinuousAnimation, CyclicAnimation, PlayerAnimation},
    interaction::{Interactable, PendingInteractionExecuted},
    inventory::Inventory,
    player::PlayerState,
    utils::entity::get_n_parent,
};
use std::time::Duration;

const TRANSITION_DURATION: Duration = Duration::from_millis(250);

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
                    on_finish_attack_animation,
                ),
            );
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
                    .from_asset("embedded://models/man.glb"), // idle
                GltfAssetLabel::Animation(PlayerAnimation::Jogging.index())
                    .from_asset("embedded://models/man.glb"), // jogging
                GltfAssetLabel::Animation(PlayerAnimation::OneHandedSlashRightLightAttack.index())
                    .from_asset("embedded://models/man.glb"), // slash right light attack
                GltfAssetLabel::Animation(PlayerAnimation::Running.index())
                    .from_asset("embedded://models/man.glb"), // running
                GltfAssetLabel::Animation(PlayerAnimation::UnarmedLeftHeavyAttack.index())
                    .from_asset("embedded://models/man.glb"), // unarmed left heavy attack
                GltfAssetLabel::Animation(PlayerAnimation::UnarmedLeftLightAttack.index())
                    .from_asset("embedded://models/man.glb"), // unarmed left light attack
                GltfAssetLabel::Animation(PlayerAnimation::UnarmedRightHeavyAttack.index())
                    .from_asset("embedded://models/man.glb"), // unarmed right heavy attack
                GltfAssetLabel::Animation(PlayerAnimation::UnarmedRightLightAttack.index())
                    .from_asset("embedded://models/man.glb"), // unarmed right light attack
                GltfAssetLabel::Animation(1).from_asset("embedded://models/treasure_chest.glb"), // open
                GltfAssetLabel::Animation(0).from_asset("embedded://models/treasure_chest.glb"), // close
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
    mut animation_player_query: Query<Entity, Added<AnimationPlayer>>,
    continuous_animation_query: Query<&ContinuousAnimation>,
    parent_query: Query<&Parent>,
    animation_lib: Res<AnimationLib>,
) {
    for entity in &mut animation_player_query {
        if continuous_animation_query
            .get(get_n_parent(entity, &parent_query, 3))
            .is_ok()
        {
            commands
                .entity(entity)
                .insert(animation_lib.graph.clone())
                .insert(AnimationTransitions::new());
        }
    }
}

fn handle_cyclic_interaction_animations(
    mut commands: Commands,
    mut event_reader: EventReader<PendingInteractionExecuted>,
    mut animation_player_query: Query<(Entity, &mut AnimationPlayer)>,
    mut cyclic_animation_query: Query<&mut CyclicAnimation, With<Interactable>>,
    parent_query: Query<&Parent>,
    animation_lib: Res<AnimationLib>,
) {
    for event in event_reader.read() {
        for (entity, mut animation_player) in &mut animation_player_query {
            let parent = get_n_parent(entity, &parent_query, 3);
            if parent != event.0 {
                continue;
            }

            if let Ok(mut cyclic_animation) = cyclic_animation_query.get_mut(parent) {
                let c = cyclic_animation.cycle();

                animation_player.stop_all();
                animation_player
                    .play(animation_lib.nodes[c as usize])
                    .replay();

                commands.entity(entity).insert(animation_lib.graph.clone());
            }
        }
    }
}

fn change_player_animation(
    mut animation_player_query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    player_animation: Res<State<PlayerAnimation>>,
    player_state: Res<State<PlayerState>>,
    mut next_player_animation: ResMut<NextState<PlayerAnimation>>,
    animation_lib: Res<AnimationLib>,
    inventory: Res<Inventory>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (mut animation_player, mut transitions) in animation_player_query.iter_mut() {
        let is_moving =
            keys.any_pressed([KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD]);

        let ps = player_state.get();
        let pa = player_animation.get();

        match ps {
            PlayerState::Walking | PlayerState::Sprinting => {
                let i = if is_moving {
                    if *ps == PlayerState::Walking && *pa != PlayerAnimation::Jogging {
                        next_player_animation.set(PlayerAnimation::Jogging);
                        PlayerAnimation::Jogging.index()
                    } else if *ps == PlayerState::Sprinting && *pa != PlayerAnimation::Running {
                        next_player_animation.set(PlayerAnimation::Running);
                        PlayerAnimation::Running.index()
                    } else {
                        continue;
                    }
                } else if *pa != PlayerAnimation::Idle {
                    next_player_animation.set(PlayerAnimation::Idle);
                    PlayerAnimation::Idle.index()
                } else {
                    continue;
                };

                transitions
                    .play(
                        &mut animation_player,
                        animation_lib.nodes[i],
                        TRANSITION_DURATION,
                    )
                    .repeat();
            }
            PlayerState::Attacking(attack_type, attack_hand) => {
                let slot = inventory.equipment.at(&attack_hand.into());
                if pa.is_matching_attack_animation(attack_type, attack_hand, slot) {
                    continue;
                }

                let new_pa = PlayerAnimation::new_attack_animation(attack_type, attack_hand, slot);
                next_player_animation.set(new_pa);

                transitions.play(
                    &mut animation_player,
                    animation_lib.nodes[new_pa.index()],
                    TRANSITION_DURATION,
                );
            }
        };
    }
}

fn on_finish_attack_animation(
    animation_player_query: Query<&AnimationPlayer, With<AnimationTransitions>>,
    player_state: Res<State<PlayerState>>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
) {
    for animation_player in animation_player_query.iter() {
        for (_, active_animation) in animation_player.playing_animations() {
            if active_animation.is_finished() && *player_state.get() != PlayerState::Walking {
                next_player_state.set(PlayerState::Walking);
            }
        }
    }
}
