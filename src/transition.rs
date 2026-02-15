use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    prelude::*,
    render::render_resource::TextureFormat,
    ui::FocusPolicy,
};

use crate::{
    anim::{IgnorePlayingState, PlayAnimation},
    loader::PreLoadAssets,
};

pub struct TransitionPlugin;

impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<TransitionState>()
            .add_systems(Startup, setup)
            .add_systems(
                PostUpdate,
                update_transition_mesh_children.run_if(in_state(TransitionState::In)),
            )
            .add_systems(
                PreUpdate,
                check_transition_state
                    .run_if(in_state(TransitionState::In).or(in_state(TransitionState::Out))),
            )
            .add_observer(on_transition_start)
            .add_observer(on_transition_continue);

        #[cfg(feature = "dev")]
        app.add_systems(
            Update,
            bevy::dev_tools::states::log_transitions::<TransitionState>,
        );
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, States)]
enum TransitionState {
    #[default]
    NotStarted,
    In,
    Middle,
    Out,
}

// Ins

/// Trigger a transition
#[derive(Event)]
pub struct StartTransition;

/// Continue a transition after it reached it's middle
#[derive(Event)]
pub struct ContinueTransition;

// Outs

/// The transition just started
#[derive(Event)]
pub struct TransitionStarted;

/// The transition reached the middle
#[derive(Event)]
pub struct TransitionReachedMiddle;

/// The transition was resumed after stoping at the middle
#[derive(Event)]
pub struct TransitionResume;

/// The transition ended
#[derive(Event)]
pub struct TransitionEnded;

// ---

const TRANSITION_RENDER_LAYER: RenderLayers = RenderLayers::layer(1);

#[derive(Component)]
struct TransitionCamera;

#[derive(Component)]
struct TransitionMesh;

#[derive(Component)]
struct TransitionIn;

#[derive(Component)]
struct TransitionOut;

/// Spawns a camera which will render to an image covering the whole screen through the UI
/// The transition will render on top of everything
fn setup(mut commands: Commands, window: Single<&Window>, mut images: ResMut<Assets<Image>>) {
    let image = Image::new_target_texture(
        window.resolution.physical_width(),
        window.resolution.physical_height(),
        TextureFormat::Rgba8UnormSrgb,
        Some(TextureFormat::Rgba8UnormSrgb),
    );

    let image_handle = images.add(image);

    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..Default::default()
        },
        Transform::IDENTITY,
        TransitionCamera,
        TRANSITION_RENDER_LAYER,
        RenderTarget::Image(image_handle.clone().into()),
        Name::new("Transition Camera"),
    ));

    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                ..Default::default()
            },
            ZIndex(1000),
            FocusPolicy::Pass,
            Pickable::IGNORE,
            Name::new("Transition UI"),
        ))
        .with_children(|command| {
            command.spawn((
                ImageNode {
                    image: image_handle.clone(),
                    ..Default::default()
                },
                FocusPolicy::Pass,
                Pickable::IGNORE,
            ));
        });
}

/// Start the transition when [`StartTransition`] is triggered
fn on_transition_start(
    _: On<StartTransition>,
    mut commands: Commands,
    assets: Res<PreLoadAssets>,
    mut next_state: ResMut<NextState<TransitionState>>,
) {
    commands.spawn((
        TransitionMesh,
        TransitionIn,
        SceneRoot(assets.eye.clone()),
        PlayAnimation {
            graph: assets.eye_animation_graph.clone(),
            index: assets.eye_close,
        },
        IgnorePlayingState,
        TRANSITION_RENDER_LAYER,
        Name::new("Transition mesh"),
    ));

    commands.trigger(TransitionStarted);
    next_state.set(TransitionState::In);
    info!("Starting transition");
}

/// Once the transition mesh is added, we need to assign the render layer to it's children
/// (since the mesh is actually a Scene).
fn update_transition_mesh_children(
    query: Query<Entity, Added<Mesh3d>>,
    mut commands: Commands,
    transition_mesh: Single<Entity, With<TransitionMesh>>,
    child_of: Query<&ChildOf>,
) {
    for entity in &query {
        if child_of
            .iter_ancestors(entity)
            .any(|parent| parent == *transition_mesh)
        {
            commands.entity(entity).insert(TRANSITION_RENDER_LAYER);
        }
    }
}

fn on_transition_continue(
    _: On<ContinueTransition>,
    mut commands: Commands,
    transition_mesh: Single<Entity, With<TransitionMesh>>,
    assets: Res<PreLoadAssets>,
    mut next_state: ResMut<NextState<TransitionState>>,
) {
    commands.entity(*transition_mesh).insert((
        PlayAnimation {
            graph: assets.eye_animation_graph.clone(),
            index: assets.eye_open,
        },
        IgnorePlayingState,
    ));
    next_state.set(TransitionState::Out);
    info!("Continuing transition");
}

fn check_transition_state(
    mut commands: Commands,
    players: Query<(Entity, &AnimationPlayer), Without<PlayAnimation>>,
    child_of: Query<&ChildOf>,
    transition: Single<(Entity, Has<TransitionIn>, Has<TransitionOut>), With<TransitionMesh>>,
    current_state: Res<State<TransitionState>>,
    mut next_state: ResMut<NextState<TransitionState>>,
) {
    let transition = transition.into_inner();

    for (entity, player) in &players {
        if player.all_finished()
            && child_of
                .iter_ancestors(entity)
                .any(|parent| transition.0 == parent)
        {
            if *current_state == TransitionState::In {
                commands.trigger(TransitionReachedMiddle);
                next_state.set(TransitionState::Middle);
                commands.entity(transition.0).remove::<TransitionIn>();
                info!("Transition reached middle");
            } else if *current_state == TransitionState::Out {
                commands.trigger(TransitionEnded);
                commands.entity(transition.0).despawn();
                next_state.set(TransitionState::NotStarted);
                info!("Transition ended");
            }
        }
    }
}
