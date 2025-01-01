use crate::{
    interaction::{Interactable, PendingInteractionExecuted},
    player::{
        attack::{AttackHand, AttackType},
        PlayerState,
    },
    utils::{entity::get_n_parent, CyclicCounter},
};

use bevy::{animation::animate_targets, prelude::*};
use std::time::Duration;

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
    Running,
    UnarmedLeftLightAttack,
    UnarmedLeftHeavyAttack,
    UnarmedRightLightAttack,
    UnarmedRightHeavyAttack,
}

impl PlayerAnimation {
    pub fn index(&self) -> usize {
        match self {
            Self::Idle => 0,
            Self::Jogging => 1,
            Self::Running => 2,
            Self::UnarmedLeftHeavyAttack => 3,
            Self::UnarmedLeftLightAttack => 4,
            Self::UnarmedRightHeavyAttack => 5,
            Self::UnarmedRightLightAttack => 6,
        }
    }

    pub fn new_attack_animation(attack_type: &AttackType, attack_hand: &AttackHand) -> Self {
        match (attack_type, attack_hand) {
            (AttackType::Light, AttackHand::Left) => Self::UnarmedLeftLightAttack,
            (AttackType::Light, AttackHand::Right) => Self::UnarmedRightLightAttack,
            (AttackType::Heavy, AttackHand::Left) => Self::UnarmedLeftHeavyAttack,
            (AttackType::Heavy, AttackHand::Right) => Self::UnarmedRightHeavyAttack,
        }
    }

    pub fn is_attack_animation(&self) -> bool {
        match self {
            Self::UnarmedLeftLightAttack
            | Self::UnarmedLeftHeavyAttack
            | Self::UnarmedRightLightAttack
            | Self::UnarmedRightHeavyAttack => true,
            _ => false,
        }
    }

    pub fn is_matching_attack_animation(
        &self,
        attack_type: &AttackType,
        attack_hand: &AttackHand,
    ) -> bool {
        match self.is_attack_animation() {
            true => *self == Self::new_attack_animation(attack_type, attack_hand),
            false => false,
        }
    }
}

#[derive(Component)]
pub struct ContinuousAnimation;

#[derive(Component)]
pub struct CyclicAnimation {
    counter: CyclicCounter,
}

impl CyclicAnimation {
    pub fn new(min: u32, max: u32) -> Self {
        Self {
            counter: CyclicCounter::new(min, max),
        }
    }

    fn _value(&self) -> u32 {
        self.counter.value()
    }

    pub fn cycle(&mut self) -> u32 {
        self.counter.cycle()
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
                    .from_asset("models/Man.glb"), // idle
                GltfAssetLabel::Animation(PlayerAnimation::Jogging.index())
                    .from_asset("models/Man.glb"), // jogging
                GltfAssetLabel::Animation(PlayerAnimation::Running.index())
                    .from_asset("models/Man.glb"), // running
                GltfAssetLabel::Animation(PlayerAnimation::UnarmedLeftHeavyAttack.index())
                    .from_asset("models/Man.glb"), // unarmed left heavy attack
                GltfAssetLabel::Animation(PlayerAnimation::UnarmedLeftLightAttack.index())
                    .from_asset("models/Man.glb"), // unarmed left light attack
                GltfAssetLabel::Animation(PlayerAnimation::UnarmedRightHeavyAttack.index())
                    .from_asset("models/Man.glb"), // unarmed right heavy attack
                GltfAssetLabel::Animation(PlayerAnimation::UnarmedRightLightAttack.index())
                    .from_asset("models/Man.glb"), // unarmed right light attack
                GltfAssetLabel::Animation(1).from_asset("models/Treasure_Chest.glb"), // open
                GltfAssetLabel::Animation(0).from_asset("models/Treasure_Chest.glb"), // close
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
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (mut animation_player, mut transitions) in animation_player_query.iter_mut() {
        let is_moving =
            keys.any_pressed([KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD]);

        let ps = player_state.get();
        let pa = player_animation.get();

        let i = match ps {
            PlayerState::Walking | PlayerState::Sprinting => {
                if is_moving {
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
                }
            }
            PlayerState::Attacking(attack_type, attack_hand, _) => {
                if pa.is_matching_attack_animation(attack_type, attack_hand) {
                    continue;
                }

                let new_pa = PlayerAnimation::new_attack_animation(attack_type, attack_hand);

                next_player_animation.set(new_pa);
                new_pa.index()
            }
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
